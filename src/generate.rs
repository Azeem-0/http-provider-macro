use crate::input::{EndpointDef, HttpMethod, HttpProviderInput};
use heck::ToSnakeCase;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use regex::Regex;

pub fn generate_provider(def: HttpProviderInput) -> syn::Result<TokenStream> {
    let struct_name = def.struct_name;
    let methods: Vec<TokenStream> = def
        .endpoints
        .iter()
        .map(generate_method)
        .collect::<Result<_, _>>()?;

    Ok(quote! {
        pub struct #struct_name {
            url: reqwest::Url,
            client: reqwest::Client,
            timeout: std::time::Duration,
        }

        impl #struct_name {
            pub fn new(url: reqwest::Url, timeout: u64) -> Self {
                let client = reqwest::Client::new();
                let timeout = std::time::Duration::from_secs(timeout);
                Self { url, client, timeout }
            }

            #(#methods)*
        }
    })
}

fn generate_method(def: &EndpointDef) -> syn::Result<TokenStream> {
    let path = def.path.value();
    let method = &def.method;

    // Auto-generate fn_name if needed
    let fn_name = if let Some(ref name) = def.fn_name {
        name.clone()
    } else {
        let method_str = format!("{:?}", def.method).to_lowercase();
        let path_str = def.path.value().trim_start_matches('/').replace("/", "_");
        let auto_name = format!("{}_{}", method_str, path_str).to_snake_case();
        Ident::new(&auto_name, def.path.span())
    };

    let req = def.req.as_ref();
    let res = &def.res;
    let headers = def.headers.as_ref();
    let query_params = def.query_params.as_ref();
    let path_params = def.path_params.as_ref();

    // === Parameter Generation ===
    let body_param = req.map(|r| quote! { body: &#r, }).unwrap_or_default();
    let headers_param = headers.map(|h| quote! { headers: #h, }).unwrap_or_default();
    let query_param = query_params
        .map(|q| quote! { query: #q, })
        .unwrap_or_default();
    let path_param = path_params
        .map(|p| quote! { path_params: &#p, })
        .unwrap_or_default();

    // === Path Parameter Substitution ===
    let path_replacement = if let Some(_) = path_params {
        let re = Regex::new(r"\{([a-zA-Z0-9_]+)\}").unwrap();
        let mut replacements = Vec::new();
        for cap in re.captures_iter(&path) {
            let param = &cap[1];
            let param_ident = Ident::new(param, proc_macro2::Span::call_site());
            replacements.push(quote! {
                    path = path.replace(concat!("{", #param, "}"), &path_params.#param_ident.to_string());
                });
        }
        quote! {
            let mut path = #path.trim_start_matches("/").to_string();
            #(#replacements)*
            let url = self.url.join(&path)
                .map_err(|e| format!("Failed to join url with the path {:?} : {:?}", path, e))?;
        }
    } else {
        quote! {
            let url = self.url.join(#path)
                .map_err(|e| format!("Failed to join url with the path {:?} : {:?}", #path, e))?;
        }
    };

    // === Request Building Logic ===
    let body_logic = if req.is_some() {
        quote! { request = request.json(body); }
    } else {
        quote! {}
    };
    let headers_logic = headers
        .map(|_| quote! { request = request.headers(headers.clone()); })
        .unwrap_or_default();
    let query_logic = query_params
        .map(|_| quote! { request = request.query(&query); })
        .unwrap_or_default();

    let request_builder = match method {
        HttpMethod::GET => quote! { self.client.get(url.clone()) },
        HttpMethod::POST => quote! { self.client.post(url.clone()) },
        HttpMethod::PUT => quote! { self.client.put(url.clone()) },
        HttpMethod::DELETE => quote! { self.client.delete(url.clone()) },
    };

    // === Final Method Definition ===
    let fn_tokens = quote! {
        pub async fn #fn_name(&self, #path_param #body_param #headers_param #query_param) -> Result<#res, String> {
            #path_replacement

            let mut request = #request_builder;
            #body_logic
            #headers_logic
            #query_logic

            let response = request
                .timeout(self.timeout)
                .send()
                .await
                .map_err(|e| format!("Failed to send request to {:?}: {:?}", url.to_string(), e))?;

            if !response.status().is_success() {
                return Err(format!("Request failed with status: {}", response.status()));
            }

            let parsed_response: #res = response
                .json()
                .await
                .map_err(|e| format!("Failed to deserialize response from {}: {}", url.to_string(), e))?;

            Ok(parsed_response)
        }
    };

    Ok(fn_tokens)
}

use crate::parser::{EndpointDef, HttpMethod, HttpProviderDef};
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate_provider(def: HttpProviderDef) -> syn::Result<TokenStream> {
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
            timeout : std::time::Duration,
        }

        impl #struct_name {
            pub fn new(url: reqwest::Url, timeout : u64) -> Self {
                let client = reqwest::Client::new();
                let timeout = std::time::Duration::from_secs(timeout);
                Self {
                    url,
                    client,
                    timeout,
                }
            }

            #(#methods)*
        }
    })
}

fn generate_method(def: &EndpointDef) -> syn::Result<TokenStream> {
    let fn_name = &def.fn_name;
    let path = def.path.value();
    let method = &def.method;
    let req = def.req.as_ref();
    let res = &def.res;
    let headers = def.headers.as_ref();
    let query_params = def.query_params.as_ref();

    // === 1. Build parameters ===
    let body_param = match (method, req) {
        (HttpMethod::GET, Some(r)) => quote! { body: &#r, },
        (HttpMethod::POST | HttpMethod::PUT, Some(r)) => quote! { body: &#r, },
        _ => quote! {},
    };

    let headers_param = if let Some(h) = headers {
        quote! { headers: #h, }
    } else {
        quote! {}
    };

    let query_param = if let Some(q) = query_params {
        quote! { query: #q, }
    } else {
        quote! {}
    };

    // // === 2. Build logic for body, headers, query
    let body_logic = match (method, req) {
        (HttpMethod::GET, Some(_)) => quote! {
            request = request.json(&body);
        },
        (HttpMethod::POST | HttpMethod::PUT, Some(_)) => quote! {
            request = request.json(body);
        },
        _ => quote! {},
    };

    let headers_logic = match headers {
        Some(_) => quote! {
            request = request.headers(headers.clone());
        },
        None => quote! {},
    };

    let query_logic = match query_params {
        Some(_) => quote! {
            request = request.query(&query);
        },
        _ => quote! {},
    };

    // === 3. Select HTTP method ===
    let request_builder = match method {
        HttpMethod::GET => quote! { self.client.get(url.clone()) },
        HttpMethod::POST => quote! { self.client.post(url.clone()) },
        HttpMethod::PUT => quote! { self.client.put(url.clone()) },
        HttpMethod::DELETE => quote! { self.client.delete(url.clone()) },
    };

    // === 4. Compose full function ===
    let fn_tokens = quote! {
        pub async fn #fn_name(&self, #body_param #headers_param #query_param) -> Result<#res, String> {
            let url = self.url.join(#path)
                .map_err(|e| format!("Failed to join url with the path {:?} : {:?}", #path, e))?;

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
                return Err(format!("Request failed with status: {}", response.status()).into());
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

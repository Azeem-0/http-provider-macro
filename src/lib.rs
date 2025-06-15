extern crate proc_macro;

use crate::{
    error::{MacroError, MacroResult},
    input::{EndpointDef, HttpMethod, HttpProviderInput},
};
use heck::ToSnakeCase;
use quote::quote;
use regex::Regex;
use syn::{parse_macro_input, Ident};

mod error;
mod generate;
mod input;

#[proc_macro]
pub fn http_provider(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = parse_macro_input!(input as HttpProviderInput);

    let mut expander = HttpProviderMacroExpander::new();

    match expander.expand(parsed) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

struct HttpProviderMacroExpander;

impl HttpProviderMacroExpander {
    fn new() -> Self {
        Self
    }

    fn expand(&mut self, input: HttpProviderInput) -> MacroResult<proc_macro2::TokenStream> {
        if input.endpoints.is_empty() {
            return Err(MacroError::Custom {
                message: "No endpoints defined".to_string(),
                span: input.struct_name.span(),
            });
        }

        let struct_name = input.struct_name;

        let methods: Vec<proc_macro2::TokenStream> = input
            .endpoints
            .iter()
            .map(|endpoint| self.expand_method(endpoint))
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

    fn expand_method(&self, endpoint: &EndpointDef) -> MacroResult<proc_macro2::TokenStream> {
        let method_expander = MethodExpander::new(endpoint);

        let fn_signature = method_expander.expand_fn_signature();
        let url_construction = method_expander.build_url_construction();
        let request_building = method_expander.build_request();
        let response_handling = method_expander.build_response_handling();

        Ok(quote! {
            #fn_signature {
                #url_construction
                #request_building
                #response_handling
            }
        })
    }
}

struct MethodExpander<'a> {
    def: &'a EndpointDef,
}

impl<'a> MethodExpander<'a> {
    fn new(def: &'a EndpointDef) -> Self {
        Self { def }
    }

    fn expand_fn_signature(&self) -> proc_macro2::TokenStream {
        let path = self.def.path.value();
        let method = &self.def.method;

        let fn_name = if let Some(ref name) = self.def.fn_name {
            name.clone()
        } else {
            let method_str = format!("{:?}", method).to_lowercase();
            let path_str = path.trim_start_matches('/').replace("/", "_");
            let auto_name = format!("{}_{}", method_str, path_str).to_snake_case();
            Ident::new(&auto_name, self.def.path.span())
        };

        let res = &self.def.res;

        let mut params = vec![];

        if let Some(path_params) = &self.def.path_params {
            params.push(quote! { path_params: &#path_params });
        }
        if let Some(body) = &self.def.req {
            params.push(quote! { body: &#body });
        }
        if let Some(headers) = &self.def.headers {
            params.push(quote! { headers: #headers });
        }
        if let Some(query) = &self.def.query_params {
            params.push(quote! { query: #query });
        }

        quote! {
            pub async fn #fn_name(&self, #(#params),*) -> Result<#res, String>
        }
    }

    fn build_url_construction(&self) -> proc_macro2::TokenStream {
        let path = self.def.path.value();

        if self.def.path_params.is_some() {
            let re = Regex::new(r"\{([a-zA-Z0-9_]+)\}").unwrap();
            let mut replacements = Vec::new();

            for cap in re.captures_iter(&path) {
                let param_name = &cap[1];
                let ident = Ident::new(param_name, proc_macro2::Span::call_site());
                replacements.push(quote! {
                    path = path.replace(concat!("{", #param_name, "}"), &path_params.#ident.to_string());
                });
            }

            quote! {
                let mut path = #path.to_string();
                #(#replacements)*
                let url = self.url.join(&path)
                    .map_err(|e| format!("Failed to construct URL: {}", e))?;
            }
        } else {
            quote! {
                let url = self.url.join(#path)
                    .map_err(|e| format!("Failed to construct URL: {}", e))?;
            }
        }
    }

    fn build_request(&self) -> proc_macro2::TokenStream {
        let method_call = match self.def.method {
            HttpMethod::GET => quote! { self.client.get(url) },
            HttpMethod::POST => quote! { self.client.post(url) },
            HttpMethod::PUT => quote! { self.client.put(url) },
            HttpMethod::DELETE => quote! { self.client.delete(url) },
        };

        let mut request_modifications = Vec::new();

        // Add body handling
        if self.def.req.is_some() {
            request_modifications.push(quote! {
                request = request.json(body);
            });
        }

        if self.def.query_params.is_some() {
            request_modifications.push(quote! {
                request = request.query(query_params);
            });
        }

        // Add headers
        if self.def.headers.is_some() {
            request_modifications.push(quote! {
                let request = request.headers(headers);
            });
        }

        quote! {
            let mut request = #method_call;
            #(#request_modifications)*
        }
    }

    fn build_response_handling(&self) -> proc_macro2::TokenStream {
        let res = &self.def.res;

        quote! {
            let response = request
                .send()
                .await
                .map_err(|e| format!("Request failed: {}", e))?;

            let status = response.status();
            if !status.is_success() {
                return Err(format!("HTTP request failed with status {}: {}",
                    status.as_u16(),
                    status.canonical_reason().unwrap_or("Unknown error")
                ).into());
            }

            let result: #res = response
                .json()
                .await
                .map_err(|e| format!("Failed to deserialize response: {}", e))?;

            Ok(result)
        }
    }
}

//! # HTTP Provider Macro
//!
//! A procedural macro for generating HTTP client providers with compile-time endpoint definitions.
//! This macro eliminates boilerplate code when creating HTTP clients by automatically generating
//! methods for your API endpoints.
//!
//! ## Features
//!
//! - **Zero runtime overhead** - All HTTP client code is generated at compile time
//! - **Automatic method generation** - Function names auto-generated from HTTP method and path
//! - **Type-safe requests/responses** - Full Rust type checking for all parameters
//! - **Full HTTP method support** - GET, POST, PUT, DELETE
//! - **Path parameters** - Dynamic URL path substitution with `{param}` syntax
//! - **Query parameters** - Automatic query string serialization
//! - **Custom headers** - Per-request header support
//! - **Async/await** - Built on reqwest with full async support
//! - **Configurable timeouts** - Per-client timeout configuration
//!
//! ## Quick Start
//!
//! ```rust
//! use http_provider_macro::http_provider;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize, Debug)]
//! struct User {
//!     id: u32,
//!     name: String,
//! }
//!
//! #[derive(Serialize)]
//! struct CreateUser {
//!     name: String,
//! }
//!
//! // Define your HTTP provider
//! http_provider!(
//!     UserApi,
//!     {
//!         {
//!             path: "/users",
//!             method: GET,
//!             res: Vec<User>,
//!         },
//!         {
//!             path: "/users",
//!             method: POST,
//!             req: CreateUser,
//!             res: User,
//!         }
//!     }
//! );
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let base_url = reqwest::Url::parse("https://api.example.com")?;
//! let client = UserApi::new(base_url, 30);
//!
//! // Auto-generated methods
//! let users = client.get_users().await?;
//! let new_user = client.post_users(&CreateUser {
//!     name: "John".to_string()
//! }).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Endpoint Configuration
//!
//! Each endpoint is defined within braces with these fields:
//!
//! ### Required Fields
//! - `path`: API endpoint path (string literal)
//! - `method`: HTTP method (GET, POST, PUT, DELETE)
//! - `res`: Response type implementing `serde::Deserialize`
//!
//! ### Optional Fields
//! - `fn_name`: Custom function name (auto-generated if omitted)
//! - `req`: Request body type implementing `serde::Serialize`
//! - `headers`: Header type (typically `reqwest::header::HeaderMap`)
//! - `query_params`: Query parameters type implementing `serde::Serialize`
//! - `path_params`: Path parameters type with fields matching `{param}` in path
//!
//! ## Examples
//!
//! ### Path Parameters
//!
//! ```rust
//! # use http_provider_macro::http_provider;
//! # use serde::{Deserialize, Serialize};
//! #[derive(Serialize)]
//! struct UserPath {
//!     id: u32,
//! }
//!
//! #[derive(Deserialize)]
//! struct User {
//!     id: u32,
//!     name: String,
//! }
//!
//! http_provider!(
//!     UserApi,
//!     {
//!         {
//!             path: "/users/{id}",
//!             method: GET,
//!             path_params: UserPath,
//!             res: User,
//!         }
//!     }
//! );
//! ```
//!
//! ### Query Parameters and Headers
//!
//! ```rust
//! # use http_provider_macro::http_provider;
//! # use serde::{Deserialize, Serialize};
//! # use reqwest::header::HeaderMap;
//! #[derive(Serialize)]
//! struct SearchQuery {
//!     q: String,
//!     limit: u32,
//! }
//!
//! #[derive(Deserialize)]
//! struct SearchResults {
//!     results: Vec<String>,
//! }
//!
//! http_provider!(
//!     SearchApi,
//!     {
//!         {
//!             path: "/search",
//!             method: GET,
//!             fn_name: search_items,
//!             query_params: SearchQuery,
//!             headers: HeaderMap,
//!             res: SearchResults,
//!         }
//!     }
//! );
//! ```

extern crate proc_macro;

use crate::{
    error::{MacroError, MacroResult},
    input::{EndpointDef, HttpMethod, HttpProviderInput},
};
use heck::ToSnakeCase;
use proc_macro2::Span;
use quote::quote;
use regex::Regex;
use syn::{parse_macro_input, spanned::Spanned, Ident};

mod error;
mod input;

/// Generates an HTTP client provider struct with methods for each defined endpoint.
///
/// This macro takes a struct name and a list of endpoint definitions, generating
/// a complete HTTP client with methods for each endpoint.
///
/// # Syntax
///
/// ```text
/// http_provider!(
///     StructName,
///     {
///         {
///             path: "/endpoint/path",
///             method: HTTP_METHOD,
///             [fn_name: custom_function_name,]
///             [req: RequestType,]
///             res: ResponseType,
///             [headers: HeaderType,]
///             [query_params: QueryType,]
///             [path_params: PathParamsType,]
///         },
///         // ... more endpoints
///     }
/// );
/// ```
///
/// # Generated Structure
///
/// The macro generates:
/// - A struct with `url`, `client`, and `timeout` fields
/// - A `new(url: reqwest::Url, timeout: u64)` constructor
/// - One async method per endpoint definition
///
/// # Method Naming
///
/// When `fn_name` is not provided, method names are auto-generated as:
/// `{method}_{path}` where path separators become underscores.
///
/// # Examples
///
/// ```rust
/// use http_provider_macro::http_provider;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct User {
///     id: u32,
///     name: String,
/// }
///
/// http_provider!(
///     UserClient,
///     {
///         {
///             path: "/users",
///             method: GET,
///             res: Vec<User>,
///         },
///         {
///             path: "/users/{id}",
///             method: GET,
///             path_params: UserPath,
///             res: User,
///         }
///     }
/// );
///
/// #[derive(Serialize)]
/// struct UserPath {
///     id: u32,
/// }
/// ```
#[proc_macro]
pub fn http_provider(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = parse_macro_input!(input as HttpProviderInput);

    let mut expander = HttpProviderMacroExpander::new();

    match expander.expand(parsed) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Main expander that generates the HTTP provider struct and its methods.
struct HttpProviderMacroExpander;

impl HttpProviderMacroExpander {
    fn new() -> Self {
        Self
    }

    /// Expands the macro input into a complete HTTP provider implementation.
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
            .filter(|endpoint| endpoint.trait_impl.is_none())
            .map(|endpoint| self.expand_method(endpoint))
            .collect::<Result<_, _>>()?;

        let trait_methods: Vec<proc_macro2::TokenStream> = input
            .endpoints
            .iter()
            .filter(|endpoint| endpoint.trait_impl.is_some())
            .map(|endpoint| self.expand_trait_method(&struct_name, endpoint))
            .collect::<Result<_, _>>()?;

        Ok(quote! {
            pub struct #struct_name {
                url: reqwest::Url,
                client: reqwest::Client,
                timeout: std::time::Duration,
            }

            impl #struct_name {
                /// Creates a new HTTP provider instance.
                ///
                /// # Arguments
                /// * `url` - Base URL for all requests
                /// * `timeout` - Request timeout in milliseconds
                pub fn new(url: reqwest::Url, timeout: u64) -> Self {
                    let client = reqwest::Client::new();
                    let timeout = std::time::Duration::from_millis(timeout);
                    Self { url, client, timeout }
                }

                #(#methods)*
            }

            #(#trait_methods)*
        })
    }

    fn expand_trait_method(
        &self,
        struct_name: &Ident,
        endpoint: &EndpointDef,
    ) -> MacroResult<proc_macro2::TokenStream> {
        let method = self.expand_method(endpoint)?;

        let trait_impl = endpoint
            .trait_impl
            .as_ref()
            .ok_or_else(|| MacroError::Custom {
                message: "Trait impl is not configured".to_string(),
                span: method.span(),
            })?;

        Ok(quote! {
            impl #trait_impl for #struct_name {
                #method
            }
        })
    }

    /// Generates a single HTTP method for an endpoint definition.
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
/// Handles the expansion of individual HTTP method implementations
struct MethodExpander<'a> {
    def: &'a EndpointDef,
}

impl<'a> MethodExpander<'a> {
    fn new(def: &'a EndpointDef) -> Self {
        Self { def }
    }

    /// Generates the function signature for an endpoint method.
    fn expand_fn_signature(&self) -> proc_macro2::TokenStream {
        let method = &self.def.method;

        // Handle the function name logic based on whether path is provided
        let fn_name = if let Some(ref name) = self.def.fn_name {
            name.clone()
        } else {
            let method_str = format!("{:?}", method).to_lowercase();

            // Handle the case where the path is optional
            let auto_name = if let Some(ref path) = self.def.path {
                let path_str = path.value().trim_start_matches('/').replace("/", "_");
                format!("{}_{}", method_str, path_str).to_snake_case()
            } else {
                format!("{}_no_path", method_str).to_snake_case() // Default function name if no path
            };

            Ident::new(
                &auto_name,
                self.def
                    .path
                    .as_ref()
                    .map_or_else(Span::call_site, |p| p.span()),
            )
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
        if let Some(query_params) = &self.def.query_params {
            params.push(quote! { query_params: &#query_params });
        }

        // Determine if this is for a trait implementation
        let is_trait_impl = self.def.trait_impl.is_some();
        if is_trait_impl {
            quote! {
                async fn #fn_name(&self, #(#params),*) -> Result<#res,String>
            }
        } else {
            quote! {
                pub async fn #fn_name(&self, #(#params),*) -> Result<#res, String>
            }
        }
    }

    /// Generates URL construction logic, handling path parameter substitution.
    fn build_url_construction(&self) -> proc_macro2::TokenStream {
        // If path is None, we just use the base URL as is.
        let path = if let Some(ref path) = self.def.path {
            path.value()
        } else {
            // If no path, just use the URL as is
            return quote! {
                let url = self.url.clone(); // Use the base URL as is
            };
        };

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

    /// Generates request building logic including body, headers, and query parameters
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

    /// Generates response handling logic.
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

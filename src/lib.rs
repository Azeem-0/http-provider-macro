extern crate proc_macro;
use generate::generate_provider;
use parser::HttpProviderDef;
use proc_macro::TokenStream;
use syn::parse_macro_input;

mod generate;
mod parser;

#[proc_macro]
pub fn http_provider(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as HttpProviderDef);
    match generate_provider(parsed) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

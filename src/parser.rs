use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    Ident, LitStr, Token, Type,
};

#[derive(Debug, Clone)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
}

impl Parse for HttpMethod {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        match ident.to_string().to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::GET),
            "POST" => Ok(HttpMethod::POST),
            "PUT" => Ok(HttpMethod::PUT),
            "DELETE" => Ok(HttpMethod::DELETE),
            _ => Err(syn::Error::new(
                ident.span(),
                format!("Unsupported HTTP method: {}", ident),
            )),
        }
    }
}

pub struct HttpProviderDef {
    pub struct_name: Ident,
    pub endpoints: Vec<EndpointDef>,
}

pub struct EndpointDef {
    pub path: LitStr,
    pub method: HttpMethod,
    pub fn_name: Ident,
    pub req: Option<Type>,
    pub res: Type,
    pub headers: Option<Type>,
    pub query_params: Option<Type>,
}

impl Parse for HttpProviderDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let struct_name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;

        let content;
        braced!(content in input);
        let items: Punctuated<EndpointDef, Token![,]> =
            content.parse_terminated(EndpointDef::parse, Token![,])?;
        Ok(Self {
            struct_name,
            endpoints: items.into_iter().collect(),
        })
    }
}

impl Parse for EndpointDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        braced!(content in input);

        let mut path = None;
        let mut method = None;
        let mut fn_name = None;
        let mut req = None;
        let mut res = None;
        let mut headers = None;
        let mut query_params = None;

        while !content.is_empty() {
            let field: Ident = content.parse()?;
            content.parse::<Token![:]>()?;

            match field.to_string().as_str() {
                "path" => path = Some(content.parse()?),
                "method" => method = Some(content.parse()?),
                "fn_name" => fn_name = Some(content.parse()?),
                "req" => req = Some(content.parse()?),
                "res" => res = Some(content.parse()?),
                "headers" => headers = Some(content.parse()?),
                "query_params" => query_params = Some(content.parse()?),
                _ => return Err(syn::Error::new(field.span(), "unexpected field")),
            }

            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }

        Ok(EndpointDef {
            path: path.ok_or_else(|| syn::Error::new(content.span(), "missing `path`"))?,
            method: method.ok_or_else(|| syn::Error::new(content.span(), "missing `method`"))?,
            fn_name: fn_name.ok_or_else(|| syn::Error::new(content.span(), "missing `fn_name`"))?,
            req,
            res: res.ok_or_else(|| syn::Error::new(content.span(), "missing `res`"))?,
            headers,
            query_params,
        })
    }
}

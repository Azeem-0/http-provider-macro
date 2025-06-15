use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    Ident, LitStr, Token, Type,
};

/// Represents HTTP methods supported by the provider macro.
///
/// These methods align with standard HTTP/1.1 methods and are used
/// to define the type of request for each endpoint.
#[derive(Debug, Clone)]
pub enum HttpMethod {
    /// HTTP GET method for retrieving resources
    GET,

    /// HTTP POST method for creating resources
    POST,

    /// HTTP PUT method for updating resources
    PUT,

    /// HTTP DELETE method for removing resources
    DELETE,
}

impl Parse for HttpMethod {
    /// Parses an HTTP method from the input stream.
    ///
    /// # Arguments
    /// * `input` - The parse stream containing the method identifier
    ///
    /// # Returns
    /// * `Result<Self>` - The parsed HTTP method or an error if method is unsupported
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

/// Root structure for parsing the HTTP provider macro input.
///
/// This structure represents the complete macro definition including
/// the provider struct name and all its endpoint definitions.
///
/// # Example
/// ```ignore
/// MyApiClient, {
///     {
///         path: "/users",
///         method: GET,
///         res: Vec<User>
///     }
/// }
/// ```
pub struct HttpProviderInput {
    /// Name of the provider struct that will be generated
    pub struct_name: Ident,

    /// Collection of endpoint definitions
    pub endpoints: Vec<EndpointDef>,
}

/// Represents a single API endpoint configuration.
///
/// Each endpoint definition includes all necessary information to generate
/// a typed client method for making HTTP requests.
///
/// # Fields
/// * `path` - Optional URL path for the endpoint (e.g., "/api/users")
/// * `method` - The HTTP method to use
/// * `fn_name` - Optional custom name for the generated function
/// * `req` - Optional request body type
/// * `res` - Response type that will be deserialized
/// * `headers` - Optional custom headers type
/// * `query_params` - Optional query parameters type
/// * `path_params` - Optional path parameters type
pub struct EndpointDef {
    pub path: Option<LitStr>,
    pub method: HttpMethod,
    pub fn_name: Option<Ident>,
    pub req: Option<Type>,
    pub res: Type,
    pub headers: Option<Type>,
    pub query_params: Option<Type>,
    pub path_params: Option<Type>,
}

impl Parse for HttpProviderInput {
    /// Parses the complete macro input into a structured form.
    ///
    /// Expects input in the format:
    /// `struct_name, { endpoint1, endpoint2, ... }`
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
    /// Parses a single endpoint definition block.
    ///
    /// # Format
    /// ```ignore
    /// {
    ///     path: "/path",
    ///     method: GET,
    ///     fn_name: custom_name,      // optional
    ///     req: RequestType,          // optional
    ///     res: ResponseType,         // required
    ///     headers: HeadersType,      // optional
    ///     query_params: QueryType,   // optional
    ///     path_params: ParamsType    // optional
    /// }
    /// ```
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
        let mut path_params = None;

        // Iteratively parse each key-value pair inside the endpoint block
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
                "path_params" => path_params = Some(content.parse()?),
                _ => return Err(syn::Error::new(field.span(), "unexpected field")),
            }

            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }

        Ok(EndpointDef {
            path,
            method: method.ok_or_else(|| syn::Error::new(content.span(), "missing `method`"))?,
            fn_name,
            req,
            res: res.ok_or_else(|| syn::Error::new(content.span(), "missing `res`"))?,
            headers,
            query_params,
            path_params,
        })
    }
}

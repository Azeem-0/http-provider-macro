# http-provider-macro

[![Crates.io](https://img.shields.io/crates/v/http-provider-macro.svg)](https://crates.io/crates/http-provider-macro)
[![Documentation](https://docs.rs/http-provider-macro/badge.svg)](https://docs.rs/http-provider-macro)
[![License](https://img.shields.io/crates/l/http-provider-macro.svg)](https://github.com/azeem-0/http-provider-macro#license)

A procedural macro for generating type-safe HTTP client providers in Rust. This crate allows you to declaratively define HTTP endpoints and automatically generates async client methods with proper error handling, serialization, and deserialization.

## Features

- 🚀 **Declarative API**: Define HTTP endpoints using a simple macro syntax
- 🔒 **Type Safety**: Compile-time guarantees for request/response types
- ⚡ **Async/Await**: Built on `reqwest` with full async support
- 🎯 **HTTP Methods**: Support for GET, POST, PUT, and DELETE
- 📝 **Flexible Parameters**: Optional headers, query parameters, and request bodies
- ⏱️ **Timeout Configuration**: Configurable request timeouts
- 🛡️ **Error Handling**: Comprehensive error handling with detailed messages

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
http-provider-macro = "0.1.0"
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
```

## Quick Start

```rust
use http_provider_macro::http_provider;
use serde::{Deserialize, Serialize};
use reqwest::Url;

// Define your request and response types
#[derive(Serialize, Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: u64,
    name: String,
    email: String,
}

#[derive(Serialize, Deserialize)]
struct UserQuery {
    limit: u32,
    offset: u32,
}

// Generate the HTTP client provider
http_provider!(
    UserApiClient,
    {
        {
            path: "/users",
            method: GET,
            fn_name: get_users,
            res: Vec<User>,
            query_params: UserQuery,
        },
        {
            path: "/users",
            method: POST,
            fn_name: create_user,
            req: CreateUserRequest,
            res: User,
        },
        {
            path: "/users/{id}",
            method: GET,
            fn_name: get_user,
            res: User,
        }
    }
);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the client
    let base_url = Url::parse("https://api.example.com")?;
    let client = UserApiClient::new(base_url, 30); // 30 second timeout

    // Create a new user
    let new_user = CreateUserRequest {
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    };
    
    let user = client.create_user(&new_user).await?;
    println!("Created user: {:?}", user);

    // Get users with query parameters
    let query = UserQuery { limit: 10, offset: 0 };
    let users = client.get_users(query).await?;
    println!("Users: {:?}", users);

    Ok(())
}
```

## Macro Syntax

The `http_provider!` macro takes a struct name followed by endpoint definitions:

```rust
http_provider!(
    StructName,
    {
        {
            path: "/endpoint/path",
            method: HTTP_METHOD,
            fn_name: function_name,
            // Optional fields:
            req: RequestType,
            res: ResponseType,
            headers: HeaderType,
            query_params: QueryParamsType,
        },
        // ... more endpoints
    }
);
```

### Field Descriptions

- **`path`** (required): The endpoint path relative to the base URL
- **`method`** (required): HTTP method (`GET`, `POST`, `PUT`, `DELETE`)
- **`fn_name`** (required): Name of the generated async function
- **`req`** (optional): Request body type (for POST/PUT requests)
- **`res`** (required): Response type that implements `Deserialize`
- **`headers`** (optional): Headers type (typically `reqwest::header::HeaderMap`)
- **`query_params`** (optional): Query parameters type that implements `Serialize`

## Advanced Usage

### With Headers and Query Parameters

```rust
use reqwest::header::HeaderMap;

#[derive(Serialize)]
struct ApiQuery {
    api_key: String,
    version: String,
}

http_provider!(
    ApiClient,
    {
        {
            path: "/data",
            method: GET,
            fn_name: get_data,
            res: ApiResponse,
            headers: HeaderMap,
            query_params: ApiQuery,
        }
    }
);

// Usage
let mut headers = HeaderMap::new();
headers.insert("authorization", "Bearer token".parse()?);

let query = ApiQuery {
    api_key: "your-key".to_string(),
    version: "v1".to_string(),
};

let response = client.get_data(headers, query).await?;
```

### Error Handling

The generated methods return `Result<T, String>` where `T` is your response type:

```rust
match client.get_user().await {
    Ok(user) => println!("User: {:?}", user),
    Err(error) => eprintln!("Request failed: {}", error),
}
```

Common error scenarios:
- Network connectivity issues
- HTTP error status codes (4xx, 5xx)
- JSON deserialization failures
- Request timeout

## Generated API

For each endpoint, the macro generates:

1. **Struct**: A client struct with `url`, `client`, and `timeout` fields
2. **Constructor**: `new(url: Url, timeout: u64) -> Self`
3. **Methods**: Async methods for each endpoint with appropriate parameters

## Limitations

- Only supports JSON request/response bodies
- Limited to GET, POST, PUT, and DELETE methods
- Error type is currently `String` (will be improved in future versions)
- No built-in retry or circuit breaker functionality

## Examples

See the `tests/` directory for comprehensive examples including:
- Basic CRUD operations
- Custom headers and query parameters
- Error handling scenarios
- Mock server testing with WireMock

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## License

This project is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Changelog

### 0.1.0
- Initial release
- Support for GET, POST, PUT, DELETE methods
- JSON request/response handling
- Configurable timeouts
- Optional headers and query parameters
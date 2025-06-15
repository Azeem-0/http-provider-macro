# HTTP Provider Macro

A Rust procedural macro that generates HTTP client providers with compile-time endpoint definitions. This macro eliminates boilerplate code for creating HTTP clients by automatically generating methods for your API endpoints.

## Features

- 🚀 **Zero runtime overhead** - All HTTP client code is generated at compile time
- 🔧 **Automatic method generation** - Function names auto-generated from HTTP method and path
- 🎯 **Type-safe requests/responses** - Full Rust type checking for all parameters
- 🌐 **Full HTTP method support** - GET, POST, PUT, DELETE
- 📝 **Path parameters** - Dynamic URL path substitution with `{param}` syntax
- 🔍 **Query parameters** - Automatic query string serialization
- 📋 **Custom headers** - Per-request header support
- ⚡ **Async/await** - Built on reqwest with full async support
- ⏱️ **Configurable timeouts** - Per-client timeout configuration

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
http-provider-macro = "0.1.0"
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
```

## Basic Usage

```rust
use http_provider_macro::http_provider;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[derive(Serialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

// Define your HTTP provider
http_provider!(
    UserApiProvider,
    {
        {
            path: "/users",
            method: GET,
            res: Vec<User>,
        },
        {
            path: "/users",
            method: POST,
            req: CreateUserRequest,
            res: User,
        },
        {
            path: "/users/{id}",
            method: GET,
            path_params: PathParams,
            res: User,
        }
    }
);

#[derive(Serialize)]
struct PathParams {
    id: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = reqwest::Url::parse("https://api.example.com")?;
    let client = UserApiProvider::new(base_url, 30); // 30 second timeout

    // GET /users - auto-generated method name: get_users
    let users = client.get_users().await?;
    println!("Users: {:?}", users);

    // POST /users - auto-generated method name: post_users  
    let new_user = client.post_users(&CreateUserRequest {
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    }).await?;
    println!("Created user: {:?}", new_user);

    // GET /users/{id} - auto-generated method name: get_users_id
    let user = client.get_users_id(&PathParams { id: 1 }).await?;
    println!("User: {:?}", user);

    Ok(())
}
```

## Endpoint Configuration

Each endpoint is defined within braces `{}` with the following fields:

### Required Fields

- **`path`**: The API endpoint path (string literal)
- **`method`**: HTTP method (`GET`, `POST`, `PUT`, `DELETE`)  
- **`res`**: Response type that implements `Deserialize`

### Optional Fields

- **`fn_name`**: Custom function name (defaults to auto-generated)
- **`req`**: Request body type that implements `Serialize`
- **`headers`**: Header type (typically `reqwest::header::HeaderMap`)
- **`query_params`**: Query parameters type that implements `Serialize`
- **`path_params`**: Path parameters type with fields matching `{param}` in path

## Advanced Examples

### Custom Function Names and Headers

```rust
use reqwest::header::HeaderMap;

http_provider!(
    ApiProvider,
    {
        {
            path: "/protected/data",
            method: GET,
            fn_name: fetch_protected_data,
            res: ApiResponse,
            headers: HeaderMap,
        }
    }
);

// Usage
let mut headers = HeaderMap::new();
headers.insert("Authorization", "Bearer token123".parse()?);
let data = client.fetch_protected_data(headers).await?;
```

### Query Parameters

```rust
#[derive(Serialize)]
struct SearchQuery {
    q: String,
    limit: u32,
    offset: u32,
}

http_provider!(
    SearchProvider,
    {
        {
            path: "/search",
            method: GET,
            query_params: SearchQuery,
            res: SearchResults,
        }
    }
);

// Usage
let results = client.get_search(&SearchQuery {
    q: "rust".to_string(),
    limit: 10,
    offset: 0,
}).await?;
```

### Complex Path Parameters

```rust
#[derive(Serialize)]
struct ResourcePath {
    user_id: u32,
    resource_id: String,
}

http_provider!(
    ResourceProvider,
    {
        {
            path: "/users/{user_id}/resources/{resource_id}",
            method: GET,
            path_params: ResourcePath,
            res: Resource,
        }
    }
);

// Usage  
let resource = client.get_users_user_id_resources_resource_id(&ResourcePath {
    user_id: 123,
    resource_id: "abc-def".to_string(),
}).await?;
```

### All Parameters Combined

```rust
http_provider!(
    CompleteProvider,
    {
        {
            path: "/api/v1/users/{user_id}/posts",
            method: POST,
            fn_name: create_user_post,
            path_params: UserPath,
            req: CreatePostRequest,
            res: Post,
            headers: HeaderMap,
            query_params: PostQuery,
        }
    }
);

// Usage
let post = client.create_user_post(
    &UserPath { user_id: 123 },
    &CreatePostRequest { title: "Hello".to_string() },
    headers,
    &PostQuery { draft: false },
).await?;
```

## Generated Code Structure

The macro generates:

1. **Struct Definition**: A provider struct with `url`, `client`, and `timeout` fields
2. **Constructor**: `new(url: reqwest::Url, timeout: u64) -> Self`
3. **HTTP Methods**: One async method per endpoint definition

### Method Signatures

Generated methods follow this pattern:

```rust
pub async fn method_name(
    &self,
    path_params: &PathParamsType,    // if path_params specified
    body: &RequestType,              // if req specified  
    headers: HeaderMap,              // if headers specified
    query: &QueryType,               // if query_params specified
) -> Result<ResponseType, String>
```

### Auto-generated Function Names

When `fn_name` is not specified, names are generated as:
- `{method}_{path}` where path slashes become underscores
- Examples:
  - `GET /users` → `get_users`
  - `POST /api/v1/posts` → `post_api_v1_posts`
  - `PUT /users/{id}` → `put_users_id`

## Error Handling

All generated methods return `Result<T, String>` where errors include:

- **URL construction errors**: Invalid path parameter substitution
- **Network errors**: Connection timeouts, DNS failures, etc.
- **HTTP errors**: Non-2xx status codes with status information
- **Deserialization errors**: JSON parsing failures

## Requirements

- **Rust 1.70+**: For latest async/await and procedural macro features
- **reqwest**: HTTP client library
- **serde**: Serialization framework
- **tokio**: Async runtime

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
# reqwest-derive

[![Crates.io](https://img.shields.io/crates/v/reqwest-derive.svg)](https://crates.io/crates/reqwest-derive)
[![Documentation](https://docs.rs/reqwest-derive/badge.svg)](https://docs.rs/reqwest-derive)
[![License](https://img.shields.io/crates/l/reqwest-derive.svg)](https://github.com/yourusername/reqwest-derive#license)
[![Build Status](https://github.com/yourusername/reqwest-derive/workflows/CI/badge.svg)](https://github.com/yourusername/reqwest-derive/actions)

A procedural macro that automatically generates HTTP API client methods for your structs using `reqwest`. Say goodbye to boilerplate HTTP client code!

## Features

- 🚀 **Zero boilerplate** - Just annotate your response structs
- 🔧 **Configurable timeouts** - Set per-endpoint timeouts
- 🛡️ **Error handling** - Comprehensive error messages with context  
- 📦 **Easy integration** - Works seamlessly with existing `reqwest` and `serde` code
- ⚡ **Async/await ready** - Built for modern async Rust
- 🧪 **Well tested** - Comprehensive test suite with mock servers

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
reqwest-derive = "0.1"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
```

## Usage

```rust
use reqwest_derive::Provider;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[provider(url = "https://api.github.com/users/octocat", timeout = 10)]
struct GitHubUser {
    pub login: String,
    pub id: u64,
    pub avatar_url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The macro automatically generates a `fetch()` method
    let user = GitHubUser::fetch().await?;
    println!("User: {} (ID: {})", user.login, user.id);
    Ok(())
}
```

## Advanced Usage

### Custom HTTP Client

```rust
use reqwest::Client;

// Use a custom client with specific configuration
let client = Client::builder()
    .user_agent("my-app/1.0")
    .build()?;
    
let user = GitHubUser::fetch_with_client(&client).await?;
```

### Error Handling

The generated methods return detailed errors:

```rust
match GitHubUser::fetch().await {
    Ok(user) => println!("Success: {:?}", user),
    Err(e) => {
        eprintln!("Failed to fetch user: {}", e);
        // Error messages include URL and detailed context
    }
}
```

## Macro Attributes

### `#[provider]`

The main attribute that generates the HTTP client implementation.

#### Parameters

- `url` (required): The API endpoint URL as a string literal
- `timeout` (optional): Request timeout in seconds (default: 30)

#### Examples

```rust
// Basic usage
#[provider(url = "https://api.example.com/data")]
struct ApiResponse {
    data: String,
}

// With custom timeout
#[provider(url = "https://api.example.com/data", timeout = 5)]
struct FastApiResponse {
    result: Vec<String>,
}
```

## Generated Methods

For each struct annotated with `#[provider]`, the following methods are generated:

### `fetch() -> Result<Self, Box<dyn std::error::Error>>`

Creates a new HTTP client and fetches the data.

### `fetch_with_client(client: &reqwest::Client) -> Result<Self, Box<dyn std::error::Error>>`

Uses the provided HTTP client to fetch the data. Useful for sharing clients across multiple requests or when you need custom client configuration.

## Error Types

The generated methods can return these error types:

- **Network errors**: Connection failures, DNS resolution issues
- **Timeout errors**: When requests exceed the specified timeout
- **HTTP errors**: 4xx and 5xx status codes with detailed messages
- **Deserialization errors**: JSON parsing or type conversion failures

All errors include contextual information like the URL and specific failure reason.

## Requirements

- Rust 1.56+ (2021 edition)
- Your response structs must implement `serde::Deserialize`
- Requires `tokio` runtime for async execution

## Examples

Check out the [`examples/`](examples/) directory for more comprehensive usage examples:

- [Basic usage](examples/basic.rs)
- [Multiple endpoints](examples/multiple_endpoints.rs)
- [Custom client configuration](examples/custom_client.rs)
- [Error handling](examples/error_handling.rs)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Development Setup

```bash
git clone https://github.com/yourusername/reqwest-derive.git
cd reqwest-derive
cargo test
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run integration tests
cargo test --test integration_tests
```

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for a detailed changelog.

## Acknowledgments

- Built on top of the excellent [`reqwest`](https://github.com/seanmonstar/reqwest) HTTP client
- Inspired by the need to reduce boilerplate in microservice architectures
- Thanks to the Rust community for feedback and contributions

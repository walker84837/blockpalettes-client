# blockpalettes-client

[![crates.io version](https://img.shields.io/crates/v/blockpalettes-client.svg)](https://crates.io/crates/blockpalettes-client)
[![docs.rs documentation](https://img.shields.io/docsrs/blockpalettes-client)](https://docs.rs/blockpalettes-client)
[![crates.io license](https://img.shields.io/crates/l/blockpalettes-client.svg)](https://crates.io/crates/blockpalettes-client)
[![crates.io downloads](https://img.shields.io/crates/d/blockpalettes-client.svg)](https://crates.io/crates/blockpalettes-client)
[![dependency status](https://deps.rs/crate/blockpalettes-client/latest/status.svg)](https://deps.rs/crate/blockpalettes-client)

Unofficial Rust client for BlockPalettes with built-in HTTP and scraping support.

## Installation

Use `cargo add`:

```bash
cargo add blockpalettes-client
```

## Usage

Here's a very basic example of how to use `blockpalettes-client` to search for blocks:

```rust
use blockpalettes_client::BlockPalettesClient;
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let http_client = Client::new();
    let client = BlockPalettesClient::new(http_client);

    match client.search_blocks("stone").await {
        Ok(blocks) => println!("Found blocks: {:?}", blocks),
        Err(e) => eprintln!("Error searching blocks: {}", e),
    }

    Ok(())
}
```

**Note**: This is a minimal example. Please refer to the [docs.rs](https://docs.rs/blockpalettes-client) documentation for the full API and more detailed usage.

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License

This project is licensed under dual-licensed under the MIT License and the Apache 2.0 license, either at your choice. See the [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) file for details.

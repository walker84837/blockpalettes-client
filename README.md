# blockpalettes-client

[![crates.io version](https://img.shields.io/crates/v/blockpalettes-client.svg)](https://crates.io/crates/blockpalettes-client)
[![docs.rs documentation](https://img.shields.io/docsrs/blockpalettes-client)](https://docs.rs/blockpalettes-client)
[![crates.io license](https://img.shields.io/crates/l/blockpalettes-client.svg)](https://crates.io/crates/blockpalettes-client)
[![crates.io downloads](https://img.shields.io/crates/d/blockpalettes-client.svg)](https://crates.io/crates/blockpalettes-client)
[![dependency status](https://deps.rs/crate/blockpalettes-client/latest/status.svg)](https://deps.rs/crate/blockpalettes-client)

Unofficial Rust client for [BlockPalettes](https://blockpalettes.com) with built-in HTTP and scraping support.

## Why `blockpalettes-client` Exists

<https://blockpalettes.com> is great for exploring Minecraft block color palettes, but its search interface is limited:
- You can only search for one block at a time;
- Filtering by multiple blocks or automating repeated searches is not supported;
- Some data, like similar palettes or full block lists, is only accessible via the HTML pages, not the API;

This library solves these issues by:
- Allowing multi-block searches for palettes;
- Providing convenient methods to fetch popular blocks and palette details;
- Supporting similar palette retrieval;
- Scraping additional data directly from the website when necessary.

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

### Disclaimers

Because there isn't proper documentation for the API:

- If it changes its APIs, internal structure, or adds CAPTCHA verification, parts of this library may stop working (like HTML scraping). Make an issue if this happens.
- I'm not sure if the website has ANY sort of rate limiting. Please do not make requests too frequently to avoid overloading the website.

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License

This project is licensed under dual-licensed under the MIT License and the Apache 2.0 license, either at your choice. See the [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) file for details.

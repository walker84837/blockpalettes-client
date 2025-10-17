//! A client for the Block Palettes API.
//!
//! This crate provides a convenient asynchronous client for interacting with the
//! [Block Palettes](https://www.blockpalettes.com) website's API. It allows you to
//! search for palettes, retrieve popular blocks, get detailed information about
//! specific palettes, and even scrape some information directly from the website's
//! HTML pages.
//!
//! The client is built on top of `reqwest` for HTTP requests and `serde` for
//! JSON serialization/deserialization. It also uses `chrono` for date parsing
//! and `scraper` for HTML parsing when scraping.
//!
//! # Features
//!
//! - Search for palettes based on contained blocks.
//! - Retrieve lists of popular blocks.
//! - Fetch detailed information for individual palettes.
//! - Get similar palettes based on a given palette ID.
//! - Scrape palette page details (blocks and similar palette IDs) directly from HTML.
//! - Robust error handling with custom error types.
//!
//! # Error Handling
//!
//! The crate defines a custom error type, [`BlockPalettesError`], which
//! encapsulates various issues that can occur, such as HTTP errors, API-specific
//! errors, HTML parsing failures, and invalid date formats.
//!
//! # Data Structures
//!
//! Key data structures like [`Palette`], [`PaletteDetails`], and [`PopularBlock`]
//! are provided to represent the API responses.

use chrono::NaiveDateTime;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

/// Represents the possible errors that can occur when interacting with the
/// Block Palettes API.
#[derive(Debug, Error)]
pub enum BlockPalettesError {
    /// An HTTP request failed, typically due to network issues, DNS resolution,
    /// or invalid URLs.
    ///
    /// This error wraps the underlying `reqwest::Error`.
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    /// The Block Palettes API returned an error message or indicated a failure
    /// in its response.
    ///
    /// The contained `String` provides more details about the API-specific error.
    #[error("API error: {0}")]
    Api(String),
    /// An error occurred during the parsing of HTML content, typically when
    /// scraping a palette page.
    ///
    /// This can happen if the HTML structure changes unexpectedly.
    #[error("HTML parsing error")]
    HtmlParse,
    /// The date string received from the API could not be parsed into a
    /// `NaiveDateTime` object.
    ///
    /// This usually indicates an unexpected date format from the API.
    #[error("Invalid date format")]
    InvalidDateFormat,
}

/// A specialized `Result` type for Block Palettes operations.
///
/// This type is a convenience alias for `std::result::Result<T, BlockPalettesError>`.
pub type Result<T, E = BlockPalettesError> = std::result::Result<T, E>;

/// An asynchronous client for the Block Palettes API.
///
/// This struct provides methods to interact with various endpoints of the
/// Block Palettes API, allowing you to search for palettes, retrieve block
/// information, and get palette details.
///
/// # Examples
///
/// ```rust,no_run
/// use blockpalettes_client::{BlockPalettesClient, SortOrder};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = BlockPalettesClient::new(reqwest::Client::new());
///     let popular_blocks = client.popular_blocks().await?;
///     println!("First popular block: {}", popular_blocks[0].name);
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct BlockPalettesClient<'a> {
    client: Client,
    base_url: &'a str,
}

impl<'a> BlockPalettesClient<'a> {
    /// Creates a new [`BlockPalettesClient`] instance.
    ///
    /// # Arguments
    ///
    /// * `client` - An instance of `reqwest::Client` to use for making HTTP requests.
    ///
    /// # Returns
    ///
    /// A new `BlockPalettesClient` configured to use the provided `reqwest::Client`
    /// and the default base URL (`https://www.blockpalettes.com`).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use blockpalettes_client::BlockPalettesClient;
    ///
    /// let reqwest_client = reqwest::Client::new();
    /// let bp_client = BlockPalettesClient::new(reqwest_client);
    /// ```
    pub const fn new(client: Client) -> Self {
        Self {
            client,
            base_url: "https://www.blockpalettes.com",
        }
    }

    /// Searches for blocks that match a given query string.
    ///
    /// This method queries the `/api/palettes/search-block.php` endpoint.
    ///
    /// # Arguments
    ///
    /// * `query` - The search string for blocks (e.g., "stone", "wood").
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Vec<String>` of block names if successful,
    /// or a [`BlockPalettesError`] if the request fails or the API returns an error.
    ///
    /// [`BlockPalettesError`]: enum.BlockPalettesError.html
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use blockpalettes_client::BlockPalettesClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = BlockPalettesClient::new(reqwest::Client::new());
    ///     let blocks = client.search_blocks("dirt").await?;
    ///     println!("Found blocks: {:?}", blocks);
    ///     Ok(())
    /// }
    /// ```
    pub async fn search_blocks(&self, query: impl AsRef<str>) -> Result<Vec<String>> {
        let url = format!("{}/api/palettes/search-block.php", self.base_url);
        let response = self
            .client
            .get(&url)
            .query(&[("query", query.as_ref())])
            .send()
            .await?
            .json::<BlockSearchResponse>()
            .await?;

        if response.success {
            Ok(response.blocks)
        } else {
            Err(BlockPalettesError::Api("Search failed".into()))
        }
    }

    /// Retrieves a list of popular blocks.
    ///
    /// This method queries the `/api/palettes/popular-blocks.php` endpoint.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Vec<PopularBlock>` if successful,
    /// or a [`BlockPalettesError`] if the request fails or the API returns an error.
    ///
    /// [`PopularBlock`]: struct.PopularBlock.html
    /// [`BlockPalettesError`]: enum.BlockPalettesError.html
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use blockpalettes_client::BlockPalettesClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = BlockPalettesClient::new(reqwest::Client::new());
    ///     let popular_blocks = client.popular_blocks().await?;
    ///     for block in popular_blocks.iter().take(3) {
    ///         println!("Block: {}, Count: {}", block.name, block.count);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn popular_blocks(&self) -> Result<Vec<PopularBlock>> {
        let url = format!("{}/api/palettes/popular-blocks.php", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<PopularBlocksResponse>()
            .await?;

        if response.success {
            Ok(response.blocks)
        } else {
            Err(BlockPalettesError::Api(
                "Popular blocks request failed".into(),
            ))
        }
    }

    /// Retrieves a list of palettes based on specified blocks, sorting order,
    /// pagination, and limit.
    ///
    /// This method queries the `/api/palettes/all_palettes.php` endpoint.
    /// It internally filters the results to ensure all specified blocks are present
    /// in the returned palettes.
    ///
    /// # Arguments
    ///
    /// * `blocks` - A slice of string references representing the blocks that
    ///   must be present in the palettes.
    /// * `sort` - The desired sorting order for the palettes (e.g., `SortOrder::Recent`).
    /// * `page` - The page number of the results to retrieve (1-indexed).
    /// * `limit` - The maximum number of palettes to return per page.
    ///
    /// # Returns
    ///
    /// A `Result` containing a [`PaletteResponse`] if successful,
    /// or a [`BlockPalettesError`] if the request fails or the API returns an error.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use blockpalettes_client::{BlockPalettesClient, SortOrder};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = BlockPalettesClient::new(reqwest::Client::new());
    ///     let blocks_to_search = &["oak_log", "dirt"];
    ///     let response = client
    ///         .get_palettes(blocks_to_search, SortOrder::Popular, 1, 5)
    ///         .await?;
    ///
    ///     if let Some(palettes) = response.palettes {
    ///         println!("Found {} popular palettes containing oak_log and dirt:", palettes.len());
    ///         for palette in palettes {
    ///             println!("- ID: {}, Name: {:?}", palette.id, palette.name());
    ///         }
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_palettes(
        &self,
        blocks: &[&str],
        sort: SortOrder,
        page: u32,
        limit: u32,
    ) -> Result<PaletteResponse> {
        let url = format!("{}/api/palettes/all_palettes.php", self.base_url);

        let mut all_palettes = Vec::new();
        let mut total_results = 0;
        let mut total_pages = 0;

        for &block in blocks {
            let response = self
                .client
                .get(&url)
                .query(&[
                    ("sort", sort.to_string()),
                    ("page", page.to_string()),
                    ("limit", limit.to_string()),
                    ("blocks", block.to_string()),
                ])
                .send()
                .await?
                .json::<PaletteResponse>()
                .await?;

            if total_results == 0 {
                total_results = response.total_results;
                total_pages = response.total_pages;
            }

            if let Some(mut ps) = response.palettes {
                all_palettes.append(&mut ps);
            }
        }

        // filter the collected palettes to ensure they contain ALL specified blocks
        let filtered = all_palettes
            .into_iter()
            .filter(|p| p.contains_all_blocks(blocks))
            .collect();

        Ok(PaletteResponse {
            success: true,
            palettes: Some(filtered),
            total_results,
            total_pages,
        })
    }

    /// Retrieves detailed information for a single palette by its ID.
    ///
    /// This method queries the `/api/palettes/single_palette.php` endpoint.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the palette.
    ///
    /// # Returns
    ///
    /// A `Result` containing a [`PaletteDetails`] if successful,
    /// or a [`BlockPalettesError`] if the request fails, the API returns an error
    /// (e.g., palette not found), or the response cannot be deserialized.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use blockpalettes_client::BlockPalettesClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = BlockPalettesClient::new(reqwest::Client::new());
    ///     let palette_id = 12345; // Replace with an actual palette ID
    ///     match client.get_palette_details(palette_id).await {
    ///         Ok(details) => println!("Palette ID {}: Username {}", details.id, details.username),
    ///         Err(e) => eprintln!("Failed to get palette details: {}", e),
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_palette_details(&self, id: u64) -> Result<PaletteDetails> {
        let url = format!("{}/api/palettes/single_palette.php", self.base_url);
        let response = self
            .client
            .get(&url)
            .query(&[("id", id.to_string())])
            .send()
            .await?
            .json::<SinglePaletteResponse>()
            .await?;

        if response.success {
            Ok(response.palette)
        } else {
            Err(BlockPalettesError::Api("Palette not found".into()))
        }
    }

    /// Retrieves a list of palettes similar to a given palette ID.
    ///
    /// This method queries the `/api/palettes/similar_palettes.php` endpoint.
    ///
    /// # Arguments
    ///
    /// * `palette_id` - The ID of the reference palette to find similar ones.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Vec<Palette>` of similar palettes if successful,
    /// or a [`BlockPalettesError`] if the request fails, the API returns an error,
    /// or the response cannot be deserialized.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use blockpalettes_client::BlockPalettesClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = BlockPalettesClient::new(reqwest::Client::new());
    ///     let reference_palette_id = 12345; // Replace with an actual palette ID
    ///     let similar_palettes = client.get_similar_palettes(reference_palette_id).await?;
    ///     println!("Found {} similar palettes for ID {}:", similar_palettes.len(), reference_palette_id);
    ///     for palette in similar_palettes.iter().take(3) {
    ///         println!("- ID: {}, Name: {:?}", palette.id, palette.name());
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_similar_palettes(&self, palette_id: u64) -> Result<Vec<Palette>> {
        let url = format!("{}/api/palettes/similar_palettes.php", self.base_url);
        let response = self
            .client
            .get(&url)
            .query(&[("palette_id", palette_id.to_string())])
            .send()
            .await?
            .json::<SimilarPalettesResponse>()
            .await?;

        if response.success {
            Ok(response.palettes)
        } else {
            Err(BlockPalettesError::Api("Similar palettes not found".into()))
        }
    }

    /// Scrapes details directly from a Block Palettes HTML page for a given palette ID.
    ///
    /// This method is useful for extracting information that might not be available
    /// directly through the public API endpoints, such as the full list of blocks
    /// displayed on the page or IDs of similar palettes linked on the page.
    ///
    /// # Arguments
    ///
    /// * `palette_id` - The ID of the palette whose page details are to be scraped.
    ///
    /// # Returns
    ///
    /// A `Result` containing a [`PalettePageDetails`] if successful,
    /// or a [`BlockPalettesError`] if the request fails, HTML parsing fails,
    /// or expected elements are not found.
    ///
    /// # Caveats
    ///
    /// This method relies on the specific HTML structure of `blockpalettes.com`.
    /// Any changes to the website's front-end might break this scraping functionality.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use blockpalettes_client::BlockPalettesClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = BlockPalettesClient::new(reqwest::Client::new());
    ///     let palette_id = 12345; // Replace with an actual palette ID
    ///     match client.scrape_palette_page(palette_id).await {
    ///         Ok(details) => {
    ///             println!("Scraped blocks for palette {}: {:?}", palette_id, details.blocks);
    ///             println!("Similar palette IDs: {:?}", details.similar_palette_ids);
    ///         },
    ///         Err(e) => eprintln!("Failed to scrape palette page: {}", e),
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn scrape_palette_page(&self, palette_id: u64) -> Result<PalettePageDetails> {
        let url = format!("{}/palette/{}", self.base_url, palette_id);
        let html = self.client.get(&url).send().await?.text().await?;

        let document = Html::parse_document(&html);

        // extract palette blocks
        let block_selector =
            Selector::parse(".single-block").map_err(|_| BlockPalettesError::HtmlParse)?;
        let mut blocks = Vec::new();

        for element in document.select(&block_selector) {
            if let Some(block_name) = element.text().last() {
                blocks.push(block_name.trim().to_string());
            }
        }

        // extract similar palettes if available
        let similar_selector =
            Selector::parse(".palette-card").map_err(|_| BlockPalettesError::HtmlParse)?;
        let mut similar = Vec::new();

        for element in document.select(&similar_selector) {
            if let Some(id) = element
                .value()
                .attr("href")
                .and_then(|href| href.split('/').next_back())
                .and_then(|id| id.parse::<u64>().ok())
            {
                similar.push(id);
            }
        }

        Ok(PalettePageDetails {
            blocks,
            similar_palette_ids: similar,
        })
    }
}

/// Represents the different sorting orders available for retrieving palettes.
///
/// These variants correspond to the `sort` parameter in the Block Palettes API.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortOrder {
    /// Sort by the most recently added palettes.
    Recent,
    /// Sort by the most popular palettes.
    Popular,
    /// Sort by the oldest palettes.
    Oldest,
    /// Sort by trending palettes.
    Trending,
}

impl std::fmt::Display for SortOrder {
    /// Converts the `SortOrder` enum variant into its corresponding API string representation.
    ///
    /// # Returns
    ///
    /// A `String` slice representing the sort order (e.g., "recent", "popular").
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortOrder::Recent => write!(f, "recent"),
            SortOrder::Popular => write!(f, "popular"),
            SortOrder::Oldest => write!(f, "oldest"),
            SortOrder::Trending => write!(f, "trending"),
        }
    }
}

/// Internal struct for deserializing the response from the `/api/palettes/search-block.php` endpoint.
#[derive(Debug, Deserialize)]
struct BlockSearchResponse {
    success: bool,
    blocks: Vec<String>,
}

/// Internal struct for deserializing the response from the `/api/palettes/popular-blocks.php` endpoint.
#[derive(Debug, Deserialize)]
struct PopularBlocksResponse {
    success: bool,
    blocks: Vec<PopularBlock>,
}

/// Internal struct for deserializing the response from the `/api/palettes/single_palette.php` endpoint.
#[derive(Debug, Deserialize)]
struct SinglePaletteResponse {
    success: bool,
    palette: PaletteDetails,
}

/// Internal struct for deserializing the response from the `/api/palettes/similar_palettes.php` endpoint.
#[derive(Debug, Deserialize)]
struct SimilarPalettesResponse {
    success: bool,
    palettes: Vec<Palette>,
}

/// Represents a popular block returned by the API.
#[derive(Debug, Deserialize, Serialize)]
pub struct PopularBlock {
    /// The name of the block (e.g., "stone", "dirt").
    #[serde(rename = "block")]
    pub name: String,
    /// The number of palettes this block appears in.
    pub count: u32,
}

/// Represents the response structure when fetching a list of palettes.
#[derive(Debug, Deserialize, Serialize)]
pub struct PaletteResponse {
    /// Indicates if the API request was successful.
    pub success: bool,
    /// The total number of results found for the query.
    pub total_results: u32,
    /// The total number of pages available for the query.
    pub total_pages: u32,
    /// An optional vector of [`Palette`] objects. It will be `None` if no palettes were found.
    pub palettes: Option<Vec<Palette>>,
}

/// Represents a single palette returned by the Block Palettes API.
///
/// This struct contains core information about a palette, including its ID,
/// associated blocks, likes, and creation date.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Palette {
    /// The unique identifier for the palette.
    pub id: u64,
    /// The ID of the user who created the palette.
    pub user_id: u64,
    /// The creation date of the palette as a string (e.g., "YYYY-MM-DD HH:MM:SS").
    pub date: String,
    /// The number of likes the palette has received.
    pub likes: u32,
    /// The first block in the palette.
    #[serde(rename = "blockOne")]
    pub block_one: String,
    /// The second block in the palette.
    #[serde(rename = "blockTwo")]
    pub block_two: String,
    /// The third block in the palette.
    #[serde(rename = "blockThree")]
    pub block_three: String,
    /// The fourth block in the palette.
    #[serde(rename = "blockFour")]
    pub block_four: String,
    /// The fifth block in the palette.
    #[serde(rename = "blockFive")]
    pub block_five: String,
    /// The sixth block in the palette.
    #[serde(rename = "blockSix")]
    pub block_six: String,
    /// A flag indicating if the palette is hidden (0 for not hidden, 1 for hidden).
    pub hidden: u8,
    /// A flag indicating if the palette is featured (0 for not featured, 1 for featured).
    pub featured: u8,
    /// An optional hash associated with the palette.
    pub hash: Option<String>,
    /// A human-readable string indicating how long ago the palette was created (e.g., "2 days ago").
    pub time_ago: String,
}

impl Palette {
    /// Returns a vector containing all six block names from the palette.
    ///
    /// This is a convenience method to access all blocks without individually
    /// referencing `block_one`, `block_two`, etc.
    ///
    /// # Returns
    ///
    /// A `Vec<String>` containing the names of the six blocks in the palette.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use blockpalettes_client::Palette;
    /// # let palette = Palette {
    /// #    id: 1, user_id: 1, date: "2023-01-01 12:00:00".to_string(), likes: 10,
    /// #    block_one: "stone".to_string(), block_two: "dirt".to_string(),
    /// #    block_three: "grass_block".to_string(), block_four: "oak_log".to_string(),
    /// #    block_five: "cobblestone".to_string(), block_six: "sand".to_string(),
    /// #    hidden: 0, featured: 0, hash: None, time_ago: "1 day ago".to_string()
    /// # };
    /// let blocks = palette.name();
    /// assert_eq!(blocks.len(), 6);
    /// assert!(blocks.contains(&"stone".to_string()));
    /// ```
    pub fn name(&self) -> Vec<String> {
        vec![
            self.block_one.clone(),
            self.block_two.clone(),
            self.block_three.clone(),
            self.block_four.clone(),
            self.block_five.clone(),
            self.block_six.clone(),
        ]
    }

    /// Checks if the palette contains all the specified blocks.
    ///
    /// This method is useful for client-side filtering of palettes.
    ///
    /// # Arguments
    ///
    /// * `blocks` - A slice of string references, where each string is a block name
    ///   to check for.
    ///
    /// # Returns
    ///
    /// `true` if the palette contains all blocks specified in the `blocks` slice,
    /// `false` otherwise. The comparison is case-sensitive.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use blockpalettes_client::Palette;
    /// # let palette = Palette {
    /// #    id: 1, user_id: 1, date: "2023-01-01 12:00:00".to_string(), likes: 10,
    /// #    block_one: "stone".to_string(), block_two: "dirt".to_string(),
    /// #    block_three: "grass_block".to_string(), block_four: "oak_log".to_string(),
    /// #    block_five: "cobblestone".to_string(), block_six: "sand".to_string(),
    /// #    hidden: 0, featured: 0, hash: None, time_ago: "1 day ago".to_string()
    /// # };
    /// assert!(palette.contains_all_blocks(&["stone", "dirt"]));
    /// assert!(!palette.contains_all_blocks(&["stone", "diamond_block"]));
    /// ```
    pub fn contains_all_blocks(&self, blocks: &[&str]) -> bool {
        let palette_blocks: HashSet<&str> = HashSet::from([
            self.block_one.as_str(),
            self.block_two.as_str(),
            self.block_three.as_str(),
            self.block_four.as_str(),
            self.block_five.as_str(),
            self.block_six.as_str(),
        ]);

        blocks.iter().all(|&b| palette_blocks.contains(b))
    }

    /// Parses the `date` string of the palette into a `NaiveDateTime` object.
    ///
    /// This provides a more structured way to work with the palette's creation date.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `NaiveDateTime` if the date string is successfully parsed,
    /// or a [`BlockPalettesError::InvalidDateFormat`] if the string does not match
    /// the expected format ("YYYY-MM-DD HH:MM:SS").
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use blockpalettes_client::Palette;
    /// # use chrono::{NaiveDate, Timelike};
    /// # let palette = Palette {
    /// #    id: 1, user_id: 1, date: "2023-01-01 12:30:00".to_string(), likes: 10,
    /// #    block_one: "stone".to_string(), block_two: "dirt".to_string(),
    /// #    block_three: "grass_block".to_string(), block_four: "oak_log".to_string(),
    /// #    block_five: "cobblestone".to_string(), block_six: "sand".to_string(),
    /// #    hidden: 0, featured: 0, hash: None, time_ago: "1 day ago".to_string()
    /// # };
    /// let datetime = palette.parse_date().unwrap();
    /// assert_eq!(datetime.date(), NaiveDate::from_ymd_opt(2023, 1, 1).unwrap());
    /// assert_eq!(datetime.hour(), 12);
    /// ```
    pub fn parse_date(&self) -> Result<NaiveDateTime> {
        NaiveDateTime::parse_from_str(&self.date, "%Y-%m-%d %H:%M:%S")
            .map_err(|_| BlockPalettesError::InvalidDateFormat)
    }
}

/// Represents detailed information for a single palette, including the username.
///
/// This struct is typically returned by the [`BlockPalettesClient::get_palette_details`] method.
/// It extends the basic [`Palette`] information with the `username` of the creator.
#[derive(Debug, Deserialize, Serialize)]
pub struct PaletteDetails {
    /// The unique identifier for the palette.
    pub id: u64,
    /// The ID of the user who created the palette.
    #[serde(rename = "user_id")]
    pub user_id: u64,
    /// The creation date of the palette as a string (e.g., "YYYY-MM-DD HH:MM:SS").
    pub date: String,
    /// The number of likes the palette has received.
    pub likes: u32,
    /// The first block in the palette.
    #[serde(rename = "blockOne")]
    pub block_one: String,
    /// The second block in the palette.
    #[serde(rename = "blockTwo")]
    pub block_two: String,
    /// The third block in the palette.
    #[serde(rename = "blockThree")]
    pub block_three: String,
    /// The fourth block in the palette.
    #[serde(rename = "blockFour")]
    pub block_four: String,
    /// The fifth block in the palette.
    #[serde(rename = "blockFive")]
    pub block_five: String,
    /// The sixth block in the palette.
    #[serde(rename = "blockSix")]
    pub block_six: String,
    /// A flag indicating if the palette is hidden (0 for not hidden, 1 for hidden).
    pub hidden: u8,
    /// A flag indicating if the palette is featured (0 for not featured, 1 for featured).
    pub featured: u8,
    /// The hash associated with the palette.
    pub hash: String,
    /// The username of the palette creator.
    pub username: String,
    /// A human-readable string indicating how long ago the palette was created (e.g., "2 days ago").
    #[serde(rename = "time_ago")]
    pub time_ago: String,
}

/// Represents details scraped directly from a palette's HTML page.
///
/// This struct is typically returned by the [`BlockPalettesClient::scrape_palette_page`] method.
/// It contains information extracted by parsing the HTML, which might include
/// blocks displayed on the page and IDs of similar palettes linked.
///
/// [`BlockPalettesClient::scrape_palette_page`]: struct.BlockPalettesClient.html#method.scrape_palette_page
#[derive(Debug, Serialize)]
pub struct PalettePageDetails {
    /// A list of block names found on the palette's page.
    pub blocks: Vec<String>,
    /// A list of IDs of similar palettes linked on the page.
    pub similar_palette_ids: Vec<u64>,
}

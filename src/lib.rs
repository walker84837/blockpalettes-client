use chrono::NaiveDateTime;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BlockPalettesError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("API error: {0}")]
    Api(String),
    #[error("HTML parsing error")]
    HtmlParse,
    #[error("Invalid date format")]
    InvalidDateFormat,
}

pub type Result<T, E = BlockPalettesError> = std::result::Result<T, E>;

#[derive(Debug, Clone)]
/// A client for the Block Palettes API.
pub struct BlockPalettesClient<'a> {
    client: Client,
    base_url: &'a str,
}

impl<'a> BlockPalettesClient<'a> {
    pub const fn new(client: Client) -> Self {
        Self {
            client,
            base_url: "https://www.blockpalettes.com",
        }
    }

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
                .and_then(|href| href.split('/').last())
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortOrder {
    Recent,
    Popular,
    Oldest,
    Trending,
}

impl ToString for SortOrder {
    fn to_string(&self) -> String {
        match self {
            SortOrder::Recent => "recent",
            SortOrder::Popular => "popular",
            SortOrder::Oldest => "oldest",
            SortOrder::Trending => "trending",
        }
        .to_string()
    }
}

#[derive(Debug, Deserialize)]
struct BlockSearchResponse {
    success: bool,
    blocks: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PopularBlocksResponse {
    success: bool,
    blocks: Vec<PopularBlock>,
}

#[derive(Debug, Deserialize)]
struct SinglePaletteResponse {
    success: bool,
    palette: PaletteDetails,
}

#[derive(Debug, Deserialize)]
struct SimilarPalettesResponse {
    success: bool,
    palettes: Vec<Palette>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PopularBlock {
    #[serde(rename = "block")]
    pub name: String,
    pub count: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PaletteResponse {
    pub success: bool,
    pub total_results: u32,
    pub total_pages: u32,
    pub palettes: Option<Vec<Palette>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Palette {
    pub id: u64,
    pub user_id: u64,
    pub date: String,
    pub likes: u32,
    #[serde(rename = "blockOne")]
    pub block_one: String,
    #[serde(rename = "blockTwo")]
    pub block_two: String,
    #[serde(rename = "blockThree")]
    pub block_three: String,
    #[serde(rename = "blockFour")]
    pub block_four: String,
    #[serde(rename = "blockFive")]
    pub block_five: String,
    #[serde(rename = "blockSix")]
    pub block_six: String,
    pub hidden: u8,
    pub featured: u8,
    pub hash: Option<String>,
    pub time_ago: String,
}

impl Palette {
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

    pub fn parse_date(&self) -> Result<NaiveDateTime, BlockPalettesError> {
        NaiveDateTime::parse_from_str(&self.date, "%Y-%m-%d %H:%M:%S")
            .map_err(|_| BlockPalettesError::InvalidDateFormat)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PaletteDetails {
    pub id: u64,
    #[serde(rename = "user_id")]
    pub user_id: u64,
    pub date: String,
    pub likes: u32,
    #[serde(rename = "blockOne")]
    pub block_one: String,
    #[serde(rename = "blockTwo")]
    pub block_two: String,
    #[serde(rename = "blockThree")]
    pub block_three: String,
    #[serde(rename = "blockFour")]
    pub block_four: String,
    #[serde(rename = "blockFive")]
    pub block_five: String,
    #[serde(rename = "blockSix")]
    pub block_six: String,
    pub hidden: u8,
    pub featured: u8,
    pub hash: String,
    pub username: String,
    #[serde(rename = "time_ago")]
    pub time_ago: String,
}

#[derive(Debug, Serialize)]
pub struct PalettePageDetails {
    pub blocks: Vec<String>,
    pub similar_palette_ids: Vec<u64>,
}

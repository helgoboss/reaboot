use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Recipe {
    pub name: String,
    #[ts(optional)]
    pub website: Option<String>,
    #[ts(optional)]
    pub manufacturer: Option<String>,
    #[ts(optional)]
    pub logo: Option<String>,
    pub package_urls: Vec<String>,
}

pub async fn fetch_and_parse_recipe(url: &str) -> Option<Recipe> {
    let json = fetch_recipe_json(url).await.ok()??;
    serde_json::from_str(&json).ok()
}

async fn fetch_recipe_json(url: &str) -> anyhow::Result<Option<String>> {
    let response = reqwest::get(url).await?;
    if response.status() == StatusCode::NOT_FOUND {
        return Ok(None);
    }
    let response = response.error_for_status()?;
    Ok(Some(response.text().await?))
}

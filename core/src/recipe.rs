use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Recipe {
    pub name: String,
    pub website: Option<String>,
    pub manufacturer: Option<String>,
    pub logo: Option<String>,
    pub package_urls: Vec<String>,
}

pub async fn find_and_parse_public_recipe(id: &str) -> Option<Recipe> {
    let json = find_public_recipe_json(id).await.ok()??;
    serde_json::from_str(&json).ok()
}

async fn find_public_recipe_json(id: &str) -> anyhow::Result<Option<String>> {
    let url = format!(
        "https://raw.githubusercontent.com/helgoboss/reaboot-recipes/main/recipes/{id}.json"
    );
    let response = reqwest::get(url).await?;
    if response.status() == StatusCode::NOT_FOUND {
        return Ok(None);
    }
    let response = response.error_for_status()?;
    Ok(Some(response.text().await?))
}

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Recipe {
    pub name: String,
    #[ts(optional)]
    pub description: Option<String>,
    #[ts(optional)]
    pub author: Option<String>,
    #[ts(optional)]
    pub website: Option<String>,
    #[ts(optional)]
    pub skip_additional_packages: Option<bool>,
    #[ts(optional)]
    pub required_packages: Option<Vec<String>>,
    #[ts(optional)]
    pub features: Option<BTreeMap<String, Feature>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Feature {
    pub name: String,
    #[ts(optional)]
    pub default: Option<bool>,
    #[ts(optional)]
    pub description: Option<String>,
    #[ts(optional)]
    pub packages: Option<Vec<String>>,
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

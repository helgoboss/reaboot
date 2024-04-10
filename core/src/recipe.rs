use reqwest::StatusCode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
pub struct Recipe {
    pub name: String,
    #[ts(optional = nullable)]
    pub description: Option<String>,
    #[ts(optional = nullable)]
    pub author: Option<String>,
    #[ts(optional = nullable)]
    pub website: Option<String>,
    #[ts(optional = nullable)]
    pub skip_additional_packages: Option<bool>,
    #[ts(optional = nullable)]
    pub required_packages: Option<Vec<String>>,
    #[ts(optional = nullable)]
    pub features: Option<BTreeMap<String, Feature>>,
}

impl Recipe {
    pub fn resolve_all_packages<'a>(
        &'a self,
        selected_features: &'a HashSet<String>,
    ) -> impl Iterator<Item = &String> {
        self.required_packages.iter().flatten().chain(
            self.features
                .iter()
                .flatten()
                .filter(|(id, _)| selected_features.contains(*id))
                .flat_map(|(_, feature)| feature.packages.iter().flatten()),
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
pub struct Feature {
    pub name: String,
    #[ts(optional = nullable)]
    pub default: Option<bool>,
    #[ts(optional = nullable)]
    pub description: Option<String>,
    #[ts(optional = nullable)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use schemars::schema_for;
    use std::fs;

    #[test]
    fn generate_json_schema() {
        let schema = schema_for!(Recipe);
        let text = serde_json::to_string_pretty(&schema).unwrap();
        fs::write("bindings/recipe.schema.json", text).unwrap();
    }
}

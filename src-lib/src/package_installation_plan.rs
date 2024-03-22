use crate::api::Recipe;
use anyhow::bail;

pub struct InstallationPlan {}

impl InstallationPlan {
    pub fn from_recipes(recipes: Vec<Recipe>) -> anyhow::Result<Self> {
        bail!("not implemented")
    }
}

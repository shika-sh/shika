use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File};

#[derive(Deserialize, Serialize)]
pub struct Language {
    pub name: String,
    pub types: HashMap<String, Vec<String>>,
}

pub fn get(path: &str) -> anyhow::Result<Language> {
    // TODO: Better error message in case of failure.
    let file = File::open(format!(".shika/languages/{}.yaml", path))?;
    serde_yaml::from_reader(file).map_err(|error| anyhow!(error))
}

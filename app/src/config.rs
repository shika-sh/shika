use std::fs::File;

use anyhow::anyhow;
use colored::Colorize;
use db::DatabaseKind;
use serde::{Deserialize, Serialize};

pub fn get() -> anyhow::Result<Config> {
    let content = File::open(".shika/config.yaml")
        .map_err(|_| anyhow!(
            "Could not open config. Are you sure shika has been properly initialized?\n\nTry to run\n    {}",
            "shika init".italic()
        ))?;

    let config: DeserializedConfig = serde_yaml::from_reader(content)
        .map_err(|error| anyhow!("Could not parse config: {}", error))?;

    Ok(Config {
        database: DatabaseConfig {
            kind: config.database.kind,
            exclude_tables: config.database.exclude_tables.unwrap_or(Vec::new()),
        },
        templates: config.templates,
    })
}

#[derive(Serialize, Debug)]
pub struct Config {
    pub database: DatabaseConfig,
    pub templates: Vec<TemplateConfig>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DatabaseConfig {
    #[serde(rename = "type")]
    pub kind: DatabaseKind,
    pub exclude_tables: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TemplateConfig {
    pub name: String,
    pub input: String,
    pub output_dir: String,
    pub output: String,
    #[serde(default)]
    pub single: bool,
    pub language: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DeserializedConfig {
    pub database: DeserializedDatabaseConfig,
    pub templates: Vec<TemplateConfig>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DeserializedDatabaseConfig {
    #[serde(rename = "type")]
    pub kind: DatabaseKind,
    pub exclude_tables: Option<Vec<String>>,
}

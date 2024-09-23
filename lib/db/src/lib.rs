use serde::{Deserialize, Serialize};

pub mod mysql;
pub mod postgres;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum DatabaseKind {
    #[serde(rename = "postgres")]
    Postgres,
    #[serde(rename = "mysql")]
    MySql,
    #[serde(rename = "maria")]
    Maria,
    #[serde(rename = "sqlite")]
    SqLite,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DatabaseInformation {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "type")]
    pub kind: DatabaseKind,
    pub tables: Vec<TableInformation>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TableInformation {
    pub name: String,
    pub columns: Vec<ColumnInformation>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ColumnInformation {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub optional: bool,
    pub is_primary_key: bool,
    pub is_foreign_key: bool,
}

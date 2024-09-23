use anyhow::{anyhow, Ok};
use itertools::Itertools;
use sqlx::{query_as, Pool, Postgres};

use crate::{ColumnInformation, DatabaseInformation, DatabaseKind, TableInformation};

pub async fn connect(url: &str) -> anyhow::Result<Pool<Postgres>> {
    Pool::<Postgres>::connect(url)
        .await
        .map_err(|error| anyhow!(error))
}

#[derive(sqlx::FromRow, Debug)]
struct RawColumnInformation {
    table_name: String,
    column_name: String,
    data_type: String,
    is_nullable: String,
    constraint_type: Option<String>,
}

pub async fn get(pool: &Pool<Postgres>) -> anyhow::Result<DatabaseInformation> {
    let columns = query_as::<_, RawColumnInformation>(
        r#"
        SELECT
            T.table_name,
            T.column_name,
            T.data_type,
            T.is_nullable,
            C.constraint_type
        FROM information_schema.columns AS T
        LEFT JOIN information_schema.key_column_usage AS K
            ON T.column_name = K.column_name
                AND T.table_name = K.table_name
        LEFT JOIN information_schema.table_constraints AS C
            ON C.table_name = K.table_name
                AND C.constraint_catalog = K.constraint_catalog
                AND C.constraint_schema = K.constraint_schema
                AND C.constraint_name = K.constraint_name
        WHERE
            T.table_schema = $1
            AND T.table_name NOT LIKE '\_%'
        ORDER BY T.table_name, T.ordinal_position
        "#,
    )
    .bind("public")
    .fetch_all(pool)
    .await?;

    let mut tables: Vec<TableInformation> = vec![];
    for (key, column_chunks) in &columns.iter().chunk_by(|column| &column.table_name) {
        tables.push(TableInformation {
            name: key.clone(),
            columns: column_chunks
                .map(|column| ColumnInformation {
                    name: column.column_name.clone(),
                    kind: column.data_type.clone(),
                    optional: column.is_nullable == "YES",
                    is_primary_key: column
                        .constraint_type
                        .clone()
                        .is_some_and(|val| val == "PRIMARY KEY"),
                    is_foreign_key: column
                        .constraint_type
                        .clone()
                        .is_some_and(|val| val == "FOREIGN KEY"),
                })
                .collect(),
        });
    }

    Ok(DatabaseInformation {
        name: "public".to_string(),
        kind: DatabaseKind::Postgres,
        tables,
    })
}

use crate::{config, language};
use anyhow::{anyhow, Error};
use colored::Colorize;
use db::{ColumnInformation, DatabaseInformation, DatabaseKind, TableInformation};
use dotenvy::dotenv;
use indicatif::ProgressStyle;
use std::{
    collections::HashMap,
    env,
    fs::{create_dir, create_dir_all, File},
    io::Write,
    time::Duration,
};
use tokio::runtime::Runtime;

pub fn init() -> anyhow::Result<()> {
    let mut has_diff = false;
    if create_dir(".shika").is_ok() {
        println!("{} Successfully created directory .shika", "✓".purple())
    } else {
        has_diff = true
    }

    if create_dir(".shika/templates").is_ok() {
        println!(
            "{} Successfully created directory .shika/templates",
            "✓".purple()
        );
    } else {
        has_diff = true
    }

    if create_dir(".shika/languages").is_ok() {
        println!(
            "{} Successfully created directory .shika/languages",
            "✓".purple()
        )
    } else {
        has_diff = true
    }

    if let Ok(file) = File::create_new(".shika/config.yaml") {
        println!(
            "{} Successfully created file .shika/config.yaml",
            "✓".purple()
        );

        let config = config::DeserializedConfig {
            database: config::DeserializedDatabaseConfig {
                kind: DatabaseKind::Postgres,
                env_key: None,
            },
            templates: vec![config::TemplateConfig {
                name: String::from("My Example Template"),
                input: String::from("example"),
                single: true,
                output_dir: String::from("generated"),
                output: String::from("example.rs"),
                language: String::from("rust"),
            }],
        };

        serde_yaml::to_writer(file, &config)?;

        if let Ok(file) = File::create_new(".shika/languages/rust.yaml") {
            let language = language::Language {
                name: String::from("Rust"),
                types: HashMap::from([
                    (String::from("String"), vec![String::from("text")]),
                    (String::from("i32"), vec![String::from("number")]),
                    (String::from("bool"), vec![String::from("boolean")]),
                ]),
            };

            serde_yaml::to_writer(file, &language)?;
        }

        File::create_new(".shika/templates/example.shika.tera").ok();
    } else {
        has_diff = true
    }

    if has_diff {
        println!("{} Shika is already setup for this project", "✓".purple());
    }

    Ok(())
}

pub fn pull() -> anyhow::Result<()> {
    dotenv().ok();
    let config: config::Config = config::get()?;
    let db_url = env::var(&config.database.env_key)?;

    let database: db::DatabaseInformation = Runtime::new()
        .unwrap()
        .block_on(async {
            let spinner =
                indicatif::ProgressBar::new_spinner().with_message("Querying database...");

            spinner.set_style(ProgressStyle::with_template("{spinner} {msg} {elapsed}").unwrap());

            let database = match config.database.kind {
                DatabaseKind::Postgres => {
                    println!("{} Using PostgreSQL", "✓".purple());
                    spinner.enable_steady_tick(Duration::from_millis(120));
                    let _ = db::postgres::get(&db::postgres::connect(&db_url).await?).await;
                    db::postgres::get(&db::postgres::connect(&db_url).await?).await
                }
                DatabaseKind::MySql => {
                    println!("{} Using MySQL", "✓".purple());
                    spinner.enable_steady_tick(Duration::from_millis(120));
                    db::mysql::get(&db::mysql::connect(&db_url).await?).await
                }
                DatabaseKind::Maria => todo!(),
                DatabaseKind::SqLite => todo!(),
            }?;

            spinner.finish_and_clear();
            println!("{} Successfully fetched data", "✓".purple());

            Ok(database)
        })
        .map_err(|error: Error| anyhow!("Could not read from database: {}", error))?;

    let file = File::create(".shika/database.yaml")?;
    serde_yaml::to_writer(file, &database)?;

    println!("{} Successfully created database.yaml", "✓".purple());

    Ok(())
}

pub fn generate(should_pull: bool) -> anyhow::Result<()> {
    if should_pull {
        pull()?;
    }

    let mut count = 0;

    let config: config::Config = config::get()?;
    let file = File::open(".shika/database.yaml")?;
    let database: DatabaseInformation = serde_yaml::from_reader(file)?;

    let mut templates_iter = config.templates.iter();
    let tera = renderer::create()?;
    while let Some(template) = templates_iter.next() {
        create_dir_all(&template.output_dir)?;

        let language = language::get(&template.language)?;

        let mut database = database.clone();
        database.tables = database
            .tables
            .iter()
            .map(|table| TableInformation {
                name: table.name.clone(),
                columns: table
                    .columns
                    .iter()
                    .map(|column| ColumnInformation {
                        kind: language
                            .types
                            .iter()
                            .find(|(_, matches)| {
                                matches.iter().find(|t| t == &&column.kind).is_some()
                            })
                            .map_or(column.kind.clone(), |(key, _)| key.clone()),
                        ..column.clone()
                    })
                    .collect(),
            })
            .collect();

        if template.single {
            let content = renderer::render(&template.input, &tera, &database, None)?;
            println!(
                "{} Successfully rendered \"{}\" (target: {})",
                "✓".purple(),
                &template.name,
                &language.name.blue()
            );

            let target_path = format!("{}/{}", template.output_dir, template.output);
            File::create(target_path)?.write_all(content.as_bytes())?;
            count += 1;
        } else {
            let mut tables_iter = database.tables.iter();
            while let Some(table) = tables_iter.next() {
                let content = renderer::render(&template.input, &tera, &database, Some(table))?;
                println!(
                    "{} Successfully rendered \"{}\" for \"{}\" (target: {})",
                    "✓".purple(),
                    &template.name,
                    &table.name,
                    &language.name.blue()
                );

                let target_path = format!(
                    "{}/{}",
                    template.output_dir,
                    template.output.replace("{table}", &table.name)
                );

                File::create(target_path)?.write_all(content.as_bytes())?;
                count += 1;
            }
        }
    }

    println!(
        "{} Generated {} file{}",
        "✓".purple(),
        count,
        if count != 1 { "s" } else { "" }
    );

    Ok(())
}

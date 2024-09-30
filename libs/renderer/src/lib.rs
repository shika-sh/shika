mod filter;
mod model;

pub use model::{Column, Database, Table};

use tera::{Context, Tera};

pub fn render(
    path: &str,
    tera: &Tera,
    database: &db::Database,
    table: Option<&db::Table>,
) -> anyhow::Result<String> {
    let mut context = Context::new();
    context.insert("database", &database);

    if let Some(table) = table {
        context.insert("table", &table);
    }

    Ok(tera.render(&format!("{path}.shika.tera"), &context)?)
}

pub fn create() -> anyhow::Result<Tera> {
    let mut tera = Tera::new(".shika/templates/**/*.tera")?;

    tera.register_filter("exclude_keys", filter::exclude_keys);

    Ok(tera)
}

use std::collections::HashMap;

use tera::{Error, Result, Value};

pub fn exclude_keys(value: &Value, _args: &HashMap<String, Value>) -> Result<Value> {
    Ok(value
        .clone()
        .as_array()
        .ok_or(Error::msg(Value::String("Error".to_string())))?
        .clone()
        .into_iter()
        .filter(|item| !item["is_primary_key"].as_bool().unwrap_or(false))
        .collect())
}
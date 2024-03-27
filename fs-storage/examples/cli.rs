use anyhow::{Context, Result};
use fs_storage::file_storage::FileStorage;
use serde_json::Value;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage:");
        println!(" cargo run --example cli write <path> <json_file>");
        println!(" cargo run --example cli read <path> <key1,key2,...>");
        return Ok(());
    }

    let command = &args[1];
    let path = &args[2];
    match command.as_str() {
        "read" => read_command(&args, path),
        "write" => write_command(&args, path),
        _ => {
            eprintln!("Invalid command. Use 'read' or 'write'.");
            Ok(())
        }
    }
}

fn read_command(args: &[String], path: &str) -> Result<()> {
    let keys = if args.len() > 3 {
        args[3]
            .split(',')
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
    } else {
        vec![]
    };

    let mut fs = FileStorage::new("cli".to_string(), Path::new(path));
    let map: BTreeMap<String, String> =
        fs.read_file().context("Failed to read file")?;

    if keys.is_empty() {
        for (key, value) in map {
            println!("{}: {}", key, value);
        }
    } else {
        for key in &keys {
            if let Some(value) = map.get(key) {
                println!("{}: {}", key, value);
            } else {
                eprintln!("Key '{}' not found", key);
            }
        }
    }

    Ok(())
}

fn write_command(args: &[String], path: &str) -> Result<()> {
    if args.len() < 4 {
        println!("Usage: cargo run -- write <path> <json_file>");
        return Ok(());
    }

    let json_file = &args[3];
    let json_contents =
        fs::read_to_string(json_file).context("Failed to read JSON file")?;
    let json_value: Value =
        serde_json::from_str(&json_contents).context("Invalid JSON")?;

    let mut kv_pairs = BTreeMap::new();
    if let Value::Object(object) = json_value {
        for (key, value) in object {
            if let Value::String(value_str) = value {
                kv_pairs.insert(key, value_str);
            } else {
                println!(
                    "Warning: Skipping non-string value for key '{}'",
                    key
                );
            }
        }
    } else {
        println!("JSON value is not an object");
        return Ok(());
    }

    let mut fs = FileStorage::new("cli".to_string(), Path::new(path));
    fs.write_file(&kv_pairs)
        .context("Failed to write file")?;

    Ok(())
}

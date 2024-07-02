use crate::{
    error::AppError,
    models::{key_value_to_str, Format},
};
use data_error::Result as ArklibResult;
use fs_atomic_versions::atomic::{modify, modify_json, AtomicFile};

pub fn file_append(
    atomic_file: &AtomicFile,
    content: &str,
    format: Format,
) -> Result<(), AppError> {
    match format {
        Format::Raw => Ok(modify(atomic_file, |current| {
            let mut combined_vec: Vec<u8> = current.to_vec();
            combined_vec.extend_from_slice(content.as_bytes());
            combined_vec
        })?),
        Format::KeyValue => {
            let values = key_value_to_str(content)?;

            Ok(append_json(atomic_file, values.to_vec())?)
        }
    }
}

pub fn file_insert(
    atomic_file: &AtomicFile,
    content: &str,
    format: Format,
) -> Result<(), AppError> {
    match format {
        Format::Raw => {
            Ok(modify(atomic_file, |_| content.as_bytes().to_vec())?)
        }
        Format::KeyValue => {
            let values = key_value_to_str(content)?;

            modify_json(
                atomic_file,
                |current: &mut Option<serde_json::Value>| {
                    let mut new = serde_json::Map::new();
                    for (key, value) in &values {
                        new.insert(
                            key.clone(),
                            serde_json::Value::String(value.clone()),
                        );
                    }
                    *current = Some(serde_json::Value::Object(new));
                },
            )
            .map_err(|e| AppError::FileOperationError(e.to_string()))
        }
    }
}

fn append_json(
    atomic_file: &AtomicFile,
    data: Vec<(String, String)>,
) -> ArklibResult<()> {
    modify_json(atomic_file, |current: &mut Option<serde_json::Value>| {
        let current_data = match current {
            Some(current) => {
                if let Ok(value) = serde_json::to_value(current) {
                    match value {
                        serde_json::Value::Object(map) => Some(map),
                        _ => None,
                    }
                } else {
                    None
                }
            }

            None => None,
        };
        let mut new = serde_json::Map::new();

        if current_data.is_none() {
            for (key, value) in &data {
                new.insert(
                    key.clone(),
                    serde_json::Value::String(value.clone()),
                );
            }
            *current = Some(serde_json::Value::Object(new));
        } else if let Some(values) = current_data {
            for (key, value) in &values {
                new.insert(key.clone(), value.clone());
            }

            for (key, value) in &data {
                new.insert(
                    key.clone(),
                    serde_json::Value::String(value.clone()),
                );
            }
            *current = Some(serde_json::Value::Object(new));
        }
    })?;

    Ok(())
}

pub fn format_line<A, B, C, D>(
    version: A,
    name: B,
    machine: C,
    path: D,
) -> String
where
    A: std::fmt::Display,
    B: std::fmt::Display,
    C: std::fmt::Display,
    D: std::fmt::Display,
{
    format!("{: <8} {: <14} {: <36} {}", version, name, machine, path)
}

pub fn format_file(file: &AtomicFile) -> Option<String> {
    let current = file.load().ok()?;

    if current.version == 0 {
        return None;
    }

    let mut split = current
        .path
        .file_name()
        .expect("Not a file")
        .to_str()
        .unwrap()
        .split('_');

    let name = split.next().unwrap();

    let machine = split.next().unwrap();
    let machine = &machine[..machine.len() - 2];

    Some(format_line(
        current.version,
        name,
        machine,
        current.path.display(),
    ))
}

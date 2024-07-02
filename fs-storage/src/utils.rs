use data_error::Result;
use std::{collections::BTreeMap, path::Path};

/// Parses version 2 `FileStorage` format and returns the data as a BTreeMap
///
/// Version 2 `FileStorage` format represents data as a BTreeMap in plaintext.
///
/// For example:
/// ```text
/// version: 2
/// key1:1
/// key2:2
/// key3:3
/// ```
pub fn read_version_2_fs<K, V>(path: &Path) -> Result<BTreeMap<K, V>>
where
    K: Ord
        + Clone
        + serde::Serialize
        + serde::de::DeserializeOwned
        + std::str::FromStr,
    V: Clone
        + serde::Serialize
        + serde::de::DeserializeOwned
        + std::str::FromStr,
{
    // First check if the file starts with "version: 2"
    let file_content = std::fs::read_to_string(path)?;
    if !file_content.starts_with("version: 2") {
        return Err(data_error::ArklibError::Parse);
    }

    // Parse the file content into a BTreeMap
    let mut data = BTreeMap::new();
    for line in file_content.lines().skip(1) {
        let mut parts = line.split(':');
        let key = parts
            .next()
            .unwrap()
            .parse()
            .map_err(|_| data_error::ArklibError::Parse)?;
        let value = parts
            .next()
            .unwrap()
            .parse()
            .map_err(|_| data_error::ArklibError::Parse)?;

        data.insert(key, value);
    }

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempdir::TempDir;

    /// Test reading a legacy version 2 `FileStorage` file
    #[test]
    fn test_read_legacy_fs() {
        let temp_dir = TempDir::new("ark-rust").unwrap();
        let file_path = temp_dir.path().join("test_read_legacy_fs");
        let file_content = r#"version: 2
key1:1
key2:2
key3:3
"#;
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(file_content.as_bytes()).unwrap();

        // Read the file and check the data
        let data: BTreeMap<String, i32> =
            read_version_2_fs(&file_path).unwrap();
        assert_eq!(data.len(), 3);
        assert_eq!(data.get("key1"), Some(&1));
        assert_eq!(data.get("key2"), Some(&2));
        assert_eq!(data.get("key3"), Some(&3));
    }
}

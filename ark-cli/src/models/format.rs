use crate::error::InlineJsonParseError;

#[derive(Debug, Clone, Copy)]
pub enum Format {
    KeyValue,
    Raw,
}

impl std::str::FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Format::KeyValue),
            "raw" => Ok(Format::Raw),
            _ => Err("Invalid format".to_owned()),
        }
    }
}

pub fn key_value_to_str(
    s: &str,
) -> Result<Vec<(String, String)>, InlineJsonParseError> {
    let pairs: Vec<&str> = s.split(',').collect();

    let mut values = Vec::new();

    for pair in pairs {
        let key_value: Vec<&str> = pair.split(':').collect();
        if key_value.len() == 2 {
            let key = key_value[0].trim().to_string();
            let value = key_value[1].trim().to_string();
            values.push((key, value));
        } else {
            return Err(InlineJsonParseError::InvalidKeyValPair);
        }
    }

    Ok(values)
}

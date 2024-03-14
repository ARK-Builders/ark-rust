use clap::Parser;

#[derive(Parser, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryOutput {
    Link,
    Id,
    Path,
    Both,
}

impl std::str::FromStr for EntryOutput {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "id" => Ok(EntryOutput::Id),
            "path" => Ok(EntryOutput::Path),
            "both" => Ok(EntryOutput::Both),
            "link" => Ok(EntryOutput::Link),
            _ => Err("Entry output must be either 'id', 'path' or 'both'"),
        }
    }
}

use clap::Parser;

#[derive(Parser, Debug)]
pub enum Sort {
    Asc,
    Desc,
}

impl std::str::FromStr for Sort {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "asc" => Ok(Sort::Asc),
            "desc" => Ok(Sort::Desc),
            _ => Err("Sort must be either 'asc' or 'desc'"),
        }
    }
}

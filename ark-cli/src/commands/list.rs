use std::{io::Read, path::PathBuf};

use crate::{
    provide_index, provide_root, read_storage_value, AppError, DateTime,
    EntryOutput, File, Sort, StorageEntry, Utc,
};

#[derive(Clone, Debug, clap::Args)]
#[clap(name = "list", about = "List the resources in the ark managed folder")]
pub struct List {
    #[clap(value_parser, help = "The path to the root directory")]
    root_dir: Option<PathBuf>,
    #[clap(long, short = 'i', long = "id", action, help = "Show entries' IDs")]
    entry_id: bool,
    #[clap(
        long,
        short = 'p',
        long = "path",
        action,
        help = "Show entries' paths"
    )]
    entry_path: bool,
    #[clap(
        long,
        short = 'l',
        long = "link",
        action,
        help = "Show entries' links"
    )]
    entry_link: bool,
    #[clap(long, short, action, help = "Show entries' last modified times")]
    modified: bool,
    #[clap(long, short, action, help = "Show entries' tags")]
    tags: bool,
    #[clap(long, short, action, help = "Show entries' scores")]
    scores: bool,
    #[clap(long, value_enum, help = "Sort the entries by score")]
    sort: Option<Sort>,
    #[clap(long, help = "Filter the entries by tag")]
    filter: Option<String>,
}

impl List {
    /// Get the entry output format
    /// Default to Id
    pub fn entry(&self) -> Result<EntryOutput, AppError> {
        // Link can only be used alone
        if self.entry_link {
            if self.entry_id || self.entry_path {
                return Err(AppError::InvalidEntryOption)?;
            } else {
                return Ok(EntryOutput::Link);
            }
        }

        if self.entry_id && self.entry_path {
            Ok(EntryOutput::Both)
        } else if self.entry_path {
            Ok(EntryOutput::Path)
        } else {
            // Default to id
            Ok(EntryOutput::Id)
        }
    }

    pub fn run(&self) -> Result<(), AppError> {
        let root = provide_root(&self.root_dir)?;
        let entry_output = self.entry()?;

        let mut storage_entries: Vec<StorageEntry> = provide_index(&root)
            .map_err(|_| {
                AppError::IndexError("Could not provide index".to_owned())
            })?
            .read()
            .map_err(|_| {
                AppError::IndexError("Could not read index".to_owned())
            })?
            .resources()
            .iter()
            .filter_map(|indexed_resource| {
                let path = indexed_resource.path();
                let id = indexed_resource.id();
                let tags = if self.tags {
                    Some(
                        read_storage_value(
                            &root,
                            "tags",
                            &id.to_string(),
                            &None,
                        )
                        .map_or(vec![], |s| {
                            s.split(',')
                                .map(|s| s.trim().to_string())
                                .collect::<Vec<_>>()
                        }),
                    )
                } else {
                    None
                };

                let scores = if self.scores {
                    Some(
                        read_storage_value(
                            &root,
                            "scores",
                            &id.to_string(),
                            &None,
                        )
                        .map_or(0, |s| s.parse::<u32>().unwrap_or(0)),
                    )
                } else {
                    None
                };

                let datetime = if self.modified {
                    let format = "%b %e %H:%M %Y";
                    Some(
                        DateTime::<Utc>::from(indexed_resource.last_modified())
                            .format(format)
                            .to_string(),
                    )
                } else {
                    None
                };

                let (path, resource, content) = match entry_output {
                    EntryOutput::Both => {
                        (Some(path.to_owned()), Some(id.to_owned()), None)
                    }
                    EntryOutput::Path => (Some(path.to_owned()), None, None),
                    EntryOutput::Id => (None, Some(id.to_owned()), None),
                    EntryOutput::Link => match File::open(path) {
                        Ok(mut file) => {
                            let mut contents = String::new();
                            match file.read_to_string(&mut contents) {
                                Ok(_) => {
                                    // Check if the content of the file is a
                                    // valid url
                                    let url = contents.trim();
                                    let url = url::Url::parse(url);
                                    match url {
                                        Ok(url) => {
                                            (None, None, Some(url.to_string()))
                                        }
                                        Err(_) => return None,
                                    }
                                }
                                Err(_) => return None,
                            }
                        }
                        Err(_) => return None,
                    },
                };

                Some(StorageEntry {
                    path,
                    resource,
                    content,
                    tags,
                    scores,
                    datetime,
                })
            })
            .collect::<Vec<_>>();

        match self.sort {
            Some(Sort::Asc) => {
                storage_entries.sort_by(|a, b| a.datetime.cmp(&b.datetime))
            }

            Some(Sort::Desc) => {
                storage_entries.sort_by(|a, b| b.datetime.cmp(&a.datetime))
            }
            None => (),
        };

        if let Some(filter) = &self.filter {
            storage_entries.retain(|entry| {
                entry
                    .tags
                    .as_ref()
                    .map(|tags| tags.contains(filter))
                    .unwrap_or(false)
            });
        }

        let no_tags = "NO_TAGS";
        let no_scores = "NO_SCORE";

        let longest_path = storage_entries
            .iter()
            .map(|entry| {
                if let Some(path) = entry.path.as_ref() {
                    path.display().to_string().len()
                } else {
                    0
                }
            })
            .max_by(|a, b| a.cmp(b))
            .unwrap_or(0);

        let longest_id = storage_entries.iter().fold(0, |acc, entry| {
            if let Some(resource) = &entry.resource {
                let id_len = resource.to_string().len();
                if id_len > acc {
                    id_len
                } else {
                    acc
                }
            } else {
                acc
            }
        });

        let longest_tags = storage_entries.iter().fold(0, |acc, entry| {
            let tags_len = entry
                .tags
                .as_ref()
                .map(|tags| {
                    if tags.is_empty() {
                        no_tags.len()
                    } else {
                        tags.join(", ").len()
                    }
                })
                .unwrap_or(0);
            if tags_len > acc {
                tags_len
            } else {
                acc
            }
        });

        let longest_scores = storage_entries.iter().fold(0, |acc, entry| {
            let scores_len = entry
                .scores
                .as_ref()
                .map(|score| {
                    if *score == 0 {
                        no_scores.len()
                    } else {
                        score.to_string().len()
                    }
                })
                .unwrap_or(0);
            if scores_len > acc {
                scores_len
            } else {
                acc
            }
        });

        let longest_datetime = storage_entries.iter().fold(0, |acc, entry| {
            let datetime_len = entry
                .datetime
                .as_ref()
                .map(|datetime| datetime.len())
                .unwrap_or(0);
            if datetime_len > acc {
                datetime_len
            } else {
                acc
            }
        });

        let longest_content = storage_entries.iter().fold(0, |acc, entry| {
            let content_len = entry
                .content
                .as_ref()
                .map(|content| content.len())
                .unwrap_or(0);
            if content_len > acc {
                content_len
            } else {
                acc
            }
        });

        for entry in &storage_entries {
            let mut output = String::new();

            if let Some(content) = &entry.content {
                output.push_str(&format!(
                    "{:width$} ",
                    content,
                    width = longest_content
                ));
            }

            if let Some(path) = &entry.path {
                output.push_str(&format!(
                    "{:width$} ",
                    path.display(),
                    width = longest_path
                ));
            }

            if let Some(resource) = &entry.resource {
                output.push_str(&format!(
                    "{:width$} ",
                    resource.to_string(),
                    width = longest_id
                ));
            }

            if let Some(tags) = &entry.tags {
                let tags_out = if tags.is_empty() {
                    no_tags.to_owned()
                } else {
                    tags.join(", ")
                };

                output.push_str(&format!(
                    "{:width$} ",
                    tags_out,
                    width = longest_tags
                ));
            }

            if let Some(scores) = &entry.scores {
                let scores_out = if *scores == 0 {
                    no_scores.to_owned()
                } else {
                    scores.to_string()
                };

                output.push_str(&format!(
                    "{:width$} ",
                    scores_out,
                    width = longest_scores
                ));
            }

            if let Some(datetime) = &entry.datetime {
                output.push_str(&format!(
                    "{:width$} ",
                    datetime,
                    width = longest_datetime
                ));
            }

            println!("{}", output);
        }
        Ok(())
    }
}

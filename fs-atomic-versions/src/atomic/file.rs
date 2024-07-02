#[cfg(target_family = "unix")]
use std::os::unix::fs::MetadataExt;
use std::{
    fs::{self, File},
    io::{Error, ErrorKind, Read, Result},
    path::{Path, PathBuf},
};

use crate::app_id;

const MAX_VERSION_FILES: usize = 10;

pub struct TmpFile {
    file: File,
    path: PathBuf,
}

impl TmpFile {
    pub fn create_in(temp_dir: impl AsRef<Path>) -> Result<Self> {
        let filename: String = std::iter::repeat_with(fastrand::alphanumeric)
            .take(10)
            .collect();
        let path = temp_dir.as_ref().join(filename);
        let file = std::fs::File::create(&path)?;
        Ok(Self { file, path })
    }
}

impl std::io::Read for &TmpFile {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        (&self.file).read(buf)
    }
}

impl std::io::Write for &TmpFile {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        (&self.file).write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        (&self.file).flush()
    }
}

impl Drop for TmpFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

#[derive(Clone)]
pub struct ReadOnlyFile {
    pub version: usize,
    pub path: PathBuf,
}

/// This struct is the only way to read the file. Both path and version are
/// private
impl ReadOnlyFile {
    /// Open the underlying file, which can be read from but not written to.
    /// May return `Ok(None)`, which means that no version
    /// of the`AtomicFile` has been created yet.
    pub fn open(&self) -> Result<Option<File>> {
        if self.version != 0 {
            Ok(Some(File::open(&self.path)?))
        } else {
            Ok(None)
        }
    }

    pub fn read_to_string(&self) -> Result<String> {
        match self.open() {
            Ok(None) => Err(Error::new(ErrorKind::NotFound, "File not found")),
            Ok(Some(mut file)) => {
                let mut buff = String::new();
                file.read_to_string(&mut buff)?;
                Ok(buff)
            }
            Err(e) => Err(e),
        }
    }

    pub fn read_content(&self) -> Result<Vec<u8>> {
        match self.open() {
            Ok(None) => Err(Error::new(ErrorKind::NotFound, "File not found")),
            Err(e) => Err(e),
            Ok(Some(mut file)) => {
                let mut buf = vec![];
                file.read_to_end(&mut buf)?;
                Ok(buf)
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AtomicFile {
    pub directory: PathBuf,
    pub prefix: String,
}

fn parse_version(filename: Option<&str>) -> Option<usize> {
    let (_, version) = filename?.rsplit_once('.')?;
    version.parse().ok()
}

impl AtomicFile {
    pub fn new(path: impl Into<PathBuf>) -> data_error::Result<Self> {
        let directory = path.into();
        // This UID must be treated as confidential information.
        // Depending on network transport used to sync the files (if any),
        // it can leak to an unauthorized party.
        let app_id = app_id::read()?;

        std::fs::create_dir_all(&directory)?;
        let filename: &str = match directory.file_name() {
            Some(name) => name.to_str().unwrap(),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "`path` must specify a directory name",
            ))?,
        };
        let prefix = format!("{}_{}.", filename, app_id);
        Ok(Self { directory, prefix })
    }

    /// Return the latest version together with vector of the
    /// files matching this version. Multiple files for the same version
    /// can appear due to usage of file syncronization. Different devices
    /// can create same version simultaneously.
    pub fn latest_version(&self) -> Result<(usize, Vec<ReadOnlyFile>)> {
        let files_iterator = fs::read_dir(&self.directory)?.flatten();
        let (files, version) = files_iterator.into_iter().fold(
            (vec![], 0),
            |(mut files, mut current_max_version), entry| {
                let filename = entry.file_name();
                if let Some(version) = parse_version(filename.to_str()) {
                    // It's possible to have same version for two files coming
                    // from different devices or even different local apps.
                    // Add such files to the result
                    if version >= current_max_version {
                        let read_only = ReadOnlyFile {
                            version,
                            path: entry.path(),
                        };
                        files.push(read_only);
                        current_max_version = version;
                    }
                }
                (files, current_max_version)
            },
        );
        let files = files
            .into_iter()
            .filter_map(|file| {
                let file_version = parse_version(file.path.to_str())?;
                if file_version == version {
                    Some(file)
                } else {
                    None
                }
            })
            .collect();
        Ok((version, files))
    }

    pub fn path(&self, version: usize) -> PathBuf {
        self.directory
            .join(format!("{}{version}", self.prefix))
    }

    pub fn load(&self) -> Result<ReadOnlyFile> {
        let (version, mut files) = self.latest_version()?;
        let file = match files.len() {
            0 => ReadOnlyFile {
                version,
                path: self.path(version),
            },
            1 => files.remove(0),
            _ => {
                log::warn!(
                    "There is multiple files with the version {version}"
                );
                files
                    .into_iter()
                    .find(|file| {
                        if let Some(path) = file.path.to_str() {
                            path.contains(&self.prefix)
                        } else {
                            false
                        }
                    })
                    .ok_or_else(|| {
                        Error::new(
                            ErrorKind::NotFound,
                            "File not found with correct version",
                        )
                    })?
            }
        };
        Ok(file)
    }

    pub fn make_temp(&self) -> Result<TmpFile> {
        TmpFile::create_in(&self.directory)
    }

    /// Replace the contents of the file with the contents of `new` if the
    /// latest version is the same as `current`.
    ///
    /// # Errors
    /// If `io::ErrorKind::AlreadyExists` is returned, it means that the latest
    /// version was not the same as `current` and the operation must be retried
    /// with a fresher version of the file. Any other I/O error is forwarded as
    /// well.
    pub fn compare_and_swap(
        &self,
        current: &ReadOnlyFile,
        new: TmpFile,
    ) -> Result<()> {
        let new_path = self.path(current.version + 1);
        (new.file).sync_data()?;
        // Just to check if current.version is still the latest_version
        let (latest_version, _) = self.latest_version()?;
        if latest_version > current.version {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "the `current` file is not the latest version",
            ));
        }
        // May return `EEXIST`.
        let res = std::fs::hard_link(&new.path, new_path);
        if let Err(err) = res {
            #[cfg(target_family = "unix")]
            // From open(2) manual page:
            //
            // "[...] create a unique file on the same filesystem (e.g.,
            // incorporating hostname and PID), and use link(2) to make a link
            // to the lockfile. If link(2) returns 0, the lock is successful.
            // Otherwise, use stat(2) on the unique file to check if its link
            // count has increased to 2, in which case the lock is also
            // succesful."
            if new.path.metadata()?.nlink() != 2 {
                Err(err)?;
            }
            #[cfg(not(target_family = "unix"))]
            Err(err)?;
        }

        let number_of_removed = self.prune_old_versions(latest_version);
        log::debug!("pruned {} old files", number_of_removed);
        Ok(())
    }

    /// Return the number of files deleted
    fn prune_old_versions(&self, version: usize) -> usize {
        let mut deleted = 0;
        if let Ok(iterator) = fs::read_dir(&self.directory) {
            for entry in iterator.flatten() {
                if let Some(file_version) =
                    parse_version(entry.file_name().to_str())
                {
                    if file_version + MAX_VERSION_FILES - 1 <= version
                        && fs::remove_file(entry.path()).is_ok()
                    {
                        deleted += 1;
                    }
                }
            }
        }
        deleted
    }
}

#[cfg(test)]
mod tests {
    use crate::{app_id, initialize};

    use super::*;
    use rstest::rstest;
    use std::io::Write;
    use tempdir::TempDir;

    #[test]
    fn delete_old_files() {
        initialize();
        let dir = TempDir::new("max_files").unwrap();
        let root = dir.path();
        let file = AtomicFile::new(root).unwrap();
        let number_of_version = 20;
        assert!(number_of_version > MAX_VERSION_FILES);
        for i in 0..number_of_version {
            let temp = file.make_temp().unwrap();
            let current = file.load().unwrap();
            let content = format!("Version {}", i + 1);
            (&temp).write_all(content.as_bytes()).unwrap();
            file.compare_and_swap(&current, temp).unwrap();
        }

        // Check the number of files
        let version_files = fs::read_dir(root).unwrap().count();
        assert_eq!(version_files, MAX_VERSION_FILES);
    }

    #[test]
    fn multiple_version_files() {
        initialize();

        let dir = TempDir::new("multiple_version").unwrap();
        let root = dir.path();

        let file = AtomicFile::new(root).unwrap();
        let temp = file.make_temp().unwrap();
        let current = file.load().unwrap();
        let content_local = "Locally created content".to_string();
        (&temp)
            .write_all(content_local.as_bytes())
            .unwrap();
        file.compare_and_swap(&current, temp).unwrap();

        // Other peer file (renamed on purpose to validate test)
        let current = file.load().unwrap();
        let content_remote = "Content created on remote peer".to_string();
        let temp = file.make_temp().unwrap();
        (&temp)
            .write_all(content_remote.as_bytes())
            .unwrap();
        file.compare_and_swap(&current, temp).unwrap();

        let version_2_path = file.path(2);
        let rename_path =
            root.join(format!("{}_cellphoneId.1", root.display()));
        fs::rename(version_2_path, rename_path).unwrap();

        // We should take content from current peer
        let current = file.load().unwrap();
        let content = current.read_to_string().unwrap();
        assert_eq!(content, content_local);
    }

    #[rstest]
    #[case(3, &[1, 3], "case_1")]
    #[case(5, &[2, 4], "case_2")]
    #[case(10, &[3, 5, 7, 9, 10], "case_3")]
    #[case(15, &[5, 14, 15], "case_4")]
    fn latest_version(
        #[case] versions: usize,
        #[case] cellphone_versions: &[usize],
        #[case] temp_name: &str,
    ) {
        initialize();

        // Create the files without atomic to handles files names
        let dir = TempDir::new(temp_name).unwrap();
        let root = dir.path();
        let local_peer = app_id::read().unwrap();
        let file = AtomicFile::new(root).unwrap();
        let prefix = &file.prefix;
        for version in 0..versions {
            let file_path = root.join(format!("{}{}", prefix, version + 1));
            let mut file = fs::File::create(file_path).unwrap();
            let content = format!("Version {} on {local_peer}", version + 1);
            file.write_all(content.as_bytes()).unwrap();
        }

        // Write remote peer files
        let mut path = prefix.split('_');
        let path = path.next().unwrap();
        for cellphone_version in cellphone_versions {
            let file_path =
                root.join(format!("{path}_cellphone.{cellphone_version}"));
            let mut file = fs::File::create(file_path).unwrap();
            let content = format!("Version {cellphone_version} on cellphone");
            file.write_all(content.as_bytes()).unwrap();
        }
        assert_eq!(file.latest_version().unwrap().0, versions);

        let latest = file.load().unwrap();
        let latest_content = latest.read_to_string().unwrap();
        assert_eq!(
            latest_content,
            format!("Version {} on {local_peer}", versions)
        );
    }
}

use data_error::Result;

use std::{env, fs, path::Path, str};

/// Write data to a tempory file and move that written file to destination
///
/// May failed if writing or moving failed
pub fn temp_and_move(
    data: &[u8],
    dest_dir: impl AsRef<Path>,
    filename: &str,
) -> Result<()> {
    let mut path = env::temp_dir();
    path.push(filename);

    fs::write(&path, data)?;
    fs::copy(path, dest_dir.as_ref().join(filename))?;

    Ok(())
}

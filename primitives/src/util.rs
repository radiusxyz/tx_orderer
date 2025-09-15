use std::{
    fs,
    io,
    path::Path,
};

pub fn clear_dir<P: AsRef<Path>>(path: P) -> Result<(), io::Error> {
    if path.as_ref().exists() {
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                fs::remove_dir_all(&path)?;
            } else {
                fs::remove_file(&path)?;
            }
        }
    }
    Ok(())
}
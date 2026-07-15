use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub struct PreparedDataset {
    root: PathBuf,
    _temp: Option<TempDir>,
}

impl PreparedDataset {
    pub fn open(source: &Path) -> Result<Self> {
        if source.is_dir() {
            return Ok(Self {
                root: source.to_owned(),
                _temp: None,
            });
        }
        let temp = tempfile::tempdir()?;
        let file =
            std::fs::File::open(source).with_context(|| format!("open {}", source.display()))?;
        let mut archive = zip::ZipArchive::new(file)?;
        archive.extract(temp.path())?;
        let root = find_csv_root(temp.path()).context("archive did not contain food.csv")?;
        Ok(Self {
            root,
            _temp: Some(temp),
        })
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
}

fn find_csv_root(root: &Path) -> Option<PathBuf> {
    if root.join("food.csv").exists() {
        return Some(root.to_owned());
    }
    std::fs::read_dir(root)
        .ok()?
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .find_map(|entry| find_csv_root(&entry.path()))
}

#[cfg(test)]
mod test;

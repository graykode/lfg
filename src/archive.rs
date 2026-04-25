use std::collections::BTreeSet;
use std::io::Read;
use std::path::{Component, Path};

use flate2::read::GzDecoder;
use tar::Archive;

use crate::source_diff::SourceTree;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArchiveError {
    ReadFailure(String),
    UnsafePath(String),
    DuplicatePath(String),
    NonUtf8Path,
    NonUtf8File(String),
}

pub fn read_tgz_source_tree(archive_bytes: &[u8]) -> Result<SourceTree, ArchiveError> {
    let decoder = GzDecoder::new(archive_bytes);
    let mut archive = Archive::new(decoder);
    let mut source_tree = SourceTree::new();
    let mut seen_paths = BTreeSet::new();
    let entries = archive
        .entries()
        .map_err(|error| ArchiveError::ReadFailure(error.to_string()))?;

    for entry in entries {
        let mut entry = entry.map_err(|error| ArchiveError::ReadFailure(error.to_string()))?;
        if !entry.header().entry_type().is_file() {
            continue;
        }

        let path = entry
            .path()
            .map_err(|error| ArchiveError::ReadFailure(error.to_string()))?;
        let path = normalize_archive_path(path.as_ref())?;
        if !seen_paths.insert(path.clone()) {
            return Err(ArchiveError::DuplicatePath(path));
        }

        let mut content = String::new();
        entry
            .read_to_string(&mut content)
            .map_err(|_| ArchiveError::NonUtf8File(path.clone()))?;

        source_tree.insert_text_file(path, content);
    }

    Ok(source_tree)
}

fn normalize_archive_path(path: &Path) -> Result<String, ArchiveError> {
    let mut parts = Vec::new();

    for component in path.components() {
        match component {
            Component::Normal(part) => {
                let part = part.to_str().ok_or(ArchiveError::NonUtf8Path)?;
                parts.push(part.to_owned());
            }
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(ArchiveError::UnsafePath(
                    path.to_string_lossy().into_owned(),
                ));
            }
        }
    }

    if parts.is_empty() {
        return Err(ArchiveError::UnsafePath(
            path.to_string_lossy().into_owned(),
        ));
    }

    Ok(parts.join("/"))
}

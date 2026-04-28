use std::collections::BTreeSet;
use std::io::{Cursor, Read};
use std::path::{Component, Path};

use flate2::read::GzDecoder;
use tar::Archive;

use crate::evidence::SourceTree;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArchiveError {
    ReadFailure(String),
    UnsafePath(String),
    DuplicatePath(String),
    MissingGemDataArchive,
    NonUtf8Path,
    NonUtf8File(String),
}

pub fn read_source_archive_tree(
    archive_bytes: &[u8],
    archive_url: &str,
) -> Result<SourceTree, ArchiveError> {
    if archive_url_without_query(archive_url).ends_with(".gem") {
        read_gem_source_tree(archive_bytes)
    } else {
        read_tgz_source_tree(archive_bytes)
    }
}

pub fn read_tgz_source_tree(archive_bytes: &[u8]) -> Result<SourceTree, ArchiveError> {
    let decoder = GzDecoder::new(archive_bytes);
    let mut archive = Archive::new(decoder);
    read_tar_source_tree(&mut archive)
}

fn read_gem_source_tree(archive_bytes: &[u8]) -> Result<SourceTree, ArchiveError> {
    let cursor = Cursor::new(archive_bytes);
    let mut archive = Archive::new(cursor);
    let mut data_archive = None;
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
        if path != "data.tar.gz" {
            continue;
        }

        if data_archive.is_some() {
            return Err(ArchiveError::DuplicatePath(path));
        }

        let mut bytes = Vec::new();
        entry
            .read_to_end(&mut bytes)
            .map_err(|error| ArchiveError::ReadFailure(error.to_string()))?;
        data_archive = Some(bytes);
    }

    let data_archive = data_archive.ok_or(ArchiveError::MissingGemDataArchive)?;

    read_tgz_source_tree(&data_archive)
}

fn read_tar_source_tree<R: Read>(archive: &mut Archive<R>) -> Result<SourceTree, ArchiveError> {
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

fn archive_url_without_query(archive_url: &str) -> &str {
    archive_url.split(['?', '#']).next().unwrap_or(archive_url)
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

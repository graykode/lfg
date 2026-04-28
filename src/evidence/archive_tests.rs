use std::io::Write;

use crate::evidence::SourceTree;
use crate::evidence::{read_source_archive_tree, read_tgz_source_tree, ArchiveError};
use flate2::write::GzEncoder;
use flate2::Compression;
use tar::{Builder, EntryType, Header};

fn tgz(entries: &[(&str, &str)]) -> Vec<u8> {
    let mut tar_bytes = Vec::new();
    {
        let mut builder = Builder::new(&mut tar_bytes);
        for (path, content) in entries {
            let mut header = Header::new_gnu();
            header.set_path(path).expect("set tar path");
            header.set_size(content.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder
                .append(&header, content.as_bytes())
                .expect("append tar entry");
        }
        builder.finish().expect("finish tar");
    }

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&tar_bytes).expect("write gzip body");
    encoder.finish().expect("finish gzip")
}

fn gem(entries: &[(&str, &str)]) -> Vec<u8> {
    let data_archive = tgz(entries);
    let metadata_archive = {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(b"---\nname: demo\n")
            .expect("write metadata");
        encoder.finish().expect("finish metadata gzip")
    };

    let mut archive = Vec::new();
    {
        let mut builder = Builder::new(&mut archive);
        let mut data_header = Header::new_gnu();
        data_header
            .set_path("data.tar.gz")
            .expect("set gem data path");
        data_header.set_size(data_archive.len() as u64);
        data_header.set_mode(0o644);
        data_header.set_cksum();
        builder
            .append(&data_header, data_archive.as_slice())
            .expect("append gem data");

        let mut metadata_header = Header::new_gnu();
        metadata_header
            .set_path("metadata.gz")
            .expect("set gem metadata path");
        metadata_header.set_size(metadata_archive.len() as u64);
        metadata_header.set_mode(0o644);
        metadata_header.set_cksum();
        builder
            .append(&metadata_header, metadata_archive.as_slice())
            .expect("append gem metadata");

        builder.finish().expect("finish gem archive");
    }

    archive
}

fn tgz_with_unchecked_path(path: &str, content: &str) -> Vec<u8> {
    let mut tar_bytes = Vec::new();
    {
        let mut builder = Builder::new(&mut tar_bytes);
        let mut header = Header::new_gnu();
        let path_bytes = path.as_bytes();
        header.as_mut_bytes()[..path_bytes.len()].copy_from_slice(path_bytes);
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_entry_type(EntryType::Regular);
        header.set_cksum();
        builder
            .append(&header, content.as_bytes())
            .expect("append tar entry");
        builder.finish().expect("finish tar");
    }

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&tar_bytes).expect("write gzip body");
    encoder.finish().expect("finish gzip")
}

#[test]
fn reads_text_files_from_tgz_archive() {
    let archive = tgz(&[
        ("package/index.js", "module.exports = 1;\n"),
        ("package/package.json", "{\"name\":\"demo\"}\n"),
    ]);

    let source_tree = read_tgz_source_tree(&archive).expect("read source tree");

    assert_eq!(
        source_tree,
        SourceTree::from_text_files([
            ("package/index.js", "module.exports = 1;\n"),
            ("package/package.json", "{\"name\":\"demo\"}\n"),
        ])
    );
}

#[test]
fn reads_text_files_from_rubygems_gem_archive() {
    let archive = gem(&[("lib/demo.rb", "VALUE = 1\n")]);

    let source_tree =
        read_source_archive_tree(&archive, "https://rubygems.org/gems/demo-1.0.0.gem")
            .expect("read gem source tree");

    assert_eq!(
        source_tree,
        SourceTree::from_text_files([("lib/demo.rb", "VALUE = 1\n")])
    );
}

#[test]
fn rejects_duplicate_archive_paths() {
    let archive = tgz(&[
        ("package/index.js", "module.exports = 1;\n"),
        ("package/index.js", "module.exports = 2;\n"),
    ]);

    assert_eq!(
        read_tgz_source_tree(&archive),
        Err(ArchiveError::DuplicatePath("package/index.js".to_owned()))
    );
}

#[test]
fn rejects_parent_directory_archive_paths() {
    let archive = tgz_with_unchecked_path("../escape.js", "module.exports = 1;\n");

    assert_eq!(
        read_tgz_source_tree(&archive),
        Err(ArchiveError::UnsafePath("../escape.js".to_owned()))
    );
}

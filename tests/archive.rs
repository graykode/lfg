use std::io::Write;

use flate2::write::GzEncoder;
use flate2::Compression;
use lfg::archive::{read_tgz_source_tree, ArchiveError};
use lfg::source_diff::SourceTree;
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

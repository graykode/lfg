use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTree {
    files: BTreeMap<String, String>,
}

impl SourceTree {
    pub fn from_text_files<I, P, C>(files: I) -> Self
    where
        I: IntoIterator<Item = (P, C)>,
        P: Into<String>,
        C: Into<String>,
    {
        Self {
            files: files
                .into_iter()
                .map(|(path, content)| (path.into(), content.into()))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceDiff {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffError {
    EmptyPath,
}

pub trait DiffEngine {
    fn diff(&self, previous: &SourceTree, target: &SourceTree) -> Result<SourceDiff, DiffError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnifiedDiffEngine;

impl DiffEngine for UnifiedDiffEngine {
    fn diff(&self, previous: &SourceTree, target: &SourceTree) -> Result<SourceDiff, DiffError> {
        let mut text = String::new();

        for path in changed_paths(previous, target) {
            if path.is_empty() {
                return Err(DiffError::EmptyPath);
            }

            let previous_content = previous.files.get(path);
            let target_content = target.files.get(path);

            match (previous_content, target_content) {
                (Some(previous_content), Some(target_content))
                    if previous_content != target_content =>
                {
                    render_changed_file(&mut text, path, previous_content, target_content);
                }
                (None, Some(target_content)) => {
                    render_added_file(&mut text, path, target_content);
                }
                (Some(previous_content), None) => {
                    render_removed_file(&mut text, path, previous_content);
                }
                _ => {}
            }
        }

        Ok(SourceDiff { text })
    }
}

fn changed_paths<'a>(previous: &'a SourceTree, target: &'a SourceTree) -> BTreeSet<&'a str> {
    previous
        .files
        .keys()
        .chain(target.files.keys())
        .map(String::as_str)
        .collect()
}

fn render_changed_file(
    output: &mut String,
    path: &str,
    previous_content: &str,
    target_content: &str,
) {
    render_file_header(output, path, &format!("a/{path}"), &format!("b/{path}"));
    output.push_str("@@\n");

    for line in diff_lines(previous_content) {
        output.push('-');
        output.push_str(line);
        ensure_line_ending(output);
    }

    for line in diff_lines(target_content) {
        output.push('+');
        output.push_str(line);
        ensure_line_ending(output);
    }
}

fn render_added_file(output: &mut String, path: &str, target_content: &str) {
    render_file_header(output, path, "/dev/null", &format!("b/{path}"));
    output.push_str("@@\n");

    for line in diff_lines(target_content) {
        output.push('+');
        output.push_str(line);
        ensure_line_ending(output);
    }
}

fn render_removed_file(output: &mut String, path: &str, previous_content: &str) {
    render_file_header(output, path, &format!("a/{path}"), "/dev/null");
    output.push_str("@@\n");

    for line in diff_lines(previous_content) {
        output.push('-');
        output.push_str(line);
        ensure_line_ending(output);
    }
}

fn render_file_header(output: &mut String, path: &str, previous_path: &str, target_path: &str) {
    output.push_str(&format!("diff --git a/{path} b/{path}\n"));
    output.push_str(&format!("--- {previous_path}\n"));
    output.push_str(&format!("+++ {target_path}\n"));
}

fn diff_lines(content: &str) -> impl Iterator<Item = &str> {
    content.split_inclusive('\n')
}

fn ensure_line_ending(output: &mut String) {
    if !output.ends_with('\n') {
        output.push('\n');
    }
}

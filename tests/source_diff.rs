use lfg::evidence::{DiffEngine, SourceTree, UnifiedDiffEngine};

#[test]
fn renders_changed_text_file_as_unified_diff() {
    let previous = SourceTree::from_text_files([("package/index.js", "module.exports = 1;\n")]);
    let target = SourceTree::from_text_files([("package/index.js", "module.exports = 2;\n")]);

    let diff = UnifiedDiffEngine
        .diff(&previous, &target)
        .expect("source diff renders");

    assert_eq!(
        diff.text,
        "\
diff --git a/package/index.js b/package/index.js
--- a/package/index.js
+++ b/package/index.js
@@
-module.exports = 1;
+module.exports = 2;
"
    );
}

#[test]
fn renders_added_and_removed_text_files() {
    let previous = SourceTree::from_text_files([
        ("package/removed.js", "module.exports = 'removed';\n"),
        ("package/shared.js", "module.exports = 'same';\n"),
    ]);
    let target = SourceTree::from_text_files([
        ("package/added.js", "module.exports = 'added';\n"),
        ("package/shared.js", "module.exports = 'same';\n"),
    ]);

    let diff = UnifiedDiffEngine
        .diff(&previous, &target)
        .expect("source diff renders");

    assert_eq!(
        diff.text,
        "\
diff --git a/package/added.js b/package/added.js
--- /dev/null
+++ b/package/added.js
@@
+module.exports = 'added';
diff --git a/package/removed.js b/package/removed.js
--- a/package/removed.js
+++ /dev/null
@@
-module.exports = 'removed';
"
    );
}

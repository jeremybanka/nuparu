use std::fs;
use std::path::{Path, PathBuf};

use crate::nushell::{assert_parses, lite_parse_document};

fn fixture_files(root: &Path) -> Vec<PathBuf> {
    let mut entries = Vec::new();

    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("nu") {
                out.push(path);
            }
        }
    }

    walk(root, &mut entries);
    entries.sort();
    entries
}

#[test]
fn nushell_lexer_and_lite_parser_accept_all_fixture_inputs() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let root = workspace_root.join("fixtures");

    for path in fixture_files(&root) {
        let text = fs::read_to_string(&path).unwrap();
        let name = path.strip_prefix(&workspace_root).unwrap();
        let parsed = lite_parse_document(&text);

        assert!(
            parsed.lex_error.is_none(),
            "{} should lex without errors: {:?}",
            name.display(),
            parsed.lex_error
        );
        assert!(
            parsed.lite_error.is_none(),
            "{} should lite-parse without errors: {:?}",
            name.display(),
            parsed.lite_error
        );
        assert!(
            !parsed.block.block.is_empty(),
            "{} should produce at least one lite block entry",
            name.display()
        );
    }
}

#[test]
fn nushell_full_parser_handles_simple_command() {
    let parsed = assert_parses("inline.nu", "echo hi\n").unwrap();

    assert!(
        !parsed.lexed.tokens.is_empty(),
        "{} should produce tokens",
        "inline.nu"
    );
    assert!(
        !parsed.ast.pipelines.is_empty(),
        "{} should produce at least one AST pipeline",
        "inline.nu"
    );
}

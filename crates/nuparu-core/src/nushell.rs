use std::sync::Arc;

use anyhow::{Result, bail};
use nu_parser::{LiteBlock, Token, lex, lite_parse, parse};
use nu_protocol::{
    ParseError,
    ast::Block,
    engine::{EngineState, StateWorkingSet},
};

#[derive(Debug)]
pub struct LexedDocument {
    pub tokens: Vec<Token>,
    pub lex_error: Option<ParseError>,
}

#[derive(Debug)]
pub struct LiteParsedDocument {
    pub block: LiteBlock,
    pub lex_error: Option<ParseError>,
    pub lite_error: Option<ParseError>,
}

#[derive(Debug)]
pub struct ParsedDocument {
    pub lexed: LexedDocument,
    pub lite_block: LiteBlock,
    pub lite_error: Option<ParseError>,
    pub ast: Arc<Block>,
    pub parse_errors: Vec<ParseError>,
}

pub fn lex_document(file_text: &str) -> LexedDocument {
    let (tokens, lex_error) = lex(file_text.as_bytes(), 0, &[], &[], false);
    LexedDocument { tokens, lex_error }
}

pub fn lite_parse_document(file_text: &str) -> LiteParsedDocument {
    let engine_state = EngineState::new();
    let working_set = StateWorkingSet::new(&engine_state);
    let lexed = lex_document(file_text);
    let (block, lite_error) = lite_parse(&lexed.tokens, &working_set);
    LiteParsedDocument {
        block,
        lex_error: lexed.lex_error,
        lite_error,
    }
}

pub fn parse_document(file_name: Option<&str>, file_text: &str) -> ParsedDocument {
    let engine_state = EngineState::new();
    let mut working_set = StateWorkingSet::new(&engine_state);
    let lexed = lex_document(file_text);
    let (lite_block, lite_error) = lite_parse(&lexed.tokens, &working_set);
    let ast = parse(&mut working_set, file_name, file_text.as_bytes(), true);

    ParsedDocument {
        lexed,
        lite_block,
        lite_error,
        ast,
        parse_errors: working_set.parse_errors,
    }
}

pub fn assert_parses(file_name: &str, file_text: &str) -> Result<ParsedDocument> {
    let parsed = parse_document(Some(file_name), file_text);

    if let Some(error) = &parsed.lexed.lex_error {
        bail!("{file_name}: lex error: {error:?}");
    }

    if let Some(error) = &parsed.lite_error {
        bail!("{file_name}: lite parse error: {error:?}");
    }

    if let Some(error) = parsed.parse_errors.first() {
        bail!("{file_name}: parse error: {error:?}");
    }

    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::*;

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
}

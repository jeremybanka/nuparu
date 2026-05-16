use nu_parser::{Token, TokenContents};

use crate::configuration::Configuration;
use crate::nushell::lex_document;

pub fn format_text(file_text: &str, config: &Configuration) -> String {
    let normalized = file_text.replace("\r\n", "\n");
    let lexed = lex_document(&normalized);

    if lexed.lex_error.is_some() {
        return file_text.to_string();
    }

    let lines = source_lines(&normalized);
    let multiline_string_lines = multiline_string_lines(&lines);
    let mut result = String::new();
    let mut indent_level = 0usize;
    let mut blank_run = 0u8;

    for (index, line) in lines.iter().enumerate() {
        let trimmed_end = line.text.trim_end();
        let content = trimmed_end.trim_start();

        if content.is_empty() {
            let previous_content = previous_nonempty_content(&lines, index);
            let next_content = next_nonempty_content(&lines, index);

            if previous_content.is_some_and(ends_with_opener)
                || next_content.is_some_and(starts_with_closer)
            {
                continue;
            }

            blank_run = blank_run.saturating_add(1);
            if blank_run <= config.max_blank_lines {
                result.push('\n');
            }
            continue;
        }

        blank_run = 0;
        let effective_indent = if starts_with_closer(content) {
            indent_level.saturating_sub(1)
        } else {
            indent_level
        };

        let (line_tokens, has_verbatim_multiline_token, has_structural_multiline_token) =
            line_tokens(&lexed.tokens, line.start, line.end, &normalized);

        if multiline_string_lines[index] || has_verbatim_multiline_token {
            result.push_str(trimmed_end);
        } else if has_structural_multiline_token {
            result.push_str(&" ".repeat(effective_indent * config.indent_width as usize));
            result.push_str(content);
        } else {
            result.push_str(&" ".repeat(effective_indent * config.indent_width as usize));
            result.push_str(&format_line(trimmed_end, &line_tokens, &normalized));
        }

        if line.has_newline {
            result.push('\n');
        }

        if !(multiline_string_lines[index] || has_verbatim_multiline_token) {
            indent_level = next_indent_level(content, indent_level);
        }
    }

    result
}

#[derive(Clone, Copy)]
struct SourceLine<'a> {
    text: &'a str,
    start: usize,
    end: usize,
    has_newline: bool,
}

fn source_lines(source: &str) -> Vec<SourceLine<'_>> {
    let mut lines = Vec::new();
    let mut start = 0usize;

    for segment in source.split_inclusive('\n') {
        let text = segment.strip_suffix('\n').unwrap_or(segment);
        let end = start + text.len();
        lines.push(SourceLine {
            text,
            start,
            end,
            has_newline: segment.ends_with('\n'),
        });
        start += segment.len();
    }

    if source.is_empty() || source.ends_with('\n') {
        lines
    } else {
        let trailing = &source[start..];
        if trailing.is_empty() {
            lines
        } else {
            let mut lines = lines;
            lines.push(SourceLine {
                text: trailing,
                start,
                end: source.len(),
                has_newline: false,
            });
            lines
        }
    }
}

fn multiline_string_lines(lines: &[SourceLine<'_>]) -> Vec<bool> {
    let mut result = Vec::with_capacity(lines.len());
    let mut active_quote: Option<char> = None;

    for line in lines {
        let was_in_multiline = active_quote.is_some();
        let mut chars = line.text.chars().peekable();
        let mut escaped = false;

        while let Some(ch) = chars.next() {
            if let Some(quote) = active_quote {
                if escaped {
                    escaped = false;
                    continue;
                }

                if quote == '"' && ch == '\\' {
                    escaped = true;
                    continue;
                }

                if ch == quote {
                    active_quote = None;
                }
                continue;
            }

            if ch == '#' {
                break;
            }

            if ch == '$' && chars.peek() == Some(&'"') {
                chars.next();
                active_quote = Some('"');
                continue;
            }

            if matches!(ch, '"' | '\'' | '`') {
                active_quote = Some(ch);
            }
        }

        result.push(was_in_multiline || active_quote.is_some());
    }

    result
}

fn line_tokens<'a>(
    tokens: &'a [Token],
    line_start: usize,
    line_end: usize,
    source: &str,
) -> (Vec<&'a Token>, bool, bool) {
    let mut line_tokens = Vec::new();
    let mut has_verbatim_multiline_token = false;
    let mut has_structural_multiline_token = false;

    for token in tokens {
        if token.contents == TokenContents::Eol {
            continue;
        }

        if token.span.start < line_end && token.span.end > line_start {
            if token.span.start >= line_start && token.span.end <= line_end {
                line_tokens.push(token);
            } else if is_verbatim_multiline_token(token, source) {
                has_verbatim_multiline_token = true;
            } else {
                has_structural_multiline_token = true;
            }
        }
    }

    (
        line_tokens,
        has_verbatim_multiline_token,
        has_structural_multiline_token,
    )
}

fn format_line(line_text: &str, tokens: &[&Token], source: &str) -> String {
    let mut result = String::new();
    let mut prev_kind = None;

    for (index, token) in tokens.iter().enumerate() {
        let text = token_text(token, source);
        let next_kind = tokens.get(index + 1).map(|token| token.contents);

        match token.contents {
            TokenContents::Comment => {
                if result.is_empty() {
                    result.push_str(text.trim_start());
                } else {
                    trim_trailing_spaces(&mut result);
                    result.push(' ');
                    result.push_str(text.trim_start());
                }
                break;
            }
            TokenContents::Pipe
            | TokenContents::PipePipe
            | TokenContents::AssignmentOperator
            | TokenContents::ErrGreaterPipe
            | TokenContents::OutErrGreaterPipe
            | TokenContents::OutGreaterThan
            | TokenContents::OutGreaterGreaterThan
            | TokenContents::ErrGreaterThan
            | TokenContents::ErrGreaterGreaterThan
            | TokenContents::OutErrGreaterThan
            | TokenContents::OutErrGreaterGreaterThan => {
                trim_trailing_spaces(&mut result);
                if !result.is_empty() {
                    result.push(' ');
                }
                result.push_str(text);
                if next_kind.is_some() {
                    result.push(' ');
                }
            }
            TokenContents::Semicolon => {
                trim_trailing_spaces(&mut result);
                result.push_str(text);
                if next_kind.is_some() {
                    result.push(' ');
                }
            }
            TokenContents::Item | TokenContents::Eol => {
                if needs_space_before(prev_kind, token.contents, &result) {
                    result.push(' ');
                }
                result.push_str(text);
            }
        }

        prev_kind = Some(token.contents);
    }

    trim_trailing_spaces(&mut result);

    if result.is_empty() {
        line_text.trim_start().to_string()
    } else {
        result
    }
}

fn needs_space_before(
    prev_kind: Option<TokenContents>,
    current_kind: TokenContents,
    current_text: &str,
) -> bool {
    if current_text.is_empty() {
        return false;
    }

    matches!(
        (prev_kind, current_kind),
        (Some(TokenContents::Item), TokenContents::Item)
            | (Some(TokenContents::Semicolon), TokenContents::Item)
    )
}

fn starts_with_closer(content: &str) -> bool {
    matches!(content.chars().next(), Some('}' | ']' | ')'))
}

fn ends_with_opener(content: &str) -> bool {
    matches!(content.chars().last(), Some('{' | '[' | '('))
}

fn previous_nonempty_content<'a>(lines: &[SourceLine<'a>], index: usize) -> Option<&'a str> {
    lines[..index]
        .iter()
        .rev()
        .map(|line| line.text.trim())
        .find(|content| !content.is_empty())
}

fn next_nonempty_content<'a>(lines: &[SourceLine<'a>], index: usize) -> Option<&'a str> {
    lines[index + 1..]
        .iter()
        .map(|line| line.text.trim())
        .find(|content| !content.is_empty())
}

fn next_indent_level(content: &str, current_indent: usize) -> usize {
    let mut indent = current_indent;
    let mut chars = content.chars().peekable();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escaped = false;

    while let Some(ch) = chars.next() {
        if escaped {
            escaped = false;
            continue;
        }

        match ch {
            '\\' if in_single_quote || in_double_quote => {
                escaped = true;
            }
            '\'' if !in_double_quote => in_single_quote = !in_single_quote,
            '"' if !in_single_quote => in_double_quote = !in_double_quote,
            '#' if !in_single_quote && !in_double_quote => break,
            '{' | '[' | '(' if !in_single_quote && !in_double_quote => indent += 1,
            '}' | ']' | ')' if !in_single_quote && !in_double_quote => {
                indent = indent.saturating_sub(1);
            }
            _ => {}
        }
    }

    indent
}

fn token_text<'a>(token: &Token, source: &'a str) -> &'a str {
    &source[token.span.start..token.span.end]
}

fn is_verbatim_multiline_token(token: &Token, source: &str) -> bool {
    let text = token_text(token, source).trim_start();
    text.starts_with('"')
        || text.starts_with('\'')
        || text.starts_with('`')
        || text.starts_with("$\"")
        || text.starts_with("r#")
}

fn trim_trailing_spaces(text: &mut String) {
    while text.ends_with(' ') {
        text.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_pipelines_and_comments() {
        let input = "ls   |where size > 10| sort-by name   # comment";
        let output = format_text(input, &Configuration::default());
        assert_eq!(output, "ls | where size > 10 | sort-by name # comment");
    }

    #[test]
    fn preserves_double_pipe_tokens() {
        let input = "do { foo } | complete || true\n";
        let output = format_text(input, &Configuration::default());
        assert_eq!(output, "do { foo } | complete || true\n");
    }

    #[test]
    fn normalizes_block_indentation() {
        let input = "def greet [] {\nprint \"hi\"\n}\n";
        let output = format_text(input, &Configuration::default());
        assert_eq!(output, "def greet [] {\n  print \"hi\"\n}\n");
    }

    #[test]
    fn preserves_pipeline_indentation_inside_blocks() {
        let input = "def demo [] {\nopen --raw foo\n| lines\n| each {|line| $line }\n}\n";
        let output = format_text(input, &Configuration::default());
        assert_eq!(
            output,
            "def demo [] {\n  open --raw foo\n  | lines\n  | each {|line| $line }\n}\n"
        );
    }

    #[test]
    fn preserves_blank_line_limit() {
        let input = "let x = 1\n\n\n\nlet y = 2\n";
        let output = format_text(input, &Configuration::default());
        assert_eq!(output, "let x = 1\n\nlet y = 2\n");
    }

    #[test]
    fn removes_blank_lines_at_block_edges() {
        let input = "def demo [] {\n\n  print \"hi\"\n\n}\n";
        let output = format_text(input, &Configuration::default());
        assert_eq!(output, "def demo [] {\n  print \"hi\"\n}\n");
    }

    #[test]
    fn preserves_comment_spacing_from_source() {
        let input = "bun install # repo dependencies\n";
        let output = format_text(input, &Configuration::default());
        assert_eq!(output, "bun install # repo dependencies\n");
    }
}

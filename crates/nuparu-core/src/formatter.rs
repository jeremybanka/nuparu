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
        let line_body = if multiline_string_lines[index] {
            trimmed_end.to_string()
        } else if has_verbatim_multiline_token || has_structural_multiline_token {
            content.to_string()
        } else {
            format_line(trimmed_end, &line_tokens, &normalized)
        };

        let join_with_previous = join_with_previous_line(
            &result,
            &lines,
            index,
            content,
            &line_body,
            config.line_width as usize,
        );

        if let Some(separator) = join_with_previous {
            if result.ends_with('\n') {
                result.pop();
            }
            trim_trailing_spaces(&mut result);
            if !separator.is_empty() && !result.is_empty() {
                result.push_str(separator);
            }
        } else if !multiline_string_lines[index] {
            let continuation_indent = continuation_indent_level(&lines, index, content);
            result.push_str(
                &" ".repeat(
                    (effective_indent + continuation_indent) * config.indent_width as usize,
                ),
            );
        }

        if multiline_string_lines[index] {
            if join_with_previous.is_none() {
                result.push_str(trimmed_end);
            } else {
                result.push_str(content);
            }
        } else {
            result.push_str(&line_body);
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

fn join_with_previous_line(
    result: &str,
    lines: &[SourceLine<'_>],
    index: usize,
    content: &str,
    line_body: &str,
    line_width: usize,
) -> Option<&'static str> {
    if index == 0 {
        return None;
    }

    let previous_source = lines[index - 1].text.trim();
    if previous_source.is_empty() {
        return None;
    }

    let previous_output = last_output_line(result)?;
    let separator = join_separator(previous_output, previous_source, content, lines, index)?;
    let candidate_length = previous_output.len() + separator.len() + line_body.len();

    (candidate_length <= line_width).then_some(separator)
}

fn join_separator(
    previous_output: &str,
    previous_source: &str,
    content: &str,
    lines: &[SourceLine<'_>],
    index: usize,
) -> Option<&'static str> {
    let current_indent = leading_indent(lines[index].text);
    let previous_indent = leading_indent(lines[index - 1].text);
    let continuation_indent = current_indent >= previous_indent;

    if (content.starts_with('(') || content.starts_with('['))
        && (previous_output.ends_with('=')
            || previous_output.ends_with("return")
            || matches!(previous_output, "if" | "else if" | "while" | "match"))
    {
        return Some(" ");
    }

    if content == "{" && can_join_block_opener(previous_output) {
        return Some(" ");
    }

    if content.starts_with('|') {
        if previous_output.ends_with('{') && is_closure_signature(content) {
            return Some("");
        }

        let next_pipe = next_nonempty_content(lines, index).filter(|next| next.starts_with('|'));

        if previous_source.starts_with('|') || next_pipe.is_some() {
            if is_simple_pipeline_stage(content)
                && next_pipe.is_none_or(is_simple_pipeline_stage)
                && (previous_output.contains(" | ") || !previous_source.starts_with('|'))
            {
                return Some(" ");
            }

            if !previous_source.starts_with('|')
                && next_pipe.is_some_and(is_closure_signature)
                && content.ends_with('{')
            {
                return Some(" ");
            }

            return None;
        }

        return Some(" ");
    }

    if starts_with_boolean_connector(content) && previous_output.ends_with('(') {
        return Some(" ");
    }

    if previous_output.ends_with(':') && continuation_indent && is_type_continuation(content) {
        return Some(" ");
    }

    if previous_output.ends_with('=') && continuation_indent {
        return Some(" ");
    }

    if previous_output.ends_with("return") && is_simple_expression_start(content) {
        return Some(" ");
    }

    if previous_output.ends_with('{') && is_closure_signature(previous_source) {
        return Some("");
    }

    if is_closure_signature(previous_output) && is_simple_expression_start(content) {
        return Some(" ");
    }

    if content == "}" && previous_output.contains("{|") && !previous_source.contains("{|") {
        return Some(" ");
    }

    if continuation_indent
        && can_extend_command(previous_output)
        && is_command_continuation(content, previous_output)
    {
        return Some(" ");
    }

    None
}

fn can_join_block_opener(previous: &str) -> bool {
    !previous.ends_with('{')
        && (previous.ends_with(')')
            || previous.ends_with(']')
            || previous.ends_with("else")
            || previous.ends_with('"')
            || previous.ends_with('\''))
}

fn starts_with_boolean_connector(content: &str) -> bool {
    content.starts_with("and ") || content.starts_with("or ")
}

fn is_type_continuation(content: &str) -> bool {
    content
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '[' || ch == '(')
}

fn is_simple_expression_start(content: &str) -> bool {
    content.chars().next().is_some_and(|ch| {
        matches!(ch, '$' | '"' | '\'' | '[' | '{' | '(' | '^') || ch.is_ascii_alphanumeric()
    })
}

fn is_command_continuation(content: &str, previous_output: &str) -> bool {
    if content.contains(':') || previous_output.contains(':') {
        return false;
    }

    content.starts_with("--")
        || content.starts_with('$')
        || content.starts_with('(')
        || content.starts_with('"')
        || content.starts_with('\'')
        || (is_simple_expression_start(content) && previous_output.contains(' '))
}

fn can_extend_command(previous_output: &str) -> bool {
    !previous_output.starts_with("let ")
        && !previous_output.starts_with("mut ")
        && !previous_output.starts_with("export def ")
        && !previous_output.starts_with("if ")
        && !previous_output.starts_with("else if ")
        && !previous_output.starts_with("while ")
        && !previous_output.starts_with("for ")
        && !previous_output.starts_with("return ")
        && !previous_output.ends_with('{')
        && !previous_output.ends_with('}')
        && !previous_output.ends_with(')')
        && !previous_output.ends_with(']')
        && !previous_output.ends_with(';')
        && !previous_output.contains(" = ")
}

fn is_closure_signature(content: &str) -> bool {
    content.starts_with('|') && content[1..].contains('|')
}

fn leading_indent(text: &str) -> usize {
    text.chars()
        .take_while(|ch| ch.is_ascii_whitespace())
        .count()
}

fn continuation_indent_level(lines: &[SourceLine<'_>], index: usize, content: &str) -> usize {
    if index == 0 {
        return 0;
    }

    let previous = lines[index - 1].text.trim();
    let current_indent = leading_indent(lines[index].text);
    let previous_indent = leading_indent(lines[index - 1].text);

    if current_indent <= previous_indent {
        return 0;
    }

    if previous.ends_with('=') || previous.ends_with(':') {
        return 1;
    }

    if matches!(previous, "if" | "else if" | "while" | "match" | "return")
        && (content.starts_with('(') || content.starts_with('{'))
    {
        return 1;
    }

    0
}

fn is_simple_pipeline_stage(content: &str) -> bool {
    matches!(
        content.trim(),
        "| complete" | "| ignore" | "| lines" | "| first" | "| last"
    )
}

fn last_output_line(result: &str) -> Option<&str> {
    let text = result.strip_suffix('\n').unwrap_or(result);
    if text.is_empty() {
        None
    } else {
        Some(text.rsplit('\n').next().unwrap_or(text))
    }
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
    let chars = content.chars();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escaped = false;

    for ch in chars {
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

    #[test]
    fn rejoins_parenthesized_assignments_split_at_spaces() {
        let input = "let parsed =\n($line | parse --regex \"x\")\n";
        let output = format_text(input, &Configuration::default());
        assert_eq!(output, "let parsed = ($line | parse --regex \"x\")\n");
    }

    #[test]
    fn rejoins_if_conditions_and_block_openers_split_at_spaces() {
        let input = "if\n(\n$raw_value | str starts-with \"~/\"\n)\n{\n$env.HOME\n}\n";
        let output = format_text(input, &Configuration::default());
        assert_eq!(
            output,
            "if (\n  $raw_value | str starts-with \"~/\"\n) {\n  $env.HOME\n}\n"
        );
    }

    #[test]
    fn rejoins_parameter_types_and_defaults() {
        let input = "export def get-setting [\n  settings:\n    record\n  default_value: any =\n    null\n]\n";
        let output = format_text(input, &Configuration::default());
        assert_eq!(
            output,
            "export def get-setting [\n  settings: record\n  default_value: any = null\n]\n"
        );
    }

    #[test]
    fn keeps_distinct_parameters_on_separate_lines() {
        let input = "export def get-setting [\n  settings:\n    record\n  key: string\n  default_value: any =\n    null\n]\n";
        let output = format_text(input, &Configuration::default());
        assert_eq!(
            output,
            "export def get-setting [\n  settings: record\n  key: string\n  default_value: any = null\n]\n"
        );
    }

    #[test]
    fn keeps_long_assignments_broken_when_they_exceed_line_width() {
        let input = "let latest_url =\n  $\"https://channels.nixos.org/($channel)/latest-nixos-($flavor)-($arch)-linux.iso\"\n";
        let output = format_text(input, &Configuration::default());
        assert_eq!(output, input);
    }

    #[test]
    fn rejoins_short_pipelines() {
        let input = "$env.FILE_PWD\n| path dirname\n";
        let output = format_text(input, &Configuration::default());
        assert_eq!(output, "$env.FILE_PWD | path dirname\n");
    }

    #[test]
    fn keeps_long_pipelines_broken_when_they_exceed_line_width() {
        let input = "open --raw $settings_file\n| lines\n| each {|line| $line | str trim }\n| where {|line| $line != \"\" and not ($line | str starts-with \"#\") }\n";
        let output = format_text(input, &Configuration::default());
        assert_eq!(output, input);
    }

    #[test]
    fn rejoins_command_heads_and_arguments() {
        let input = "open\n  --raw\n  $settings_file\n";
        let output = format_text(input, &Configuration::default());
        assert_eq!(output, "open --raw $settings_file\n");
    }

    #[test]
    fn rejoins_boolean_conditions_when_they_fit() {
        let input =
            "if (\n  $line != \"\"\n  and not ($line | str starts-with \"#\")\n) {\n  $line\n}\n";
        let output = format_text(input, &Configuration::default());
        assert_eq!(
            output,
            "if (\n  $line != \"\" and not ($line | str starts-with \"#\")\n) {\n  $line\n}\n"
        );
    }

    #[test]
    fn rejoins_return_record_literals() {
        let input = "return\n{\n  key: value\n}\n";
        let output = format_text(input, &Configuration::default());
        assert_eq!(output, "return {\n  key: value\n}\n");
    }

    #[test]
    fn rejoins_closure_signatures_and_simple_bodies() {
        let input = "items\n| each {\n  |line|\n  $line | str trim\n}\n";
        let output = format_text(input, &Configuration::default());
        assert_eq!(output, "items | each {|line| $line | str trim }\n");
    }

    #[test]
    fn rejoins_completion_tails() {
        let input = "do { ^limactl stop $instance_name }\n| complete\n| ignore\n";
        let output = format_text(input, &Configuration::default());
        assert_eq!(
            output,
            "do { ^limactl stop $instance_name } | complete | ignore\n"
        );
    }
}

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
    let preserved_multiline_list_lines = preserved_multiline_list_lines(&lines);
    let preserved_multiline_group_expression_lines =
        preserved_multiline_group_expression_lines(&lines);
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

        let continuation_indent = continuation_indent_level(&lines, index, content);
        let line_indent_width =
            (effective_indent + continuation_indent) * config.indent_width as usize;
        let (line_tokens, has_verbatim_multiline_token, has_structural_multiline_token) =
            line_tokens(&lexed.tokens, line.start, line.end, &normalized);
        let line_body = if multiline_string_lines[index] {
            trimmed_end.to_string()
        } else if has_verbatim_multiline_token || has_structural_multiline_token {
            restore_separator_spaces(&split_reserved_statement_heads(
                &normalize_inline_whitespace(content),
                line_indent_width,
            ))
        } else {
            format_line(trimmed_end, &line_tokens, &normalized, line_indent_width)
        };

        let join_with_previous = if multiline_string_lines[index]
            || (index > 0 && multiline_string_lines[index - 1])
            || preserved_multiline_list_lines[index]
            || preserved_multiline_group_expression_lines[index]
        {
            None
        } else {
            join_with_previous_line(
                &result,
                &lines,
                index,
                content,
                &line_body,
                config.line_width as usize,
            )
        };

        if let Some(separator) = join_with_previous {
            if result.ends_with('\n') {
                result.pop();
            }
            trim_trailing_spaces(&mut result);
            if !separator.is_empty() && !result.is_empty() {
                result.push_str(separator);
            }
        } else if !multiline_string_lines[index] {
            if preserved_multiline_group_expression_lines[index] {
                let indent_width = if should_preserve_group_expression_source_indent(content) {
                    leading_indent(lines[index].text)
                } else {
                    line_indent_width
                };
                result.push_str(&" ".repeat(indent_width));
            } else {
                result.push_str(&" ".repeat(line_indent_width));
            }
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

fn preserved_multiline_list_lines(lines: &[SourceLine<'_>]) -> Vec<bool> {
    const MULTILINE_LIST_PRESERVE_THRESHOLD: usize = 6;

    #[derive(Clone, Copy)]
    struct OpenList {
        start_line: usize,
        start_depth: usize,
    }

    let mut preserve = vec![false; lines.len()];
    let mut list_stack: Vec<OpenList> = Vec::new();
    let mut nesting_stack: Vec<char> = Vec::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escaped = false;

    for (line_index, line) in lines.iter().enumerate() {
        let chars = line.text.chars();

        for ch in chars {
            if escaped {
                escaped = false;
                continue;
            }

            if in_single_quote {
                if ch == '\\' {
                    escaped = true;
                } else if ch == '\'' {
                    in_single_quote = false;
                }
                continue;
            }

            if in_double_quote {
                if ch == '\\' {
                    escaped = true;
                } else if ch == '"' {
                    in_double_quote = false;
                }
                continue;
            }

            match ch {
                '#' => break,
                '\'' => in_single_quote = true,
                '"' => in_double_quote = true,
                '[' => {
                    let start_depth = nesting_stack.len();
                    nesting_stack.push('[');
                    list_stack.push(OpenList {
                        start_line: line_index,
                        start_depth,
                    });
                }
                '{' | '(' => nesting_stack.push(ch),
                ']' => {
                    if matches!(nesting_stack.last(), Some('[')) {
                        nesting_stack.pop();
                    }

                    if let Some(open_list) = list_stack.pop()
                        && open_list.start_line < line_index
                    {
                        let item_lines = count_significant_list_item_lines(
                            lines,
                            open_list.start_line,
                            line_index,
                            open_list.start_depth,
                        );
                        if item_lines >= MULTILINE_LIST_PRESERVE_THRESHOLD {
                            for preserved_line in preserve
                                .iter_mut()
                                .take(line_index)
                                .skip(open_list.start_line + 1)
                            {
                                *preserved_line = true;
                            }
                        }
                    }
                }
                '}' => {
                    if matches!(nesting_stack.last(), Some('{')) {
                        nesting_stack.pop();
                    }
                }
                ')' => {
                    if matches!(nesting_stack.last(), Some('(')) {
                        nesting_stack.pop();
                    }
                }
                _ => {}
            }
        }
    }

    preserve
}

fn preserved_multiline_group_expression_lines(lines: &[SourceLine<'_>]) -> Vec<bool> {
    #[derive(Clone, Copy)]
    struct OpenGroup {
        start_line: usize,
    }

    let mut preserve = vec![false; lines.len()];
    let mut group_stack: Vec<OpenGroup> = Vec::new();
    let mut nesting_stack: Vec<char> = Vec::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escaped = false;

    for (line_index, line) in lines.iter().enumerate() {
        let chars = line.text.chars();

        for ch in chars {
            if escaped {
                escaped = false;
                continue;
            }

            if in_single_quote {
                if ch == '\\' {
                    escaped = true;
                } else if ch == '\'' {
                    in_single_quote = false;
                }
                continue;
            }

            if in_double_quote {
                if ch == '\\' {
                    escaped = true;
                } else if ch == '"' {
                    in_double_quote = false;
                }
                continue;
            }

            match ch {
                '#' => break,
                '\'' => in_single_quote = true,
                '"' => in_double_quote = true,
                '(' => {
                    nesting_stack.push('(');
                    group_stack.push(OpenGroup {
                        start_line: line_index,
                    });
                }
                '[' | '{' => nesting_stack.push(ch),
                ')' => {
                    if matches!(nesting_stack.last(), Some('(')) {
                        nesting_stack.pop();
                    }

                    if let Some(open_group) = group_stack.pop()
                        && open_group.start_line < line_index
                        && should_preserve_multiline_group_expression(
                            lines,
                            open_group.start_line,
                            line_index,
                        )
                    {
                        for preserved_line in preserve
                            .iter_mut()
                            .take(line_index)
                            .skip(open_group.start_line + 1)
                        {
                            *preserved_line = true;
                        }
                    }
                }
                ']' => {
                    if matches!(nesting_stack.last(), Some('[')) {
                        nesting_stack.pop();
                    }
                }
                '}' => {
                    if matches!(nesting_stack.last(), Some('{')) {
                        nesting_stack.pop();
                    }
                }
                _ => {}
            }
        }
    }

    preserve
}

fn should_preserve_multiline_group_expression(
    lines: &[SourceLine<'_>],
    start_line: usize,
    end_line: usize,
) -> bool {
    let opener = lines[start_line].text.trim();

    if !(opener == "(" || opener.ends_with("= (") || opener.ends_with("return (")) {
        return false;
    }

    count_significant_group_expression_lines(lines, start_line, end_line) >= 2
}

fn count_significant_group_expression_lines(
    lines: &[SourceLine<'_>],
    start_line: usize,
    end_line: usize,
) -> usize {
    let mut count = 0usize;

    for line in &lines[start_line + 1..end_line] {
        let content = line.text.trim();
        if content.is_empty() {
            continue;
        }

        let structural_only = content
            .chars()
            .all(|ch| ch.is_ascii_whitespace() || matches!(ch, '(' | ')' | '[' | ']' | '{' | '}'));

        if structural_only {
            continue;
        }

        count += 1;
    }

    count
}

fn count_significant_list_item_lines(
    lines: &[SourceLine<'_>],
    start_line: usize,
    end_line: usize,
    _list_depth: usize,
) -> usize {
    let mut count = 0usize;

    for line in &lines[start_line + 1..end_line] {
        let content = line.text.trim();
        if content.is_empty() {
            continue;
        }

        let structural_only = content.chars().all(|ch| {
            ch.is_ascii_whitespace() || matches!(ch, '[' | ']' | '(' | ')' | '{' | '}' | ',')
        });

        if structural_only {
            continue;
        }

        count += 1;
    }

    count
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

fn format_line(line_text: &str, tokens: &[&Token], source: &str, indent_width: usize) -> String {
    let mut result = String::new();
    let mut prev_kind = None;

    for (index, token) in tokens.iter().enumerate() {
        let raw_text = token_text(token, source);
        let text = if token.contents == TokenContents::Item {
            normalize_inline_whitespace(raw_text)
        } else {
            raw_text.to_string()
        };
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
                result.push_str(&text);
                if next_kind.is_some() {
                    result.push(' ');
                }
            }
            TokenContents::Semicolon => {
                trim_trailing_spaces(&mut result);
                result.push_str(&text);
                if next_kind.is_some() {
                    result.push(' ');
                }
            }
            TokenContents::Item | TokenContents::Eol => {
                if needs_space_before(prev_kind, token.contents, &result) {
                    result.push(' ');
                }
                result.push_str(&text);
            }
        }

        prev_kind = Some(token.contents);
    }

    trim_trailing_spaces(&mut result);

    let result = restore_separator_spaces(&split_reserved_statement_heads(&result, indent_width));

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

fn is_reserved_statement_head(text: &str) -> bool {
    matches!(
        text,
        "let" | "const" | "mut" | "return" | "if" | "for" | "while" | "match"
    )
}

fn split_reserved_statement_heads(text: &str, indent_width: usize) -> String {
    let mut result = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut quote: Option<char> = None;
    let mut escaped = false;
    let mut index = 0usize;

    while index < chars.len() {
        let ch = chars[index];

        if let Some(active_quote) = quote {
            result.push(ch);

            if escaped {
                escaped = false;
                index += 1;
                continue;
            }

            if ch == '\\' {
                escaped = true;
                index += 1;
                continue;
            }

            if ch == active_quote {
                quote = None;
            }

            index += 1;
            continue;
        }

        if matches!(ch, '"' | '\'' | '`') {
            quote = Some(ch);
            result.push(ch);
            index += 1;
            continue;
        }

        if ch.is_ascii_alphabetic() {
            let word_start = index;
            while index < chars.len()
                && (chars[index].is_ascii_alphanumeric() || chars[index] == '_')
            {
                index += 1;
            }

            let word: String = chars[word_start..index].iter().collect();
            let preceded_by_whitespace =
                word_start > 0 && chars[word_start - 1].is_ascii_whitespace();

            if preceded_by_whitespace
                && is_reserved_statement_head(&word)
                && should_split_reserved_statement_line(&result, &word)
            {
                trim_trailing_spaces(&mut result);
                result.push('\n');
                result.push_str(&" ".repeat(indent_width));
            }

            result.push_str(&word);
            continue;
        }

        result.push(ch);
        index += 1;
    }

    result
}

fn restore_separator_spaces(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut quote: Option<char> = None;
    let mut escaped = false;
    let mut delimiter_stack: Vec<char> = Vec::new();
    let mut in_closure_signature = false;
    let mut index = 0usize;

    while index < chars.len() {
        let ch = chars[index];

        if let Some(active_quote) = quote {
            result.push(ch);

            if escaped {
                escaped = false;
                index += 1;
                continue;
            }

            if ch == '\\' {
                escaped = true;
                index += 1;
                continue;
            }

            if ch == active_quote {
                quote = None;
            }

            index += 1;
            continue;
        }

        match ch {
            '"' | '\'' | '`' => {
                quote = Some(ch);
                result.push(ch);
            }
            '[' | '(' => {
                delimiter_stack.push(ch);
                result.push(ch);
            }
            '{' => {
                delimiter_stack.push(ch);
                result.push(ch);

                if chars.get(index + 1) == Some(&'|') {
                    result.push(' ');
                }
            }
            ']' => {
                if delimiter_stack.last() == Some(&'[') {
                    delimiter_stack.pop();
                }
                result.push(ch);
            }
            ')' => {
                if delimiter_stack.last() == Some(&'(') {
                    delimiter_stack.pop();
                }
                result.push(ch);
            }
            '}' => {
                if needs_space_before_inline_closer(&result) {
                    trim_trailing_spaces(&mut result);
                    result.push(' ');
                }

                if delimiter_stack.last() == Some(&'{') {
                    delimiter_stack.pop();
                }
                result.push(ch);
                in_closure_signature = false;
            }
            ':' => {
                result.push(ch);
                if delimiter_stack
                    .last()
                    .is_some_and(|delimiter| matches!(delimiter, '[' | '{'))
                    && chars
                        .get(index + 1)
                        .is_some_and(|next| !next.is_ascii_whitespace())
                {
                    result.push(' ');
                }
            }
            ',' => {
                result.push(ch);
                if delimiter_stack.last() == Some(&'[')
                    && chars
                        .get(index + 1)
                        .is_some_and(|next| !next.is_ascii_whitespace())
                {
                    result.push(' ');
                }
            }
            '|' => {
                let previous_nonspace = result.chars().rev().find(|ch| !ch.is_ascii_whitespace());
                let opening_closure_bar = previous_nonspace == Some('{')
                    || (result.trim().is_empty() && looks_like_closure_signature(&chars, index));
                let closing_closure_bar = in_closure_signature && !opening_closure_bar;

                result.push(ch);

                if opening_closure_bar {
                    in_closure_signature = true;
                } else if closing_closure_bar
                    && chars
                        .get(index + 1)
                        .is_some_and(|next| !next.is_ascii_whitespace())
                {
                    result.push(' ');
                    in_closure_signature = false;
                } else if !in_closure_signature
                    && chars
                        .get(index + 1)
                        .is_some_and(|next| !next.is_ascii_whitespace())
                    && chars.get(index + 1) != Some(&'|')
                {
                    result.push(' ');
                }
            }
            _ => result.push(ch),
        }

        index += 1;
    }

    result
}

fn needs_space_before_inline_closer(current_output: &str) -> bool {
    current_output
        .chars()
        .rev()
        .find(|ch| !ch.is_ascii_whitespace())
        .is_some_and(|ch| !matches!(ch, '{' | '[' | '(' | '|'))
}

fn looks_like_closure_signature(chars: &[char], start_index: usize) -> bool {
    if chars.get(start_index) != Some(&'|') {
        return false;
    }

    chars[start_index + 1..].contains(&'|')
}

fn should_split_reserved_statement_line(current_output: &str, next_word: &str) -> bool {
    let current_line = current_output
        .lines()
        .next_back()
        .map(str::trim_end)
        .unwrap_or("");

    if current_line.trim().is_empty() {
        return false;
    }

    let trimmed = current_line.trim_end();

    if matches!(trimmed.chars().last(), Some('{' | '[' | '(' | '|')) {
        return false;
    }

    if matches!(trimmed, "let" | "const" | "mut") {
        return false;
    }

    if next_word == "if"
        && (trimmed.ends_with("else") || matches!(trimmed.chars().last(), Some('=')))
    {
        return false;
    }

    true
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
    let previous_output = previous_output.trim_start();
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

    if content == "{"
        && (can_join_block_opener(previous_output)
            || (previous_output == ")" && follows_split_parenthesized_block_header(lines, index)))
    {
        return Some(" ");
    }

    if content.starts_with('|') {
        if previous_output.ends_with('{') && is_closure_signature(content) {
            return Some(" ");
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

    if starts_with_boolean_connector(content) && current_indent == previous_indent {
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

    if previous_output.ends_with('{') && opens_inline_closure(previous_source) {
        return Some("");
    }

    if opens_inline_closure(previous_source)
        && !is_catch_clause_line(previous_source)
        && should_join_closure_body_line(lines, index, content)
    {
        return Some(" ");
    }

    if opens_inline_closure(previous_output)
        && !is_catch_clause_line(previous_output)
        && should_join_closure_body_line(lines, index, content)
    {
        return Some(" ");
    }

    if content == "}"
        && opens_inline_closure(previous_output)
        && !opens_inline_closure(previous_source)
    {
        return Some(" ");
    }

    if continuation_indent && can_extend_command(previous_output) {
        if current_indent > previous_indent && is_command_continuation(content, previous_output) {
            return Some(" ");
        }

        if current_indent == previous_indent
            && can_join_equal_indent_command_continuation(previous_source, content)
        {
            return Some(" ");
        }
    }

    None
}

fn can_join_block_opener(previous: &str) -> bool {
    !previous.ends_with('{')
        && (previous.starts_with("if ")
            || previous.starts_with("else if ")
            || previous == "else"
            || previous.starts_with("while ")
            || previous.starts_with("for ")
            || previous.starts_with("match ")
            || previous.starts_with("def ")
            || previous.starts_with("export def ")
            || previous == "try"
            || previous == "do"
            || previous.starts_with("do "))
}

fn follows_split_parenthesized_block_header(lines: &[SourceLine<'_>], index: usize) -> bool {
    let mut found_open_paren = false;

    for content in lines[..index].iter().rev().map(|line| line.text.trim()) {
        if content.is_empty() {
            continue;
        }

        if !found_open_paren {
            if content == "(" {
                found_open_paren = true;
            }
            continue;
        }

        return matches!(content, "if" | "else if" | "while" | "match");
    }

    false
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
        && !contains_closure_signature(previous_output)
}

fn can_join_equal_indent_command_continuation(previous_source: &str, content: &str) -> bool {
    if !is_simple_argument_line(previous_source) {
        return false;
    }

    content.starts_with("--")
        || content.starts_with('$')
        || content.starts_with('(')
        || content.starts_with('"')
        || content.starts_with('\'')
        || content.starts_with('[')
}

fn is_simple_argument_line(content: &str) -> bool {
    content.starts_with("--")
        || content.starts_with('$')
        || content.starts_with('(')
        || content.starts_with('"')
        || content.starts_with('\'')
        || content.starts_with('[')
}

fn is_simple_closure_body_line(content: &str) -> bool {
    if content.starts_with("let ")
        || content.starts_with("mut ")
        || content.starts_with("if ")
        || content.starts_with("for ")
        || content.starts_with("while ")
        || content.starts_with("match ")
    {
        return false;
    }

    is_simple_expression_start(content)
}

fn should_join_closure_body_line(lines: &[SourceLine<'_>], index: usize, content: &str) -> bool {
    is_simple_closure_body_line(content)
        && next_nonempty_content(lines, index).is_some_and(|next| next == "}")
}

fn is_closure_signature(content: &str) -> bool {
    content.starts_with('|') && content[1..].contains('|')
}

fn opens_inline_closure(content: &str) -> bool {
    let trimmed = content.trim_start();
    if is_closure_signature(trimmed) {
        return true;
    }

    last_unmatched_open_brace(trimmed)
        .map(|open_brace| trimmed[open_brace + 1..].trim_start())
        .is_some_and(is_closure_signature)
}

fn is_catch_clause_line(content: &str) -> bool {
    let trimmed = content.trim_start();
    trimmed.starts_with("catch ") || trimmed.contains(" catch ")
}

fn last_unmatched_open_brace(content: &str) -> Option<usize> {
    let mut unmatched_open_braces = Vec::new();

    for (index, ch) in content.char_indices() {
        match ch {
            '{' => unmatched_open_braces.push(index),
            '}' => {
                unmatched_open_braces.pop();
            }
            _ => {}
        }
    }

    unmatched_open_braces.last().copied()
}

fn contains_closure_signature(content: &str) -> bool {
    if is_closure_signature(content.trim_start()) {
        return true;
    }

    if let Some(open_brace) = content.find('{') {
        let after_brace = content[open_brace + 1..].trim_start();
        return is_closure_signature(after_brace);
    }

    false
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

fn should_preserve_group_expression_source_indent(content: &str) -> bool {
    is_simple_argument_line(content) || content == "{" || content == "}" || content.contains(':')
}

fn normalize_inline_whitespace(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut quote: Option<char> = None;
    let mut escaped = false;
    let mut pending_space = false;

    for ch in text.chars() {
        if let Some(active_quote) = quote {
            result.push(ch);

            if escaped {
                escaped = false;
                continue;
            }

            if ch == '\\' {
                escaped = true;
                continue;
            }

            if ch == active_quote {
                quote = None;
            }

            continue;
        }

        if ch.is_ascii_whitespace() {
            pending_space = !result.is_empty();
            continue;
        }

        if pending_space {
            result.push(' ');
            pending_space = false;
        }

        result.push(ch);

        if matches!(ch, '"' | '\'' | '`') {
            quote = Some(ch);
        }
    }

    result
}

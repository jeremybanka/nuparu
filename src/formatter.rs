use crate::configuration::Configuration;

pub fn format_text(file_text: &str, config: &Configuration) -> String {
    let normalized = file_text.replace("\r\n", "\n");
    let mut result = String::new();
    let mut indent_level = 0usize;
    let mut blank_run = 0u8;

    for raw_line in normalized.lines() {
        let trimmed_end = raw_line.trim_end();
        let content = trimmed_end.trim_start();

        if content.is_empty() {
            blank_run = blank_run.saturating_add(1);
            if blank_run <= config.max_blank_lines {
                result.push('\n');
            }
            continue;
        }

        blank_run = 0;
        let effective_indent = indent_for_line(content, indent_level);
        result.push_str(&" ".repeat(effective_indent * config.indent_width as usize));
        result.push_str(&normalize_inline_spacing(content));
        result.push('\n');
        indent_level = next_indent_level(content, effective_indent);
    }

    if file_text.ends_with('\n') || result.is_empty() {
        result
    } else {
        result.trim_end_matches('\n').to_string()
    }
}

fn indent_for_line(content: &str, current_indent: usize) -> usize {
    if starts_with_closer(content) {
        current_indent.saturating_sub(1)
    } else {
        current_indent
    }
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

fn starts_with_closer(content: &str) -> bool {
    matches!(content.chars().next(), Some('}' | ']' | ')'))
}

fn normalize_inline_spacing(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escaped = false;
    let mut saw_space = false;

    while let Some(ch) = chars.next() {
        if escaped {
            result.push(ch);
            escaped = false;
            continue;
        }

        match ch {
            '\\' if in_single_quote || in_double_quote => {
                result.push(ch);
                escaped = true;
            }
            '\'' if !in_double_quote => {
                if saw_space && !result.ends_with(' ') {
                    result.push(' ');
                }
                result.push(ch);
                in_single_quote = !in_single_quote;
                saw_space = false;
            }
            '"' if !in_single_quote => {
                if saw_space && !result.ends_with(' ') {
                    result.push(' ');
                }
                result.push(ch);
                in_double_quote = !in_double_quote;
                saw_space = false;
            }
            '#' if !in_single_quote && !in_double_quote => {
                if !result.is_empty() && !result.ends_with(' ') {
                    result.push(' ');
                }
                result.push('#');
                let comment = chars.collect::<String>();
                result.push_str(comment.trim_start());
                break;
            }
            ' ' | '\t' if !in_single_quote && !in_double_quote => {
                if !result.is_empty() && !result.ends_with(' ') {
                    saw_space = true;
                }
            }
            '|' if !in_single_quote && !in_double_quote => {
                trim_trailing_space(&mut result);
                if !result.is_empty() {
                    result.push(' ');
                }
                result.push('|');
                if chars.peek().is_some() {
                    result.push(' ');
                }
                saw_space = false;
            }
            ',' if !in_single_quote && !in_double_quote => {
                trim_trailing_space(&mut result);
                result.push(',');
                if chars.peek().is_some() {
                    result.push(' ');
                }
                saw_space = false;
            }
            _ => {
                if saw_space && !result.ends_with(' ') {
                    result.push(' ');
                }
                result.push(ch);
                saw_space = false;
            }
        }
    }

    trim_trailing_space(&mut result);
    result
}

fn trim_trailing_space(text: &mut String) {
    while text.ends_with(' ') {
        text.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_pipelines_and_comments() {
        let input = "ls   |where size > 10| sort-by name   #comment";
        let output = format_text(input, &Configuration::default());
        assert_eq!(output, "ls | where size > 10 | sort-by name #comment");
    }

    #[test]
    fn normalizes_block_indentation() {
        let input = "def greet [] {\nprint \"hi\"\n}\n";
        let output = format_text(input, &Configuration::default());
        assert_eq!(output, "def greet [] {\n  print \"hi\"\n}\n");
    }

    #[test]
    fn preserves_blank_line_limit() {
        let input = "let x = 1\n\n\n\nlet y = 2\n";
        let output = format_text(input, &Configuration::default());
        assert_eq!(output, "let x = 1\n\nlet y = 2\n");
    }
}

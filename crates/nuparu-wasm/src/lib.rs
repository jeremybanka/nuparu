use nuparu_core::{Configuration, format_text};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = formatText)]
pub fn format_text_wasm(
    file_text: &str,
    indent_width: u8,
    max_blank_lines: u8,
    line_width: u16,
) -> String {
    format_text(
        file_text,
        &Configuration {
            indent_width,
            max_blank_lines,
            line_width,
        },
    )
}

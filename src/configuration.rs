use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    pub indent_width: u8,
    pub max_blank_lines: u8,
    pub line_width: u16,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            indent_width: 2,
            max_blank_lines: 1,
            line_width: 80,
        }
    }
}

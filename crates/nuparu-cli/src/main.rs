use std::io::{self, Read};

use anyhow::Result;
use dprint_core::async_runtime::LocalBoxFuture;
use dprint_core::async_runtime::async_trait;
use dprint_core::configuration::{
    ConfigKeyMap, ConfigurationDiagnostic, GlobalConfiguration, get_unknown_property_diagnostics,
    get_value,
};
use dprint_core::plugins::process::{
    get_parent_process_id_from_cli_args, handle_process_stdio_messages,
    start_parent_process_checker_task,
};
use dprint_core::plugins::{
    AsyncPluginHandler, FileMatchingInfo, FormatRequest, FormatResult, HostFormatRequest,
    PluginInfo, PluginResolveConfigurationResult,
};
use nuparu_core::{Configuration, format_text};

struct NuPluginHandler;

#[async_trait(?Send)]
impl AsyncPluginHandler for NuPluginHandler {
    type Configuration = Configuration;

    fn plugin_info(&self) -> PluginInfo {
        PluginInfo {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            config_key: "nushell".to_string(),
            help_url: "https://www.nushell.sh/book/".to_string(),
            config_schema_url: String::new(),
            update_url: None,
        }
    }

    fn license_text(&self) -> String {
        format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    }

    async fn resolve_config(
        &self,
        mut config: ConfigKeyMap,
        global_config: GlobalConfiguration,
    ) -> PluginResolveConfigurationResult<Configuration> {
        let mut diagnostics: Vec<ConfigurationDiagnostic> = Vec::new();
        let indent_width = get_value(
            &mut config,
            "indentWidth",
            global_config.indent_width.unwrap_or(2),
            &mut diagnostics,
        );
        let max_blank_lines = get_value(&mut config, "maxBlankLines", 1u8, &mut diagnostics);
        let line_width = get_value(
            &mut config,
            "lineWidth",
            global_config.line_width.unwrap_or(80) as u16,
            &mut diagnostics,
        );
        diagnostics.extend(get_unknown_property_diagnostics(config));

        PluginResolveConfigurationResult {
            file_matching: FileMatchingInfo {
                file_extensions: vec!["nu".to_string()],
                file_names: vec![],
            },
            config: Configuration {
                indent_width,
                max_blank_lines,
                line_width,
            },
            diagnostics,
        }
    }

    async fn format(
        &self,
        request: FormatRequest<Self::Configuration>,
        _format_with_host: impl FnMut(HostFormatRequest) -> LocalBoxFuture<'static, FormatResult>
        + 'static,
    ) -> FormatResult {
        if request.range.is_some() {
            return Ok(None);
        }

        let file_text = String::from_utf8(request.file_bytes.to_vec())?;
        let formatted = format_text(&file_text, &request.config);
        let formatted = if file_text.contains("\r\n") {
            formatted.replace('\n', "\r\n")
        } else {
            formatted
        };

        if formatted == file_text {
            Ok(None)
        } else {
            Ok(Some(formatted.into_bytes()))
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    if let Some(parent_process_id) = get_parent_process_id_from_cli_args() {
        start_parent_process_checker_task(parent_process_id);
        return handle_process_stdio_messages(NuPluginHandler).await;
    }

    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    print!("{}", format_text(&buffer, &Configuration::default()));
    Ok(())
}

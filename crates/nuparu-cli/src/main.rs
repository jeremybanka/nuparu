use std::io::{self, Read};
use std::path::Path;

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

struct CliRun {
    exit_code: i32,
    stdout: String,
    stderr: String,
}

fn run_cli_with_formatter(
    args: &[String],
    stdin: &str,
    format: impl Fn(&str) -> String,
) -> Result<CliRun> {
    if args.is_empty() {
        return Ok(CliRun {
            exit_code: 0,
            stdout: format(stdin),
            stderr: String::new(),
        });
    }

    if matches!(args.first().map(String::as_str), Some("--write" | "-w")) {
        let file_paths = &args[1..];
        if file_paths.is_empty() {
            return Ok(CliRun {
                exit_code: 1,
                stdout: String::new(),
                stderr: "nuparu --write requires at least one file path.\n".to_string(),
            });
        }

        for file_path in file_paths {
            rewrite_file_preserving_permissions(Path::new(file_path), &format)?;
        }

        return Ok(CliRun {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        });
    }

    Ok(CliRun {
        exit_code: 1,
        stdout: String::new(),
        stderr: "nuparu does not support command-line arguments yet.\n".to_string(),
    })
}

fn rewrite_file_preserving_permissions(path: &Path, format: impl Fn(&str) -> String) -> Result<()> {
    let file_text = std::fs::read_to_string(path)?;
    let formatted = format(&file_text);
    if formatted == file_text {
        return Ok(());
    }

    let permissions = std::fs::metadata(path)?.permissions();
    std::fs::write(path, formatted)?;
    std::fs::set_permissions(path, permissions)?;
    Ok(())
}

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

    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let mut buffer = String::new();
    if args.is_empty() {
        io::stdin().read_to_string(&mut buffer)?;
    }
    let run = run_cli_with_formatter(&args, &buffer, |input| {
        format_text(input, &Configuration::default())
    })?;

    if !run.stdout.is_empty() {
        print!("{}", run.stdout);
    }
    if !run.stderr.is_empty() {
        eprint!("{}", run.stderr);
    }

    if run.exit_code != 0 {
        std::process::exit(run.exit_code);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::run_cli_with_formatter;

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn formats_stdin_when_no_arguments_are_provided() {
        let run = run_cli_with_formatter(&[], "echo hi", |_| "formatted".to_string()).unwrap();
        assert_eq!(run.exit_code, 0);
        assert_eq!(run.stdout, "formatted");
        assert_eq!(run.stderr, "");
    }

    #[cfg(unix)]
    #[test]
    fn preserves_executable_mode_when_rewriting_real_update_fixture_shape() {
        let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/dotfiles/scripts/update.nu");
        let fixture_text = fs::read_to_string(&fixture_path).unwrap();
        let temp_path = unique_temp_path("nuparu-cli-update-mode");

        fs::write(&temp_path, &fixture_text).unwrap();
        let permissions = fs::Permissions::from_mode(0o755);
        fs::set_permissions(&temp_path, permissions).unwrap();

        let args = vec![
            "--write".to_string(),
            temp_path.to_string_lossy().into_owned(),
        ];
        let run = run_cli_with_formatter(&args, "", |input| format!("{input}\n")).unwrap();

        assert_eq!(run.exit_code, 0);
        assert_eq!(run.stdout, "");
        assert_eq!(run.stderr, "");
        let mode = fs::metadata(&temp_path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o755);

        let _ = fs::remove_file(&temp_path);
    }

    fn unique_temp_path(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{nanos}.nu"))
    }
}

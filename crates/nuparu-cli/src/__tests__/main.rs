use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use super::super::run_cli_with_formatter;

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
    let fixture_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/dotfiles/scripts/update.nu");
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

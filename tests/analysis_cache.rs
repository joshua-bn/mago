//! End-to-end tests for the persistent analyzer result cache.

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

fn mago_bin() -> PathBuf {
    std::env::var("CARGO_BIN_EXE_mago")
        .ok()
        .or_else(|| option_env!("CARGO_BIN_EXE_mago").map(String::from))
        .unwrap_or_else(|| "mago".to_owned())
        .into()
}

fn can_run_mago() -> bool {
    Command::new(mago_bin()).arg("--help").output().is_ok_and(|output| output.status.success())
}

fn analyze(workspace: &Path, cache: &Path) -> std::process::Output {
    Command::new(mago_bin())
        .args([
            "--workspace",
            workspace.to_str().unwrap(),
            "analyze",
            "--cache",
            cache.to_str().unwrap(),
            "--reporting-format",
            "count",
        ])
        .env("MAGO_LOG", "info")
        .output()
        .expect("failed to run mago")
}

fn combined_output(output: &std::process::Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr))
}

#[test]
fn reuses_unchanged_results_and_invalidates_source_changes() {
    if !can_run_mago() {
        return;
    }

    let temporary = tempfile::tempdir().unwrap();
    let workspace = temporary.path();
    let source_directory = workspace.join("src");
    let cache_directory = workspace.join("cache");
    std::fs::create_dir(&source_directory).unwrap();
    std::fs::write(workspace.join("mago.toml"), "php-version = \"8.4\"\n[source]\npaths = [\"src\"]\n[analyzer]\n")
        .unwrap();
    std::fs::write(source_directory.join("example.php"), "<?php\nfunction example(): int { return 'invalid'; }\n")
        .unwrap();

    let cold = analyze(workspace, &cache_directory);
    assert!(!cold.status.success());
    assert!(combined_output(&cold).contains("Analysis cache miss"));
    assert!(cache_directory.join("analyze.bin").is_file());

    let cached = analyze(workspace, &cache_directory);
    assert!(!cached.status.success());
    assert!(combined_output(&cached).contains("Analysis cache hit"));

    std::fs::write(source_directory.join("example.php"), "<?php\nfunction example(): int { return 1; }\n").unwrap();
    let invalidated = analyze(workspace, &cache_directory);
    assert!(invalidated.status.success());
    assert!(combined_output(&invalidated).contains("Analysis cache miss"));
}

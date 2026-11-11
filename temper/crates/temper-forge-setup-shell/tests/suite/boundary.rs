use std::{fs, path::Path};
fn forbidden(source: &str) -> bool {
    let compact: String = source.chars().filter(|c| !c.is_whitespace()).collect();
    // Calls through the setup's process boundary are not std process operations
    // (temper/docs/design/protocol/setup/README.md, 7).
    let compact = compact.replace("temper_forge_setup_shell::process::", "ForgeProcess::");
    compact.contains("std::process")
        || compact.contains("process::") && compact.contains("usestd::{")
        || ["CommandExt", "try_lock", "lock_shared", ".lock()"].iter().any(|name| compact.contains(name))
}
fn forbidden_at(path: &Path, source: &str) -> bool {
    let source = if path.ends_with("temper-box-shell/src/binary.rs")
        || path.ends_with("temper-client/src/main.rs")
        || path.ends_with("temper-client-shell/src/lib.rs")
        || path.ends_with("temper-engine/main.rs")
        || path.ends_with("temper-engine-shell/src/binary.rs")
        || path.ends_with("temper-agent/main.rs")
        || path.ends_with("temper-agent-shell/src/entry.rs")
        || path.ends_with("temper-worker-shell/src/binary.rs")
    {
        source.replace("std::process::ExitCode", "ExitCode").replace("process::ExitCode", "ExitCode")
    } else if path.ends_with("temper-service-shell/src/lib.rs") {
        // Serve's run reads the first link's secret from stdin (protocol/setup/box.md, 1).
        // Its stdin guard is not a file lock; file locks and process operations stay guarded.
        source.replace("std::io::stdin().lock()", "BoxLinkInput")
    } else if path.ends_with("temper-worker-shell/src/program_login.rs") {
        // Terminal sign-in execs smith's command (protocol/credentials.md, 3).
        // Other process types and every lock operation retain the same guard.
        source.replace("std::process::Command", "TerminalCommand").replace("process::CommandExt", "TerminalExec")
    } else if path.ends_with("temper-fetch-shell/src/fetch.rs") {
        // Claude Code's missing build is fetched under an exclusive file lock
        // (protocol/setup/owned.md, 6); process operations stay in process.rs.
        source.replace(".lock()", ".fetch_lock()")
    } else if path.ends_with("temper-launcher/src/lib.rs") {
        // The launcher execs the pinned build and links the running executable;
        // its process id names concurrent link replacements (protocol/setup/owned.md, 6).
        source
            .replace("std::process::Command", "LauncherCommand")
            .replace("process::CommandExt", "LauncherExec")
            .replace("std::process::current_exe", "LauncherExecutable")
            .replace("std::process::id", "LauncherId")
    } else if path.ends_with("temper-service-shell/src/restart_fault.rs") {
        // The process tier's exact-listener fault aborts its own serve only.
        // Process creation, exec and every lock still belong to their boundaries.
        source.replace("std::process::abort", "RestartFaultAbort")
    } else {
        source.into()
    };
    forbidden(&source)
}
fn scan(directory: &Path) {
    for entry in fs::read_dir(directory).expect("source directory") {
        let path = entry.expect("source entry").path();
        if path.file_name().is_some_and(|name| name == "tests" || name == "tests.rs") {
            continue;
        }
        if path.is_dir() {
            scan(&path);
        } else if path.extension().is_some_and(|name| name == "rs") {
            if path.ends_with("temper-forge-setup-shell/src/process.rs") {
                continue;
            }
            let source = fs::read_to_string(&path).expect("Rust source");
            assert!(!forbidden_at(&path, &source), "process or lock outside its boundary: {}", path.display());
        }
    }
}
#[test]
fn process_and_file_locks_stay_in_their_declared_modules() {
    let crates = skein_scratch::paths::tree().join("crates");
    for entry in fs::read_dir(crates).expect("crates") {
        let source = entry.expect("crate").path().join("src");
        if source.is_dir() {
            scan(&source);
        }
    }
}
#[test]
fn calls_through_the_process_boundary_do_not_exempt_direct_operations() {
    for (source, expected) in [
        ("use std::{fs}; temper_forge_setup_shell :: process :: child()", false),
        ("use std::{fs}; temper_forge_setup_shell::process::child(); std::process::Command", true),
        ("use std::{fs, process::Child}; temper_forge_setup_shell::process::child()", true),
        ("use std::{fs}; temper_forge_setup_shell::process::child(); file.lock()", true),
        ("use std::{fs}; temper_forge_setup_shell::process::child(); file.try_lock()", true),
        ("use std::{fs}; temper_forge_setup_shell::process::child(); file.lock_shared()", true),
        ("use std::{fs}; temper_forge_setup_shell::process::child(); CommandExt", true),
    ] {
        assert_eq!(forbidden(source), expected, "{source}");
    }
}
#[test]
fn binary_exit_codes_do_not_exempt_processes_or_file_locks() {
    for path in [
        "temper-box-shell/src/binary.rs",
        "temper-client/src/main.rs",
        "temper-client-shell/src/lib.rs",
        "temper-engine/main.rs",
        "temper-engine-shell/src/binary.rs",
        "temper-agent/main.rs",
        "temper-agent-shell/src/entry.rs",
        "temper-worker-shell/src/binary.rs",
    ] {
        for (source, expected) in [
            ("use std::process::ExitCode;", false),
            ("use std::{process::ExitCode};", false),
            ("std::process::Command", true),
            ("std::process::Child", true),
            ("std::process::exit(1)", true),
            ("try_lock", true),
            ("file.lock()", true),
            ("use std::process::ExitCode; std::process::Command", true),
            ("use std::process::ExitCode; file.lock()", true),
        ] {
            assert_eq!(forbidden_at(Path::new(path), source), expected, "{path}: {source}");
        }
    }
}
#[test]
fn serve_stdin_guard_does_not_exempt_file_locks_or_processes() {
    let path = Path::new("temper-service-shell/src/lib.rs");
    for (source, expected) in [
        ("std::io::stdin().lock()", false),
        ("file.lock()", true),
        ("file.try_lock()", true),
        ("std::process::Command", true),
        ("std::io::stdin().lock(); file.lock()", true),
        ("std::io::stdin().lock(); std::process::Command", true),
    ] {
        assert_eq!(forbidden_at(path, source), expected, "{source}");
    }
}
#[test]
fn the_scan_catches_each_form_and_passes_other_process_modules() {
    for (path, source, expected) in [
        ("other.rs", "use std::{fs, process::Child};", true),
        ("other.rs", "std::process::exit(1)", true),
        ("temper-box-shell/src/lib.rs", "std::io::stdin().lock()", true),
        ("temper-service-shell/src/host.rs", "std::io::stdin().lock()", true),
        ("other.rs", "CommandExt", true),
        ("other.rs", "try_lock", true),
        ("other.rs", "smith::process::Event", false),
        ("temper-worker-shell/src/program_login.rs", "std::process::Command", false),
        ("temper-worker-shell/src/program_login.rs", "use std::os::unix::process::CommandExt;", false),
        ("temper-worker-shell/src/program_login.rs", "std::process::Child", true),
        ("temper-worker-shell/src/program_login.rs", "std::process::exit(1)", true),
        ("temper-worker-shell/src/program_login.rs", "try_lock", true),
        ("temper-fetch-shell/src/fetch.rs", "file.lock()", false),
        ("temper-fetch-shell/src/fetch.rs", "try_lock", true),
        ("temper-fetch-shell/src/fetch.rs", "std::process::Command", true),
        ("temper-launcher/src/lib.rs", "std::process::Command", false),
        ("temper-launcher/src/lib.rs", "use std::os::unix::process::CommandExt;", false),
        ("temper-launcher/src/lib.rs", "std::process::current_exe()", false),
        ("temper-launcher/src/lib.rs", "std::process::id()", false),
        ("temper-launcher/src/lib.rs", "std::process::Child", true),
        ("temper-launcher/src/lib.rs", "std::process::exit(1)", true),
        ("temper-launcher/src/lib.rs", "file.lock()", true),
        ("other.rs", "file.lock()", true),
        ("temper-box-shell/src/host.rs", "std::process::abort()", true),
        ("temper-box-shell/src/restart_fault.rs", "std::process::abort()", true),
        ("temper-service-shell/src/restart_fault.rs", "std::process::abort()", false),
        ("temper-service-shell/src/restart_fault.rs", "std::process::Command", true),
        ("temper-service-shell/src/restart_fault.rs", "try_lock", true),
        ("temper-service-shell/src/restart_fault.rs", "file.lock()", true),
        ("other.rs", "std::process::Command", true),
    ] {
        assert_eq!(forbidden_at(Path::new(path), source), expected, "{path}: {source}");
    }
}

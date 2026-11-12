//! Builds the browser's bundle and writes the list `src/lib.rs` includes (client/web/README.md, 2.6), unless
//! `TEMPER_INTERNAL_BUNDLE` is `none`.
use std::{
    env, fs,
    path::PathBuf,
    process::{Command, ExitCode, Stdio},
};

mod steps;

#[expect(
    clippy::disallowed_methods,
    reason = "build script: Cargo's CARGO_MANIFEST_DIR, PROFILE, RUSTC and CARGO configure the browser build"
)]
fn embed(out: &std::path::Path) -> Result<String, String> {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("cargo sets CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(|why| why.to_string())?;
    let profile = env::var("PROFILE").expect("cargo sets PROFILE");
    let profile = steps::profile(&profile)?;
    let target = steps::target_dir(out).ok_or("temper-client-bundle: OUT_DIR has no cargo target ancestor")?;
    let rustc = env::var_os("RUSTC").expect("cargo sets RUSTC");
    let check = Command::new(rustc)
        .args(["--print", "target-libdir", "--target", "wasm32-unknown-unknown"])
        .stderr(Stdio::null())
        .output();
    match check {
        Ok(check) => steps::wasm_target(check.status.success(), &check.stdout)?,
        Err(_) => return Err(steps::NEED_WASM.into()),
    }
    let cargo = env::var_os("CARGO").expect("cargo sets CARGO");
    let commands = steps::commands(&cargo, &root, &target, out, profile);
    for (step, description) in ["browser module", "bundler build", "bundling"].into_iter().zip(commands) {
        if step == "bundling" && out.join("bundle").exists() {
            fs::remove_dir_all(out.join("bundle"))
                .map_err(|why| format!("temper-client-bundle: removing previous bundle: {why}"))?;
        }
        let mut command = Command::new(description.program);
        command.args(description.args).current_dir(description.directory).stdout(Stdio::from(std::io::stderr()));
        for name in description.remove {
            command.env_remove(name);
        }
        for (name, value) in description.set {
            command.env(name, value);
        }
        let status = command.status().map_err(|why| format!("temper-client-bundle: {step} failed: {why}"))?;
        if !status.success() {
            return Err(format!("temper-client-bundle: {step} failed: {status}"));
        }
    }
    let dep = steps::module_dep_info(&target, profile.1);
    let dep = fs::read_to_string(dep).map_err(|why| format!("temper-client-bundle: reading module dep-info: {why}"))?;
    let workspace = root.parent().ok_or("temper-client-bundle: temper's tree has no workspace above it")?;
    for path in steps::rerun_paths(workspace, &root, &steps::dep_info(&dep)?) {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    Ok(steps::source(&steps::bundle(&out.join("bundle"))?))
}

#[expect(
    clippy::disallowed_methods,
    reason = "build script: TEMPER_INTERNAL_BUNDLE selects the bundle and Cargo's OUT_DIR receives its source"
)]
fn main() -> ExitCode {
    println!("cargo:rerun-if-env-changed=TEMPER_INTERNAL_BUNDLE");
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("cargo sets OUT_DIR"));
    let list = match steps::choice(env::var_os("TEMPER_INTERNAL_BUNDLE").as_deref()) {
        Ok(steps::Choice::None) => Ok(steps::NO_BUNDLE.into()),
        Ok(steps::Choice::Embed) => embed(&out),
        Err(why) => Err(why),
    };
    match list.and_then(|list| {
        fs::write(out.join("bundle.rs"), list)
            .map_err(|why| format!("temper-client-bundle: writing bundle list: {why}"))
    }) {
        Ok(()) => ExitCode::SUCCESS,
        Err(why) => {
            eprintln!("{why}");
            ExitCode::FAILURE
        }
    }
}

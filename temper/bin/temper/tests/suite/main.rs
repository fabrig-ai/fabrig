//! The client's binary wiring (temper/docs/design/cli/README.md, 7.4 and 10).
#![forbid(unsafe_code)]

mod box_binary;
mod help;

fn output(arguments: &[&str]) -> std::process::Output {
    let scratch = skein_scratch::Scratch::new("client-binary");
    std::process::Command::new(skein_scratch::paths::binary("temper"))
        .args(arguments)
        .env_clear()
        .env("TEMPER_CONFIG", scratch.path().join("config"))
        .env("TEMPER_DATA", scratch.path().join("data"))
        .env("TEMPER_CACHE", scratch.path().join("cache"))
        .current_dir(scratch.path())
        .stdin(std::process::Stdio::null())
        .output()
        .expect("run temper")
}

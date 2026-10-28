use std::path::PathBuf;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    skein_fail_stop::install();
    let mut git = None;
    let mut scratch = None;
    let mut out = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let value = args.next().ok_or("expected --git <path> --scratch <dir> --out <dir>")?;
        match arg.as_str() {
            "--git" => git = Some(PathBuf::from(value)),
            "--scratch" => scratch = Some(PathBuf::from(value)),
            "--out" => out = Some(PathBuf::from(value)),
            _ => return Err("expected --git <path> --scratch <dir> --out <dir>".into()),
        }
    }
    git_transcripts::run(
        &git.ok_or("--git required")?,
        &scratch.ok_or("--scratch required")?,
        &out.ok_or("--out required")?,
    )
}

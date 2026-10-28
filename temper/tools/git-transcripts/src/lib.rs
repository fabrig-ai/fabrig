//! Real git's arguments and output, recorded without credentials (protocol/git.md, 9).
mod cases;
mod invocation;
pub mod mask;
use skein_throwaway_forgejo::{Forgejo, Settings};
use std::{
    error::Error,
    fs,
    io::Write,
    os::unix::process::ExitStatusExt,
    path::{Path, PathBuf},
    process::Stdio,
};
type Result<T> = std::result::Result<T, Box<dyn Error>>;
pub(crate) struct Capture {
    git: PathBuf,
    home: PathBuf,
    out: PathBuf,
    mask: mask::Mask,
    secrets: Vec<String>,
}
impl Capture {
    fn execute(
        &mut self,
        case: Option<&str>,
        dir: &Path,
        args: &[&str],
        identity: Option<(&str, &str)>,
        commit: bool,
        input: &[u8],
    ) -> Result<std::process::Output> {
        let mut cmd = invocation::command(&self.git, &self.home, dir, identity, commit);
        cmd.args(args).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
        let arguments =
            format!("git\n{}\n", cmd.get_args().map(|a| a.to_string_lossy()).collect::<Vec<_>>().join("\n"));
        let mut child = cmd.spawn()?;
        child.stdin.take().ok_or("git stdin missing")?.write_all(input)?;
        let output = child.wait_with_output()?;
        if let Some(case) = case {
            let exit = output
                .status
                .code()
                .map_or_else(|| format!("signal {}\n", output.status.signal().unwrap_or(0)), |n| format!("{n}\n"));
            let files = [
                ("argv", arguments.as_bytes()),
                ("exit", exit.as_bytes()),
                ("stdout", &output.stdout),
                ("stderr", &output.stderr),
            ];
            for (_, bytes) in files {
                self.scan(case, bytes)?;
            }
            let dir = self.out.join("transcripts").join(case);
            fs::create_dir_all(&dir)?;
            for (name, bytes) in files {
                let masked = self.mask.bytes(bytes);
                self.scan(case, &masked)?;
                fs::write(dir.join(name), masked)?;
            }
        }
        Ok(output)
    }
    fn scan(&self, case: &str, bytes: &[u8]) -> Result<()> {
        if self.secrets.iter().any(|s| !s.is_empty() && bytes.windows(s.len()).any(|w| w == s.as_bytes())) {
            return Err(format!("{case}: a credential reached git's output").into());
        }
        Ok(())
    }
}
/// # Errors
/// Refuses an old or relative git, a provisioning failure or an output containing credentials.
pub fn run(git: &Path, scratch: &Path, out: &Path) -> Result<()> {
    if !git.is_absolute() {
        return Err("--git must be an absolute path".into());
    }
    let binary = std::env::var_os("TEMPER_FORGEJO").ok_or("TEMPER_FORGEJO must name the pinned Forgejo")?;
    let forgejo = Forgejo::from_path(Path::new(&binary)).map_err(|e| format!("{e:?}"))?;
    let mut settings = Settings::small();
    settings.actions = false;
    settings.git = git.into();
    let instance = forgejo.start(scratch, &settings).map_err(|e| format!("{e:?}"))?;
    let scratch = instance.scratch();
    let home = scratch.join("worker/home");
    fs::create_dir_all(&home)?;
    fs::set_permissions(&home, std::os::unix::fs::PermissionsExt::from_mode(0o700))?;
    let mut capture = Capture {
        git: git.into(),
        home,
        out: out.into(),
        mask: mask::Mask::new(scratch, instance.port()),
        secrets: instance.secrets(),
    };
    cases::record(&forgejo, instance, &mut capture)
}
#[cfg(test)]
mod tests;

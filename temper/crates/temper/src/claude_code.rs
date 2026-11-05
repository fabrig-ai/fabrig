//! Claude Code's launcher (protocol/setup.md, 6.6).
use std::{
    ffi::OsStr,
    fs,
    os::unix::{
        ffi::OsStrExt,
        fs::{DirBuilderExt, symlink},
        process::CommandExt,
    },
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};
use temper_forge_setup::{
    claude_code::{self, Libc},
    setup::Origin,
};

/// A link named `claude` selects the launcher before temper reads its flags.
#[must_use]
pub fn is_launcher(program: &OsStr) -> bool {
    Path::new(program).file_name() == Some(OsStr::new("claude"))
}

/// Fetch the pinned build if missing and replace this process, preserving argv.
#[must_use]
pub fn run(program: &OsStr, arguments: &[Box<[u8]>]) -> u8 {
    let why = launch(program, arguments).expect_err("exec only returns on failure");
    eprintln!("temper: claude: {why}");
    1
}
fn launch(program: &OsStr, arguments: &[Box<[u8]>]) -> Result<(), String> {
    let directory = Path::new(program).parent().ok_or("launcher has no parent")?;
    let cache = fs::read_link(directory.join("cache")).map_err(|why| format!("cache: {why}"))?;
    let build = claude_code::build(
        if cfg!(target_os = "macos") {
            skein_forge::forgejo::pin::Platform::Macos
        } else {
            skein_forge::forgejo::pin::Platform::Linux
        },
        std::env::consts::ARCH,
        if cfg!(target_env = "musl") { Libc::Musl } else { Libc::Gnu },
    )
    .map_err(|why| String::from_utf8_lossy(&why.words()).into_owned())?;
    let pin = claude_code::pin(
        cache.as_os_str().as_bytes(),
        Origin { host: b"downloads.claude.ai".as_slice().into(), port: 443 },
        build,
    );
    let file = claude_code::file(cache.as_os_str().as_bytes());
    temper_forge_setup_shell::fetch::claude_code(
        &jig_protocol_config::profile(jig_protocol_config::Profile::Small).io,
        &skein_shell::Trust::Machine { ssl_cert_file: std::env::var_os("SSL_CERT_FILE").map(Into::into) },
        super::path(&file),
        pin,
        build.bytes,
        &mut std::io::stderr(),
    )
    .map_err(|why| why.words())?;
    let why = std::process::Command::new(super::path(&file))
        .args(arguments.iter().map(|argument| OsStr::from_bytes(argument)))
        .env("DISABLE_AUTOUPDATER", "1")
        .exec();
    Err(format!("{}: {why}", super::path(&file).display()))
}

pub(crate) fn place(directory: &Path, cache: &Path) -> Result<(), String> {
    let why = |why| format!("{}: {why}", directory.display());
    if !smith_agent_shell::program::private_directory(directory).map_err(|error| error.to_string())? {
        match fs::DirBuilder::new().recursive(true).mode(0o700).create(directory) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(why(error)),
        }
    }
    smith_agent_shell::program::private_directory(directory).map_err(|error| error.to_string())?;
    let executable = std::env::current_exe().map_err(why)?;
    link(directory, "claude", &executable)?;
    link(directory, "cache", cache)
}

fn link(directory: &Path, name: &str, target: &Path) -> Result<(), String> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let path = directory.join(name);
    if fs::read_link(&path).ok().as_deref() == Some(target) {
        return Ok(());
    }
    let part = directory.join(format!(".{name}.{}.{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    symlink(target, &part).map_err(|why| format!("{}: {why}", path.display()))?;
    if let Err(why) = fs::rename(&part, &path) {
        match fs::remove_file(&part) {
            Ok(()) | Err(_) => {}
        }
        return Err(format!("{}: {why}", path.display()));
    }
    Ok(())
}

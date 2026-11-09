//! Claude Code's launcher (protocol/setup.md, 6.6).
use std::{
    ffi::OsStr,
    fs,
    os::unix::{
        ffi::OsStrExt,
        fs::{DirBuilderExt, symlink},
        process::CommandExt,
    },
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use temper_fetch::{
    Origin,
    claude_code::{self, Libc},
};

/// A link named `claude` selects the launcher before temper reads its flags.
#[must_use]
pub fn is_launcher(program: &OsStr) -> bool {
    Path::new(program).file_name() == Some(OsStr::new("claude"))
}

/// Fetch the pinned build if missing and replace this process, preserving argv.
#[must_use]
pub fn run(name: &str, program: &OsStr, arguments: &[Box<[u8]>]) -> u8 {
    let why = launch(program, arguments).expect_err("exec only returns on failure");
    eprintln!("{name}: claude: {why}");
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
    crate::fetch::claude_code(
        &IO,
        &skein_shell::Trust::Machine { ssl_cert_file: std::env::var_os("SSL_CERT_FILE").map(Into::into) },
        path(&file),
        pin,
        build.bytes,
        &mut std::io::stderr(),
    )
    .map_err(|why| why.words())?;
    let why = std::process::Command::new(path(&file))
        .args(arguments.iter().map(|argument| OsStr::from_bytes(argument)))
        .env("DISABLE_AUTOUPDATER", "1")
        .exec();
    Err(format!("{}: {why}", path(&file).display()))
}

pub fn place(directory: &Path, cache: &Path) -> Result<(), String> {
    let why = |why| format!("{}: {why}", directory.display());
    if !smith_agent_document::program::private_directory(directory).map_err(|error| error.to_string())? {
        match fs::DirBuilder::new().recursive(true).mode(0o700).create(directory) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(why(error)),
        }
    }
    smith_agent_document::program::private_directory(directory).map_err(|error| error.to_string())?;
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

/// The launcher fetch's small io limits (protocol/setup.md, 6.6).
pub const IO: skein_io::Limits = skein_io::Limits {
    sockets: 16,
    names: 8,
    refusals: 8,
    intake: 1_048_576,
    receive: 16_384,
    output: 1_048_576,
    sends: 8,
    accepts: 4,
    backlog: 16,
    close_timeout: skein_lib::Duration::from_secs(30),
    retry: skein_lib::Duration::from_millis(250),
};

fn path(bytes: &[u8]) -> &Path {
    Path::new(OsStr::from_bytes(bytes))
}

/// Claude Code's launcher, beside its programs directory (protocol/setup.md, 6.6).
#[must_use]
pub fn beside(directory: &Path) -> Option<PathBuf> {
    directory.parent().map(|parent| parent.join("claude-code/claude"))
}

/// A program `"fetched"` names its launcher in the document smith decodes.
pub fn substituted(bytes: &[u8]) -> Result<Option<Box<[u8]>>, String> {
    use skein_json::{document, tokenizer};
    // Inspect the bounded document first; smith keeps its own diagnostics for other documents.
    let Ok(length) = u32::try_from(bytes.len()) else { return Ok(None) };
    if length > 1 << 20 {
        return Ok(None);
    }
    let limits = tokenizer::Limits { depth: 32, string: length, number: length, chunk: length.max(1), length };
    let retained = document::Limits { tokens: 65_536, text: 1 << 20 };
    let Ok(document) = skein_json::parse(bytes, &limits, &retained) else { return Ok(None) };
    let program = document.member(0, b"programs").and_then(|programs| document.member(programs, b"claude_code"));
    let at = program.and_then(|program| document.member(program, b"path"));
    let fetched = at
        .and_then(|at| document.token(at))
        .filter(|token| token.kind == document::Kind::String)
        .and_then(|token| document.text(token))
        == Some(claude_code::FETCHED);
    if !fetched {
        return Ok(None);
    }
    let directory = program
        .and_then(|program| document.member(program, b"directory"))
        .and_then(|at| document.token(at))
        .ok_or("programs.claude_code.directory: programs/claude_code/directory: missing or wrong kind")?;
    if directory.kind != document::Kind::String {
        return Err("programs.claude_code.directory: programs/claude_code/directory: expected text".into());
    }
    let directory = document.text(directory).expect("retained text");
    let launcher =
        beside(path(directory)).ok_or("programs.claude_code.directory: expected a directory with a parent")?;
    let at = at.expect("fetched path");
    let limits = skein_json::writer::Limits { depth: 32, length: 1 << 20 };
    let mut measure = skein_json::writer::Encoder::measure(&limits);
    substituted_document(&mut measure, &document, at, launcher.as_os_str().as_bytes());
    let length = measure.measured().map_err(|_| "public document: past byte bound")?;
    let mut writer = skein_json::writer::Encoder::write(length, &limits);
    substituted_document(&mut writer, &document, at, launcher.as_os_str().as_bytes());
    Ok(Some(writer.finish()))
}

fn substituted_document(
    writer: &mut skein_json::writer::Encoder,
    document: &skein_json::Document,
    at: u32,
    launcher: &[u8],
) {
    use skein_json::Kind;
    for index in 0..document.len() {
        let record = document.token(index).expect("within the retained record count");
        let text = document.text(record).expect("retained text");
        match record.kind {
            Kind::ObjectStart => writer.object_start(),
            Kind::ObjectEnd => writer.object_end(),
            Kind::ArrayStart => writer.array_start(),
            Kind::ArrayEnd => writer.array_end(),
            Kind::Key => writer.key(text),
            Kind::String => writer.string(if index == at { launcher } else { text }),
            Kind::Number => writer.number(text),
            Kind::True => writer.boolean(true),
            Kind::False => writer.boolean(false),
            Kind::Null => writer.null(),
            Kind::Long => unreachable!("Long has no text to write"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn document(program: &serde_json::Value) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({"programs":{"claude_code":program}})).unwrap()
    }
    #[test]
    fn fetched_names_the_launcher_beside_the_programs_directory() {
        let bytes = document(&serde_json::json!({"path":"fetched","directory":"/data/programs"}));
        let bytes = substituted(&bytes).unwrap().unwrap();
        let expected = document(&serde_json::json!({"path":"/data/claude-code/claude","directory":"/data/programs"}));
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&bytes).unwrap(),
            serde_json::from_slice::<serde_json::Value>(&expected).unwrap()
        );
    }
    #[test]
    fn an_absolute_program_document_goes_to_smith_byte_for_byte() {
        let bytes =
            b"{ \"programs\": {\"claude_code\": {\"path\": \"/bin/claude\", \"directory\": \"/data/programs\"}}}\n";
        assert_eq!(substituted(bytes).unwrap(), None);
    }
    #[test]
    fn fetched_requires_a_programs_directory_with_a_parent() {
        for program in [serde_json::json!({"path":"fetched"}), serde_json::json!({"path":"fetched","directory":"/"})] {
            let why = substituted(&document(&program)).unwrap_err();
            assert!(why.contains("programs.claude_code.directory"), "{why}");
        }
    }
}

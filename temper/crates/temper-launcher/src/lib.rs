//! Claude Code's launcher (temper/docs/design/protocol/setup/owned.md, 6).
use std::{
    ffi::OsStr,
    fs, io,
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
use temper_shell::directories::EnvironmentEntry;

/// A link named `claude` selects the launcher before temper reads its flags.
#[must_use]
pub fn is_launcher(program: &OsStr) -> bool {
    Path::new(program).file_name() == Some(OsStr::new("claude"))
}

/// Fetch the pinned build if missing and replace this process, preserving argv.
#[must_use]
pub fn run(name: &str, program: &OsStr, arguments: &[Box<[u8]>], environment: &[EnvironmentEntry]) -> u8 {
    let mut stderr = io::stderr();
    let why = launch(program, arguments, environment, &mut stderr).expect_err("exec only returns on failure");
    refusal(name, &why, &mut stderr)
}

fn refusal(name: &str, why: &str, out: &mut dyn io::Write) -> u8 {
    let (code, line) = launch_refusal(name, why);
    out.write_all(line.as_bytes()).expect("write launcher refusal to stderr");
    code
}

fn launch_refusal(name: &str, why: &str) -> (u8, String) {
    (1, format!("{name}: claude: {why}\n"))
}

fn launch(
    program: &OsStr,
    arguments: &[Box<[u8]>],
    environment: &[EnvironmentEntry],
    out: &mut dyn io::Write,
) -> Result<(), String> {
    let mut command = launch_command_with(
        program,
        arguments,
        |cache| fs::read_link(cache),
        |file, pin, bytes| temper_fetch_shell::fetch::claude_code(&IO, &trust(environment), file, pin, bytes, out),
    )?;
    let why = command.exec();
    Err(format!("{}: {why}", Path::new(command.get_program()).display()))
}

fn launch_command_with(
    program: &OsStr,
    arguments: &[Box<[u8]>],
    read_cache: impl FnOnce(&Path) -> io::Result<PathBuf>,
    fetch: impl FnOnce(&Path, temper_fetch::fetch::Pin, u64) -> Result<(), temper_fetch_shell::fetch::Failed>,
) -> Result<std::process::Command, String> {
    let directory = Path::new(program).parent().ok_or("launcher has no parent")?;
    let cache = read_cache(&directory.join("cache")).map_err(|why| format!("cache: {why}"))?;
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
    fetch(path(&file), pin, build.bytes).map_err(|why| why.words())?;
    Ok(command(path(&file), arguments))
}

fn command(file: &Path, arguments: &[Box<[u8]>]) -> std::process::Command {
    let mut command = std::process::Command::new(file);
    command
        .args(arguments.iter().map(|argument| OsStr::from_bytes(argument)))
        .env("DISABLE_AUTOUPDATER", "1")
        .env("DISABLE_UPDATES", "1");
    command
}

/// Write the launcher's links in `directory`: `claude` to the agent's program, `cache` to the cache (protocol/setup/owned.md, 6).
pub fn place(directory: &Path, cache: &Path, program: &Path) -> Result<(), String> {
    let why = |why| format!("{}: {why}", directory.display());
    if !smith_agent_document::program::private_directory(directory).map_err(|error| error.to_string())? {
        match fs::DirBuilder::new().recursive(true).mode(0o700).create(directory) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(why(error)),
        }
    }
    smith_agent_document::program::private_directory(directory).map_err(|error| error.to_string())?;
    place_links(directory, cache, program, &mut DiskLinks)
}

fn place_links(directory: &Path, cache: &Path, program: &Path, links: &mut impl Links) -> Result<(), String> {
    link(directory, "claude", program, links)?;
    link(directory, "cache", cache, links)
}

trait Links {
    fn read(&mut self, path: &Path) -> std::io::Result<PathBuf>;
    fn stage(&mut self, directory: &Path, name: &str, target: &Path) -> std::io::Result<PathBuf>;
    fn rename(&mut self, part: &Path, path: &Path) -> std::io::Result<()>;
    fn remove(&mut self, part: &Path) -> std::io::Result<()>;
}

struct DiskLinks;

impl Links for DiskLinks {
    fn read(&mut self, path: &Path) -> std::io::Result<PathBuf> {
        fs::read_link(path)
    }

    fn stage(&mut self, directory: &Path, name: &str, target: &Path) -> std::io::Result<PathBuf> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let part = directory.join(format!(".{name}.{}.{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        symlink(target, &part)?;
        Ok(part)
    }

    fn rename(&mut self, part: &Path, path: &Path) -> std::io::Result<()> {
        fs::rename(part, path)
    }

    fn remove(&mut self, part: &Path) -> std::io::Result<()> {
        fs::remove_file(part)
    }
}

fn link(directory: &Path, name: &str, target: &Path, links: &mut impl Links) -> Result<(), String> {
    let path = directory.join(name);
    if links.read(&path).ok().as_deref() == Some(target) {
        return Ok(());
    }
    let part = links.stage(directory, name, target).map_err(|why| format!("{}: {why}", path.display()))?;
    if let Err(why) = links.rename(&part, &path) {
        match links.remove(&part) {
            Ok(()) | Err(_) => {}
        }
        return Err(format!("{}: {why}", path.display()));
    }
    Ok(())
}

/// The launcher fetch's small io limits (temper/docs/design/protocol/setup/owned.md, 6).
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

fn trust(environment: &[EnvironmentEntry]) -> skein_shell::Trust {
    skein_shell::Trust::Machine {
        ssl_cert_file: temper_shell::environment::SSL_CERT_FILE.value(environment).map(|value| path(value).into()),
    }
}

fn path(bytes: &[u8]) -> &Path {
    Path::new(OsStr::from_bytes(bytes))
}

/// Claude Code's launcher, beside its programs directory (temper/docs/design/protocol/setup/owned.md, 6).
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
    use skein_fake_machine::{How, Item, Machine, Opened};

    #[test]
    fn launcher_disables_both_update_paths() {
        let command = command(Path::new("/cached/claude"), &[]);
        for name in ["DISABLE_AUTOUPDATER", "DISABLE_UPDATES"] {
            assert_eq!(
                command.get_envs().find(|(key, _)| *key == OsStr::new(name)),
                Some((OsStr::new(name), Some(OsStr::new("1")))),
                "{name} keeps the pinned build from updating itself"
            );
        }
    }

    #[test]
    fn a_cache_refusal_names_the_agent_and_claude() {
        let failure = temper_fetch_shell::fetch::Failed::Refused(temper_fetch::fetch::Failure::Present {
            file: b"/cache/claude".as_slice().into(),
        });
        assert_eq!(
            launch_refusal("temper-agent", &failure.words()),
            (1, "temper-agent: claude: /cache/claude: does not match the pinned digest\n".into())
        );
    }

    #[test]
    fn download_trust_uses_the_given_certificate_file() {
        let environment = [(Box::from(b"SSL_CERT_FILE".as_slice()), Box::from(b"/fixture/roots.pem".as_slice()))];
        let skein_shell::Trust::Machine { ssl_cert_file } = trust(&environment) else { panic!("machine trust") };
        assert_eq!(ssl_cert_file, Some(PathBuf::from("/fixture/roots.pem")));
        let empty = [(Box::from(b"SSL_CERT_FILE".as_slice()), Box::from(b"".as_slice()))];
        for environment in [&[][..], &empty[..]] {
            let skein_shell::Trust::Machine { ssl_cert_file } = trust(environment) else { panic!("machine trust") };
            assert_eq!(ssl_cert_file, None);
        }
    }

    #[test]
    fn place_links_claude_to_the_given_program() {
        let directory = Path::new("/fixture");
        let launcher = directory.join("claude-code");
        let cache = directory.join("cache");
        let mut links = ScriptedLinks::new(&[]);
        let program = Path::new("/opt/agent/temper-agent");
        place_links(&launcher, &cache, program, &mut links).unwrap();
        assert_eq!(links.read(&launcher.join("claude")).unwrap(), program);
        assert_eq!(links.read(&launcher.join("cache")).unwrap(), cache);
        let replacement = Path::new("/opt/replacement/temper-agent");
        place_links(&launcher, &cache, replacement, &mut links).unwrap();
        assert_eq!(links.read(&launcher.join("claude")).unwrap(), replacement);
        assert_eq!(links.read(&launcher.join("cache")).unwrap(), cache);
    }

    #[test]
    fn a_wrong_size_cache_refuses_after_replacing_both_stale_login_links() {
        use temper_fetch::fetch::Failure;
        use temper_fetch_shell::fetch::Failed;

        let directory = Path::new("/fixture/claude-code");
        let launcher = directory.join("claude");
        let cache_link = directory.join("cache");
        let cache = Path::new("/fixture/cache");
        let binary = Path::new("/fixture/bin/temper-box");
        let file = Path::new("/fixture/cache/claude-code/2.1.311/claude");
        let mut links = ScriptedLinks::new(&[
            Item::file(b"claude-code/claude", b"/stale/temper"),
            Item::file(b"claude-code/cache", b"/stale/cache"),
        ]);

        for first in [true, false] {
            place_links(directory, cache, binary, &mut links).expect("place launcher links for login");
            assert_eq!(links.read(&launcher).expect("claude link"), binary);
            assert_eq!(links.read(&cache_link).expect("cache link"), cache);
            if first {
                assert_eq!(links.changes, ["stage", "rename", "stage", "rename"]);
                links.changes.clear();
            } else {
                assert!(links.changes.is_empty(), "a second login keeps both links untouched");
            }

            let why = launch_command_with(
                launcher.as_os_str(),
                &[b"auth".as_slice().into(), b"login".as_slice().into()],
                |path| {
                    assert_eq!(path, cache_link);
                    links.read(path)
                },
                |candidate, pin, _| {
                    assert_eq!(candidate, file);
                    assert_eq!(pin.file.as_ref(), file.as_os_str().as_bytes());
                    // The fetch shell refuses this cache fact before executing or downloading anything.
                    Err(Failed::Refused(Failure::Present { file: pin.file }))
                },
            )
            .expect_err("no command to exec when the cache refuses");
            let mut stderr = Vec::new();
            assert_eq!(refusal("temper", &why, &mut stderr), 1);
            assert_eq!(
                stderr,
                b"temper: claude: /fixture/cache/claude-code/2.1.311/claude: does not match the pinned digest\n"
            );
        }
    }

    #[test]
    fn serve_installs_only_the_launcher_links_not_a_version_directory() {
        let directory = Path::new("/fixture/claude-code");
        let executable = Path::new("/fixture/current-exe");
        let cache = Path::new("/fixture/cache");
        let mut links = ScriptedLinks::new(&[]);
        place_links(directory, cache, executable, &mut links).expect("place launcher links for serve");
        assert_eq!(links.read(&directory.join("claude")).expect("claude link"), executable);
        assert_eq!(links.read(&directory.join("cache")).expect("cache link"), cache);
        assert_eq!(links.changes, ["stage", "rename", "stage", "rename"]);
        assert_eq!(
            links.machine.open(links.place, b"2.1.311", How::Directory),
            Err(skein_fake_machine::Refusal::NotFound)
        );
    }

    // The namespace stores read-link replies as bytes, including absolute targets.
    struct ScriptedLinks {
        machine: Machine,
        place: Opened,
        changes: Vec<&'static str>,
    }

    impl ScriptedLinks {
        fn new(items: &[Item]) -> Self {
            let mut machine = Machine::new();
            let mut tree = vec![Item::directory(b"claude-code")];
            tree.extend_from_slice(items);
            let root = machine.lay(&tree);
            let place = machine.open(root, b"claude-code", How::Directory).expect("fixture directory");
            Self { machine, place, changes: Vec::new() }
        }
    }

    fn link_name(file: &Path) -> &[u8] {
        assert_eq!(file.parent(), Some(Path::new("/fixture/claude-code")));
        file.file_name().unwrap().as_bytes()
    }

    impl Links for ScriptedLinks {
        fn read(&mut self, file: &Path) -> std::io::Result<PathBuf> {
            let opened = self
                .machine
                .open(self.place, link_name(file), How::Read)
                .map_err(|why| std::io::Error::other(format!("{why:?}")))?;
            let length = u32::try_from(self.machine.stat(opened).size).unwrap();
            let bytes = self.machine.read(opened, 0, length).unwrap();
            self.machine.close(opened);
            Ok(path(&bytes).into())
        }

        fn stage(&mut self, directory: &Path, name: &str, target: &Path) -> std::io::Result<PathBuf> {
            self.changes.push("stage");
            let part = directory.join(format!(".{name}.part"));
            let opened = self.machine.open(self.place, link_name(&part), How::Create { mode: 0o600 }).unwrap();
            self.machine.write(opened, 0, target.as_os_str().as_bytes()).unwrap();
            self.machine.close(opened);
            Ok(part)
        }

        fn rename(&mut self, part: &Path, file: &Path) -> std::io::Result<()> {
            self.changes.push("rename");
            self.machine.rename(self.place, link_name(part), self.place, link_name(file), false).unwrap();
            Ok(())
        }

        fn remove(&mut self, part: &Path) -> std::io::Result<()> {
            self.changes.push("remove");
            self.machine.remove(self.place, link_name(part), false).unwrap();
            Ok(())
        }
    }

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

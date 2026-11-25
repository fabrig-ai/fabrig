//! Retained isolated startup for `temper-box --fake-backends` (protocol/configuration/directories.md, 1).
//! The fake box's steps follow temper/docs/design/protocol/setup/box-backends.md, 1.
use crate::directories::jig_directories;
mod host;
mod import;
use crate::directories::{Directories, make_private};
use host::{Files, Host, Initialized, Native};
use import::Source;
use skein_io::kernel::{Exit, Fd, Kind};
use std::{
    ffi::OsStr,
    fs::{self, OpenOptions},
    io,
    net::{SocketAddr, TcpListener},
    num::NonZeroU16,
    os::fd::IntoRawFd,
    os::unix::{
        ffi::OsStrExt,
        fs::{MetadataExt, OpenOptionsExt},
    },
    path::{Component, Path, PathBuf},
};
use temper_fake::backends::{self, Paths, Peers};
use temper_forge_setup_shell::process as locks;

use temper_config_shell::{FILE, Shape};

pub const ADAPTER: &str = "fake-claude.path";
const REQUIRED: &[&str] = &[FILE, "github.json", "oauth.json", "llm.json", "forgejo.json", ADAPTER];

/// A new sandbox's nine ports, the process tier's (temper/docs/design/protocol/setup/box-backends.md, 1).
pub(crate) const FAKE_BOX_PORTS: skein_shell::Variable = skein_shell::Variable {
    name: "TEMPER_INTERNAL_FAKE_BOX_PORTS",
    set: skein_shell::Set::Internal,
    form: "PORTS",
    line: "Choose the fake box's nine listener and peer ports.",
    absent: "Reserve available ports.",
    notes: "Nine comma-separated nonzero ports; used only for a new sandbox.",
    way: skein_shell::Way::Reads,
};
pub const CHOSEN_PORTS: &str = FAKE_BOX_PORTS.name;
fn ports_from(environment: &[crate::EnvironmentEntry]) -> Result<Option<Chosen>, String> {
    FAKE_BOX_PORTS.value(environment).map(OsStr::from_bytes).map(chosen_ports).transpose()
}
pub struct Chosen {
    listeners: [u16; 5],
    peers: [u16; 4],
}
pub fn chosen_ports(value: &OsStr) -> Result<Chosen, String> {
    let refuse = || format!("{CHOSEN_PORTS}: expected nine comma-separated nonzero ports");
    let text = value.to_str().ok_or_else(refuse)?;
    let ports = text
        .split(',')
        .map(|word| word.parse::<u16>().ok().filter(|port| *port != 0).ok_or_else(refuse))
        .collect::<Result<Vec<_>, _>>()?;
    let [clients, owned, forge, llm, webhooks, github, oauth, model, forgejo]: [u16; 9] =
        ports.try_into().map_err(|_| refuse())?;
    Ok(Chosen { listeners: [clients, owned, forge, llm, webhooks], peers: [github, oauth, model, forgejo] })
}

pub struct Listeners(pub Vec<(Fd, NonZeroU16)>);
impl Drop for Listeners {
    fn drop(&mut self) {
        crate::close(std::mem::take(&mut self.0).into_iter().map(|(fd, _)| fd).collect());
    }
}
pub struct Prepared {
    pub directories: Directories,
    pub bytes: Box<[u8]>,
    pub listeners: Listeners,
    pub peers: Peers,
    pub steps: Box<[temper_forge_setup::Step]>,
    // Held through drive; closing the descriptor releases the OS lock even after a crash.
    _lock: locks::Lock,
}
#[derive(Default)]
pub struct Options {
    pub bundle: Option<PathBuf>,
    pub scripts: Vec<PathBuf>,
    pub strict: bool,
}
pub(crate) fn configure(rest: &[std::ffi::OsString], working: &[u8]) -> Result<Options, String> {
    let mut fake = false;
    let mut options = Options::default();
    let mut words = rest.iter();
    while let Some(word) = words.next() {
        match word.as_bytes() {
            b"--fake-backends" if !fake => fake = true,
            b"--strict" if !options.strict => options.strict = true,
            flag @ (b"--bundle" | b"--scripts") if flag == b"--scripts" || options.bundle.is_none() => {
                let flag = String::from_utf8_lossy(flag);
                let value = words.next().ok_or_else(|| format!("{flag}: missing path"))?;
                let value = value.as_bytes();
                if value.is_empty() || value.contains(&0) || value.starts_with(b"--") {
                    return Err(format!("{flag}: expected a nonempty path without NUL"));
                }
                let path = crate::path(working).join(crate::path(value));
                let name = path.as_os_str().as_bytes();
                if !path.is_absolute() || name.contains(&0) || name.len() > 4096 {
                    return Err(format!("{flag}: expected an absolute path of at most 4096 bytes without NUL"));
                }
                if flag == "--scripts" {
                    options.scripts.push(path);
                } else {
                    options.bundle = Some(path);
                }
            }
            _ => {
                return Err(
                    "invalid box arguments: expected --fake-backends [--bundle BUNDLE] [--scripts FILE]... [--strict]"
                        .into(),
                );
            }
        }
    }
    if !fake {
        return Err("--bundle, --scripts and --strict require --fake-backends".into());
    }
    Ok(options)
}
pub(crate) fn run(
    source: &Directories,
    options: &Options,
    echo: temper_box_machine::host::Echo,
    front: bool,
    record: bool,
    environment: &[crate::EnvironmentEntry],
) -> Result<Exit, String> {
    let binary = std::env::current_exe().map_err(|why| why.to_string())?;
    let adapter = sibling(&binary)?;
    let chosen = ports_from(environment)?;
    let Prepared { directories, bytes, mut listeners, peers, steps, _lock: lock } =
        prepare(source, options, &adapter, chosen.as_ref(), environment)?;
    let opening = crate::box_command::Opening {
        log_lines: Box::new([]),
        after_block: opening_lines(&directories, &peers, &binary),
    };
    let result = crate::box_command::run_configuration(
        &directories,
        &bytes,
        std::mem::take(&mut listeners.0),
        steps,
        echo,
        front,
        record,
        opening,
        environment,
    );
    drop(lock);
    result
}
/// Per-start scripts and arguments; retained peer documents are never rewritten (protocol/setup/box-backends.md, 1).
fn scripts_for_start(
    disk: &impl Files,
    dirs: &Directories,
    peers: &Peers,
    options: &Options,
) -> Result<Box<[temper_forge_setup::Step]>, String> {
    scripts_for_start_with(
        dirs,
        peers,
        options,
        |path| disk.remove(path),
        || peer_document_with(disk, dirs, "llm.json", temper_fake::DOCUMENT_BYTES),
        |path, bound| disk.read(path, bound, false),
        |path, joined| write_with(disk, path, joined, disk.exists(path)),
    )
}

fn scripts_for_start_with(
    dirs: &Directories,
    peers: &Peers,
    options: &Options,
    remove: impl FnOnce(&Path) -> std::io::Result<()>,
    model: impl FnOnce() -> Result<Box<[u8]>, String>,
    read: impl FnMut(&Path, u32) -> Result<Box<[u8]>, skein_shell::Unread>,
    publish: impl FnOnce(&Path, &[u8]) -> Result<(), String>,
) -> Result<Box<[temper_forge_setup::Step]>, String> {
    let path = crate::path(&dirs.config).join("llm-scripts.json");
    if options.scripts.is_empty() {
        match remove(&path) {
            Ok(()) => {}
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => {}
            Err(why) => return Err(format!("{}: {why}", path.display())),
        }
        return Ok(backends::steps(&dirs.config, &dirs.data, peers, None, options.strict));
    }
    let document = model()?;
    let joined = join_scripts(&options.scripts, &document, read)?;
    publish(&path, &joined)?;
    Ok(backends::steps(&dirs.config, &dirs.data, peers, Some(&bytes(&path)), options.strict))
}
fn join_scripts(
    files: &[PathBuf],
    document: &[u8],
    mut read: impl FnMut(&Path, u32) -> Result<Box<[u8]>, skein_shell::Unread>,
) -> Result<Box<[u8]>, String> {
    let mut joined = temper_fake::llm::ScriptJoin::empty();
    for file in files {
        let refuse = |why| format!("{}: {why}", file.display());
        let source = read(file, temper_fake::DOCUMENT_BYTES).map_err(|why| refuse(crate::unread(why)))?;
        joined.push(document, &source).map_err(|why| {
            let line = why.line();
            let line = String::from_utf8_lossy(&line);
            // The public JSON writer's byte refusal has no final newline.
            refuse(if why.words == b"past byte bound" { line.trim_end_matches('\n').into() } else { line.into_owned() })
        })?;
    }
    Ok(joined.into_bytes())
}

#[must_use]
pub fn bytes(path: &Path) -> Box<[u8]> {
    path.as_os_str().as_bytes().into()
}
fn public_path(path: &Path) -> Result<(), String> {
    // Public imports must not turn a link into an accidental source-secret copy.
    let path = skein_shell::resolve_system_links(path);
    public_path_with(&path, |path| {
        fs::symlink_metadata(path).map(|metadata| PublicMetadata {
            linked: metadata.file_type().is_symlink(),
            file: metadata.is_file(),
            links: metadata.nlink(),
        })
    })
}

struct PublicMetadata {
    linked: bool,
    file: bool,
    links: u64,
}

fn public_path_with(
    path: &Path,
    mut metadata: impl FnMut(&Path) -> std::io::Result<PublicMetadata>,
) -> Result<(), String> {
    let mut prefix = PathBuf::new();
    for component in path.components() {
        prefix.push(component.as_os_str());
        let facts = metadata(&prefix).map_err(|why| format!("{}: {why}", prefix.display()))?;
        if facts.linked {
            return Err(format!("{}: public documents must not be reached through links", prefix.display()));
        }
    }
    let facts = metadata(path).map_err(|why| format!("{}: {why}", path.display()))?;
    if !facts.file || facts.links != 1 {
        return Err(format!("{}: expected a regular public document with one link", path.display()));
    }
    Ok(())
}
/// Check an in-memory candidate before it replaces the retained file.
pub(super) fn configuration(dirs: &Directories, text: &[u8]) -> Result<temper_config_shell::Read, String> {
    configuration_with(&Native(&[]), dirs, text)
}
fn configuration_with(disk: &impl Host, dirs: &Directories, text: &[u8]) -> Result<temper_config_shell::Read, String> {
    let facts = disk.config_facts(dirs).map_err(|why| String::from_utf8_lossy(&why.lines()).into_owned())?;
    temper_config_shell::parse(text, Shape::Standalone, &facts)
        .map_err(|why| String::from_utf8_lossy(&why.lines()).into_owned())
}
fn public(path: &Path) -> Result<Box<[u8]>, String> {
    public_with(&Native(&[]), path)
}
fn public_with(disk: &impl Files, path: &Path) -> Result<Box<[u8]>, String> {
    disk.public_path(path)?;
    disk.read(path, crate::CONFIG_BYTES, false).map_err(|why| format!("{}: {}", path.display(), crate::unread(why)))
}
fn private_document(disk: &impl Files, path: &Path) -> Result<(), String> {
    let metadata = disk.metadata(path).map_err(|why| format!("{}: {why}", path.display()))?.stat;
    private_document_from(path, metadata.owner, disk.user(), metadata.mode)
}

fn private_document_from(path: &Path, owner: u32, user: u32, mode: u32) -> Result<(), String> {
    if owner != user || mode & 0o077 != 0 {
        return Err(format!("{}: retained documents must be private files of this user's", path.display()));
    }
    Ok(())
}
fn resolved(root: &Path, name: &[u8]) -> Result<PathBuf, String> {
    if name.contains(&0) || name.len() > 4096 {
        return Err("public path: NUL or past 4096 bytes".into());
    }
    let path = root.join(crate::path(name));
    let mut clean = PathBuf::new();
    for part in path.components() {
        match part {
            Component::ParentDir => return Err(format!("{}: parent components are unsupported", path.display())),
            Component::CurDir => {}
            Component::RootDir | Component::Normal(_) => clean.push(part.as_os_str()),
            Component::Prefix(_) => return Err("public path: unsupported prefix".into()),
        }
    }
    if clean.as_os_str().len() > 4096 {
        return Err("effective path: past 4096 bytes".into());
    }
    Ok(clean)
}
pub fn write(path: &Path, content: &[u8], replace: bool) -> Result<(), String> {
    write_with(&Native(&[]), path, content, replace)
}
fn write_with(disk: &impl Files, path: &Path, content: &[u8], replace: bool) -> Result<(), String> {
    if replace {
        let metadata = disk.metadata(path).map_err(|why| format!("{}: {why}", path.display()))?.stat;
        if metadata.kind != Kind::File || metadata.owner != disk.user() || metadata.links != 1 {
            return Err(format!("{}: expected this user's regular file", path.display()));
        }
    }
    let mut file = disk.open_write(path, 0o600, replace).map_err(|why| format!("{}: {why}", path.display()))?;
    disk.mode(&file, 0o600).map_err(|why| why.to_string())?;
    disk.write(&mut file, content).map_err(|why| format!("{}: {why}", path.display()))?;
    disk.sync(&file).map_err(|why| format!("{}: {why}", path.display()))
}
fn replace_bundle_with(
    disk: &impl Host,
    dirs: &Directories,
    content: Box<[u8]>,
    bundle: Option<&Path>,
    adapter: &Path,
    deployment: &crate::setup::Deployment,
) -> Result<Box<[u8]>, String> {
    let mut updated = content.clone();
    if let Some(bundle) = bundle {
        let mut file = configuration_with(disk, dirs, &content)?.file;
        if file.internal.bundle.as_deref() != Some(bundle) {
            file.internal.bundle = Some(bundle.to_path_buf());
            updated = temper_config_shell::render(&file, Shape::Standalone);
        }
    }
    // Check the entire candidate, including its bundle, before touching retained bytes.
    if updated == content {
        validate_checked(disk, dirs, deployment, adapter)?;
        return Ok(content);
    }
    validate_with(disk, dirs, &updated, adapter)?;
    let path = crate::path(&dirs.config).join(FILE);
    let (staging, mut file) = open_staging(&path, |path| disk.open_write(path, 0o600, false))?;
    let result: io::Result<()> = (|| {
        disk.write(&mut file, &updated)?;
        disk.sync(&file)?;
        disk.rename(&staging, &path)
    })();
    if let Err(why) = result {
        let _removed = disk.remove(&staging);
        return Err(format!("{}: bundle replacement: {why}", path.display()));
    }
    Ok(updated)
}

fn open_staging<T>(document: &Path, open: impl FnOnce(&Path) -> std::io::Result<T>) -> Result<(PathBuf, T), String> {
    let staging = document.with_file_name(format!("{FILE}.bundle-staging"));
    let file = open(&staging).map_err(|why| {
        format!(
            "{}: {why}; an existing staging file must be removed explicitly before another bundle replacement",
            staging.display()
        )
    })?;
    Ok((staging, file))
}
fn lock(root: &Path) -> Result<locks::Lock, String> {
    let path = root.join("lock");
    let user = skein_shell::effective_user();
    match fs::symlink_metadata(&path) {
        Ok(metadata)
            if !metadata.is_file()
                || metadata.file_type().is_symlink()
                || metadata.uid() != user
                || metadata.mode() & 0o077 != 0 =>
        {
            return Err(reset(root, "lock must be this user's private regular file"));
        }
        Ok(_) => {}
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => {}
        Err(why) => return Err(format!("{}: {why}", path.display())),
    }
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(&path)
        .map_err(|why| format!("{}: {why}", path.display()))?;
    let named = fs::symlink_metadata(&path).map_err(|why| why.to_string())?;
    let opened = file.metadata().map_err(|why| why.to_string())?;
    if named.file_type().is_symlink() || named.ino() != opened.ino() || named.dev() != opened.dev() {
        return Err(format!("{}: lock changed while opening", path.display()));
    }
    locks::lock_file(file)
        .map_err(|why| format!("{}: another fake-backend box is using this sandbox: {why:?}", path.display()))
}
fn reset(root: &Path, reason: &str) -> String {
    format!("{}: {reason}; remove this sandbox directory explicitly to reset it", root.display())
}
fn directories(source: &Directories) -> Result<(PathBuf, Directories), String> {
    let root = resolved(crate::path(&source.data), b"box/fake-backends")?;
    let config = bytes(&root.join("config"));
    let data = bytes(&root.join("data"));
    // Joined agent/program paths must fit the same bound as ordinary init.
    for path in [&config, &data] {
        if path.len().checked_add(32).is_none_or(|n| n > 4096) {
            return Err("fake-backends: effective directory past its path bound".into());
        }
    }
    Ok((root, Directories { config, data, cache: source.cache.clone(), origins: [skein_shell::Origin::Flag; 3] }))
}
pub(super) const PORT_ATTEMPTS: usize = 32;

pub fn reserve_from(excluded: &[u16], bind: impl FnMut() -> io::Result<TcpListener>) -> Result<TcpListener, String> {
    reserve_from_with(excluded, bind, TcpListener::local_addr)
}
fn reserve_from_with<L>(
    excluded: &[u16],
    mut bind: impl FnMut() -> io::Result<L>,
    address: impl Fn(&L) -> io::Result<SocketAddr>,
) -> Result<L, String> {
    for _ in 0..PORT_ATTEMPTS {
        let listener = match bind() {
            Ok(listener) => listener,
            Err(why) if why.kind() == io::ErrorKind::AddrInUse => continue,
            Err(why) => return Err(format!("fake peer reservation: {why}")),
        };
        if !excluded.contains(&address(&listener).map_err(|why| why.to_string())?.port()) {
            return Ok(listener);
        }
    }
    Err("fake peer reservation: could not choose a port distinct from the deployment and callback".into())
}
fn reserve_chosen_with<H: Host>(disk: &H, excluded: &[u16], port: Option<u16>) -> Result<H::Peer, String> {
    match port {
        Some(port) => disk.bind(port).map_err(|why| format!("{CHOSEN_PORTS}: port {port}: {why}")),
        None => reserve_from_with(excluded, || disk.bind(0), |listener| disk.address(listener)),
    }
}
fn contained(root: &[u8], path: &[u8]) -> bool {
    crate::path(path)
        .strip_prefix(crate::path(root))
        .is_ok_and(|rest| !rest.as_os_str().is_empty() && rest.components().all(|p| matches!(p, Component::Normal(_))))
}
fn peer_document(dirs: &Directories, name: &str, bound: u32) -> Result<Box<[u8]>, String> {
    peer_document_with(&Native(&[]), dirs, name, bound)
}
fn peer_document_with(disk: &impl Files, dirs: &Directories, name: &str, bound: u32) -> Result<Box<[u8]>, String> {
    let path = crate::path(&dirs.config).join(name);
    disk.public_path(&path)?;
    private_document(disk, &path)?;
    disk.read(&path, bound, true).map_err(|why| format!("{}: {}", path.display(), crate::unread(why)))
}
/// The ports no peer may take: the callback's and the ordinary box's listeners and readiness
/// (temper/docs/design/protocol/setup/box-backends.md, 1).
fn ordinary_ports(dirs: &Directories, bytes: &[u8]) -> Result<Vec<u16>, String> {
    let deployment = crate::setup::check(bytes, &jig_directories(dirs))?;
    ordinary_ports_checked(dirs, &deployment)
}
fn ordinary_ports_checked(dirs: &Directories, deployment: &crate::setup::Deployment) -> Result<Vec<u16>, String> {
    let (ordinary, _) = crate::box_command::configuration_steps_checked(dirs, deployment, Box::new([]), false)?;
    let mut ports = vec![1455];
    for step in
        ordinary.always.iter().chain(ordinary.asked.iter().flat_map(|group| group.steps.iter().map(|(_, step)| step)))
    {
        for listen in step.listeners() {
            ports.push(listen.port());
        }
        if let Some(ready) = step.ready() {
            ports.push(ready.port());
        }
    }
    Ok(ports)
}
fn checked_peers(
    disk: &impl Files,
    dirs: &Directories,
    deployment: &crate::setup::Deployment,
) -> Result<Peers, String> {
    checked_peers_with(dirs, deployment, |name, bound| peer_document_with(disk, dirs, name, bound))
}

fn checked_peers_with(
    dirs: &Directories,
    deployment: &crate::setup::Deployment,
    mut read: impl FnMut(&str, u32) -> Result<Box<[u8]>, String>,
) -> Result<Peers, String> {
    let mut ports = ordinary_ports_checked(dirs, deployment)?;
    let refuse = |name: &str, why: temper_fake::Refusal| {
        format!("{}: {}", crate::path(&dirs.config).join(name).display(), String::from_utf8_lossy(&why.line()))
    };
    let github = temper_fake::forge::Document::decode(&read("github.json", temper_fake::DOCUMENT_BYTES)?)
        .map_err(|why| refuse("github.json", why))?;
    let oauth = temper_fake::oauth::Document::decode(&read("oauth.json", temper_fake::oauth::DOCUMENT_BYTES)?)
        .map_err(|why| refuse("oauth.json", why))?;
    let llm = temper_fake::llm::Document::decode(&read("llm.json", temper_fake::DOCUMENT_BYTES)?)
        .map_err(|why| refuse("llm.json", why))?;
    let forgejo = source_forgejo(&read("forgejo.json", temper_fake::DOCUMENT_BYTES)?)
        .map_err(|why| refuse("forgejo.json", why))?;
    let peers = Peers { github: github.listen(), oauth: oauth.listen(), llm: llm.listen(), forgejo: forgejo.listen() };
    for (name, listen) in [
        ("github.json", peers.github),
        ("oauth.json", peers.oauth),
        ("llm.json", peers.llm),
        ("forgejo.json", peers.forgejo),
    ] {
        if !listen.ip().is_loopback() || listen.port() == 0 || ports.contains(&listen.port()) {
            return Err(format!(
                "{}: peer listen must be nonzero loopback with a distinct, unreserved port",
                crate::path(&dirs.config).join(name).display()
            ));
        }
        ports.push(listen.port());
    }
    for (name, output) in
        [("github.json", github.admin_token()), ("forgejo.json", forgejo.admin_token()), ("llm.json", llm.key())]
            .into_iter()
            .chain(llm.added_keys().iter().map(|path| ("llm.json", path.as_ref())))
            .chain(llm.root().into_iter().map(|path| ("llm.json", path)))
    {
        if !contained(&dirs.data, output) {
            return Err(format!(
                "{}: output {} must stay under effective data",
                crate::path(&dirs.config).join(name).display(),
                crate::path(output).display()
            ));
        }
    }
    Ok(peers)
}
fn source_forgejo(bytes: &[u8]) -> Result<temper_fake::forge::Document, temper_fake::Refusal> {
    let document = temper_fake::forge::Document::decode(bytes)?;
    if document.sources().is_empty() {
        return Err(temper_fake::Refusal {
            path: jig_protocol_config::Path::root().member(b"sources"),
            words: b"missing copy sources; remove this sandbox directory explicitly to reset it",
        });
    }
    Ok(document)
}
pub fn validate(dirs: &Directories, bytes: &[u8], expected: &Path) -> Result<crate::setup::Deployment, String> {
    validate_with(&Native(&[]), dirs, bytes, expected)
}
fn validate_with(
    disk: &impl Host,
    dirs: &Directories,
    bytes: &[u8],
    expected: &Path,
) -> Result<crate::setup::Deployment, String> {
    let document = configuration_with(disk, dirs, bytes)?.document;
    let deployment = crate::setup::check(&document, &jig_directories(dirs))?;
    validate_checked(disk, dirs, &deployment, expected)?;
    Ok(deployment)
}
fn validate_checked(
    disk: &impl Host,
    dirs: &Directories,
    deployment: &crate::setup::Deployment,
    expected: &Path,
) -> Result<(), String> {
    let named = &deployment.named;
    for file in named.files.iter().chain(&named.asked_files) {
        if !(contained(&dirs.config, &file.path) && !file.private || contained(&dirs.data, &file.path)) {
            return Err(format!(
                "{}: effective resource must stay under its sandbox config/data",
                crate::file_path(file.at.clone())
            ));
        }
    }
    if !contained(&dirs.data, &named.deployment)
        || named.worker.as_ref().is_none_or(|worker| !contained(&dirs.data, &worker.root))
    {
        return Err("effective deployment, worker root and agent configuration must stay in the sandbox".into());
    }
    check_adapter(
        expected,
        || peer_document_with(disk, dirs, ADAPTER, 4096),
        || provisioned_program_from(expected, disk.program_metadata(expected)),
    )?;
    for file in &named.files {
        if !file.private {
            let asset = public_with(disk, crate::path(&file.path))?;
            if asset.len() > usize::try_from(file.max).expect("file bound fits") {
                return Err(format!("{}: public asset past its byte bound", crate::file_path(file.at.clone())));
            }
        }
    }
    match &named.bundle {
        Some(bundle) => {
            let _checked = provisioned_bundle(disk, Some(crate::path(bundle)), named.bundle_limits)?;
        }
        None => return Err("application.clients.bundle: effective document must name its bundle".into()),
    }
    Ok(())
}
fn provisioned_bundle(
    disk: &impl Host,
    path: Option<&Path>,
    limits: temper_service::limits::Clients,
) -> Result<PathBuf, String> {
    provisioned_bundle_with(path, limits, |page| disk.public_path(page), |path, limits| disk.read_bundle(path, limits))
}

fn provisioned_bundle_with(
    path: Option<&Path>,
    limits: temper_service::limits::Clients,
    check_page: impl FnOnce(&Path) -> Result<(), String>,
    read: impl FnOnce(&Path, temper_service::limits::Clients) -> Result<(), String>,
) -> Result<PathBuf, String> {
    let instructions =
        "run scripts/bundle.sh DIR && temper-box --fake-backends --bundle BUNDLE (directory flags as usual)";
    let path = path.ok_or_else(|| format!("application.clients.bundle: no provisioned bundle; {instructions}"))?;
    let page = path.join("index.html");
    check_page(&page)
        .map_err(|why| format!("application.clients.bundle: {}: {why}; {instructions}", page.display()))?;
    read(path, limits).map_err(|why| format!("{why}; {instructions}"))?;
    Ok(path.to_path_buf())
}
fn opening_lines(dirs: &Directories, peers: &Peers, binary: &Path) -> Box<[u8]> {
    let mut lines = vec![
        format!("box: effective configuration {}", crate::path(&dirs.config).display()),
        format!("box: effective data {}", crate::path(&dirs.data).display()),
        format!("box: effective cache {}", crate::path(&dirs.cache).display()),
    ];
    let forgejo = temper_fake::forge::Document::decode(
        &peer_document(dirs, "forgejo.json", temper_fake::DOCUMENT_BYTES).expect("validated forgejo document"),
    )
    .expect("validated forgejo document");
    lines.extend(peer_lines(peers, &forgejo));
    // Retained peer documents may select another key file under effective data.
    let model = temper_fake::llm::Document::decode(
        &peer_document(dirs, "llm.json", temper_fake::DOCUMENT_BYTES).expect("validated model document"),
    )
    .expect("validated model document");
    lines.push(format!("box: model key {}", crate::path(model.key()).display()));
    lines.push(format!(
        "box: Forgejo elsewhere admin token file {}",
        crate::path(&dirs.data).join("forgejo-admin.token").display()
    ));
    lines.push(format!("box: program login: {}", program_login_line(dirs, binary)));
    lines.push(format!(
        "box: GitHub own App id 1, key file {}",
        crate::path(&dirs.data).join("github-app.pem").display()
    ));
    lines.push(String::new());
    lines.join("\n").into_bytes().into_boxed_slice()
}
fn peer_lines(peers: &Peers, forgejo: &temper_fake::forge::Document) -> Vec<String> {
    let mut lines = Vec::new();
    for (name, address) in
        [("github", peers.github), ("issuer", peers.oauth), ("model", peers.llm), ("forgejo", peers.forgejo)]
    {
        lines.push(format!("box: fake {name} http://{address}"));
    }
    for source in forgejo.sources() {
        lines.push(format!("box: copy source {}", String::from_utf8_lossy(&source.address)));
    }
    lines
}
fn program_login_line(dirs: &Directories, binary: &Path) -> String {
    let command =
        crate::program_login::line(dirs, &crate::printed::path_word(binary.as_os_str().as_bytes(), None), None)
            .unwrap_or_else(|| "temper program-login --config CONFIG --data DATA --cache CACHE".into());
    format!("{command} NAME (the name the Model step shows)")
}

fn sibling(binary: &Path) -> Result<PathBuf, String> {
    if !binary.is_absolute() {
        return Err("temper binary: expected an absolute path".into());
    }
    Ok(binary.parent().ok_or("temper binary: missing parent")?.join("fake-claude"))
}
fn adapter_identity(given: &[u8], expected: &Path) -> Result<(), String> {
    if given != expected.as_os_str().as_bytes() {
        return Err("fake-claude.path: expected this temper's fake-claude sibling; remove this sandbox directory explicitly to reset it".into());
    }
    Ok(())
}
/// The sandbox pins its fake adapter outside the deployment's fixed worker members.
pub fn configured_claude(dirs: &Directories) -> Result<Option<PathBuf>, String> {
    configured_claude_with(dirs, &Native(&[]))
}
fn configured_claude_with(dirs: &Directories, disk: &impl Host) -> Result<Option<PathBuf>, String> {
    let record = crate::path(&dirs.config).join(ADAPTER);
    match disk.metadata(&record) {
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => {
            let marker = record.parent().and_then(Path::parent).map(|root| root.join("prepared"));
            if marker.is_some_and(|marker| disk.exists(&marker)) {
                return Err(format!(
                    "{ADAPTER}: missing pinned adapter; remove this sandbox directory explicitly to reset it"
                ));
            }
            return Ok(None);
        }
        Err(why) => return Err(format!("{}: {why}", record.display())),
        Ok(_) => {}
    }
    let expected = sibling(&disk.binary()?)?;
    check_adapter(
        &expected,
        || peer_document_with(disk, dirs, ADAPTER, 4096),
        || provisioned_program_from(&expected, disk.program_metadata(&expected)),
    )?;
    Ok(Some(expected))
}
fn provisioned_program_from(path: &Path, metadata: Option<(bool, u32)>) -> Result<(), String> {
    let executable = metadata.is_some_and(|(file, mode)| file && mode & 0o111 != 0);
    if !executable {
        return Err(format!(
            "{}: missing or nonexecutable adapter; provision it with cargo build -p smith-fake-claude --bin fake-claude",
            path.display()
        ));
    }
    Ok(())
}

fn check_adapter(
    expected: &Path,
    read: impl FnOnce() -> Result<Box<[u8]>, String>,
    provisioned: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    adapter_identity(&read()?, expected)?;
    provisioned()
}

fn check_source(
    root: &Path,
    retained: bool,
    bundle: Option<&Path>,
    read: impl FnOnce() -> Result<Source, String>,
    check_bundle: impl FnOnce(Option<&Path>, temper_service::limits::Clients) -> Result<PathBuf, String>,
    check_program: impl FnOnce() -> Result<(), String>,
) -> Result<(Source, Option<PathBuf>), String> {
    let source = read().map_err(|why| if retained { reset(root, &why) } else { why })?;
    // Reuse checks the retained effective bundle, which may differ from the source's directory.
    let provisioned = if retained {
        None
    } else {
        let path = check_bundle(bundle.or(source.bundle.as_deref()), source.bundle_limits)?;
        check_program()?;
        Some(path)
    };
    Ok((source, provisioned))
}

fn check_marker(root: &Path, source: &Source, read: impl FnOnce() -> Result<Box<[u8]>, String>) -> Result<(), String> {
    let prepared = read().map_err(|_| reset(root, "partial preparation: prepared marker is absent or unreadable"))?;
    if prepared != source.marker {
        return Err(reset(root, "incompatible source directories or changed public source documents"));
    }
    Ok(())
}

fn check_required(
    root: &Path,
    dirs: &Directories,
    exists: impl Fn(&Path) -> bool,
    check: impl Fn(&Path) -> Result<(), String>,
) -> Result<(), String> {
    for name in REQUIRED {
        let path = crate::path(&dirs.config).join(name);
        if !exists(&path) {
            return Err(reset(root, &format!("missing required {name}")));
        }
        check(&path).map_err(|why| reset(root, &why))?;
    }
    Ok(())
}

fn new_documents(
    effective: &Directories,
    initial: temper_config_shell::Read,
    bundle: PathBuf,
    adapter: &Path,
    peers: &Peers,
) -> Result<(Box<[u8]>, backends::Documents), String> {
    let paths = Paths {
        config: effective.config.clone(),
        data: effective.data.clone(),
        cache: effective.cache.clone(),
        bundle: bytes(&bundle),
        claude: bytes(adapter),
    };
    let documents = backends::rewrite(&initial.document, &paths, peers)
        .map_err(|why| String::from_utf8_lossy(&why.line()).into_owned())?;
    let mut file = initial.file;
    file.internal.bundle = Some(bundle);
    file.test.github = Some(temper_config_shell::TestGitHub {
        api: peers.github.to_string(),
        transport: temper_config_shell::Transport::Plaintext,
        remote: format!("http://{}", peers.github),
        web: format!("http://{}", peers.github),
        git: Some(crate::path(&effective.data).join("demo/bin/git")),
    });
    Ok((temper_config_shell::render(&file, Shape::Standalone), documents))
}

fn init_ports(chosen: Option<&Chosen>) -> [u16; 2] {
    chosen.map_or([0; 2], |ports| [ports.listeners[0], ports.listeners[4]])
}
// The adapter is this temper's fake-claude sibling; it is a parameter so that tests can provision one.
pub(super) fn prepare(
    source_dirs: &Directories,
    options: &Options,
    adapter: &Path,
    chosen: Option<&Chosen>,
    environment: &[crate::EnvironmentEntry],
) -> Result<Prepared, String> {
    let Startup { directories, bytes, peers, steps, listeners, reservations, lock } =
        prepare_with(source_dirs, options, adapter, chosen, &Native(environment))?;
    let mut listeners = listeners.unwrap_or_else(|| Listeners(Vec::new()));
    for (listener, address) in reservations {
        listeners
            .0
            .push((Fd::new(listener.into_raw_fd()), NonZeroU16::new(address.port()).expect("checked peer port")));
    }
    Ok(Prepared { directories, bytes, listeners, peers, steps, _lock: lock })
}

struct Startup<H: Host> {
    directories: Directories,
    bytes: Box<[u8]>,
    peers: Peers,
    steps: Box<[temper_forge_setup::Step]>,
    listeners: Option<H::Listeners>,
    reservations: Vec<(H::Peer, SocketAddr)>,
    lock: H::Lock,
}

#[expect(clippy::too_many_lines, reason = "preparation holds its lock and listeners and publishes the marker last")]
fn prepare_with<H: Host>(
    source_dirs: &Directories,
    options: &Options,
    adapter: &Path,
    chosen: Option<&Chosen>,
    disk: &H,
) -> Result<Startup<H>, String> {
    let bundle = options.bundle.as_deref();
    let (root, effective) = directories(source_dirs)?;
    let source_config = resolved(Path::new("/"), &source_dirs.config)?;
    if source_config.starts_with(&root) {
        return Err(
            "source configuration overlaps the fake-backend sandbox; choose configuration outside D/box/fake-backends"
                .into(),
        );
    }
    let binary = disk.binary()?;
    let marker = root.join("prepared");
    let exists = match disk.metadata(&root) {
        Ok(_) => true,
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => false,
        Err(why) => return Err(format!("{}: {why}", root.display())),
    };
    let (source, provisioned) = check_source(
        &root,
        exists,
        bundle,
        || disk.source(source_dirs),
        |path, limits| provisioned_bundle(disk, path, limits),
        || provisioned_program_from(adapter, disk.program_metadata(adapter)),
    )?;
    let user = disk.user();
    disk.make_private(crate::path(&source_dirs.data), user)?;
    disk.make_private(root.parent().expect("box parent"), user)?;
    disk.make_private(&root, user).map_err(|why| reset(&root, &why))?;
    let lock = disk.lock(&root)?;
    if exists {
        check_marker(&root, &source, || public_with(disk, &marker))?;
        private_root_with(disk, crate::path(&effective.config), user)
            .map_err(|why| reset(&root, &format!("unsafe effective config: {why}")))?;
        private_root_with(disk, crate::path(&effective.data), user)
            .map_err(|why| reset(&root, &format!("unsafe effective data: {why}")))?;
        private_document(disk, &marker)?;
        check_required(
            &root,
            &effective,
            |path| disk.exists(path),
            |path| disk.public_path(path).and_then(|()| private_document(disk, path)),
        )?;
        let document = disk.read_config(&effective)?.document;
        let content = public_with(disk, &crate::path(&effective.config).join(FILE))?;
        let deployment = crate::setup::check(&document, &jig_directories(&effective))?;
        let peers = checked_peers(disk, &effective, &deployment)?;
        let steps = scripts_for_start(disk, &effective, &peers, options)?;
        let _bytes = replace_bundle_with(disk, &effective, content, bundle, adapter, &deployment)?;
        let providers = backends::providers(&peers).map_err(|why| String::from_utf8_lossy(&why.line()).into_owned())?;
        write_with(disk, &crate::path(&effective.data).join("providers.setting"), &providers, true)?;
        let document = disk.read_config(&effective)?.document;
        return Ok(Startup {
            directories: effective,
            bytes: document,
            listeners: None,
            reservations: Vec::new(),
            peers,
            steps,
            lock,
        });
    }
    let Initialized { paths, ports, listeners } = disk.init(&effective, &binary, init_ports(chosen), adapter)?;
    let mut held = Some(listeners);
    if let Some(chosen) = chosen {
        for (port, wanted) in ports.into_iter().zip(init_ports(Some(chosen))) {
            if port.get() != wanted {
                return Err(format!("{CHOSEN_PORTS}: port {wanted}: init chose port {port} instead"));
            }
        }
    }
    // Init deliberately writes public documents 0644; every retained sandbox document is private.
    for path in paths {
        if disk.metadata(&path).map_err(|why| why.to_string())?.stat.kind == Kind::File {
            disk.set_mode(&path, 0o600).map_err(|why| why.to_string())?;
        }
    }
    if let Some(temper) = source.rebase(&effective) {
        write_with(disk, &crate::path(&effective.config).join(FILE), &temper, true)?;
        // Imported listeners retain their source values; init's new reservations do not represent them.
        drop(held.take());
    }
    let initial = disk.read_config(&effective)?;
    let mut excluded = ordinary_ports(&effective, &initial.document)?;
    let mut reservations = Vec::new();
    for slot in 0..4 {
        let listener = reserve_chosen_with(disk, &excluded, chosen.map(|ports| ports.peers[slot]))?;
        let address = disk.address(&listener).map_err(|why| why.to_string())?;
        excluded.push(address.port());
        reservations.push((listener, address));
    }
    let peers = Peers {
        github: reservations[0].1,
        oauth: reservations[1].1,
        llm: reservations[2].1,
        forgejo: reservations[3].1,
    };
    let bundle = provisioned.expect("new preparation checked the bundle");
    let (text, documents) = new_documents(&effective, initial, bundle, adapter, &peers)?;
    // Existing peer decoders check their generated documents before any marker is published.
    let forge = temper_fake::forge::Document::decode(&documents.github)
        .map_err(|why| String::from_utf8_lossy(&why.line()).into_owned())?;
    let _forgejo = temper_fake::forge::Document::decode(&documents.forgejo)
        .map_err(|why| String::from_utf8_lossy(&why.line()).into_owned())?;
    let _model = temper_fake::llm::Document::decode(&documents.llm)
        .map_err(|why| String::from_utf8_lossy(&why.line()).into_owned())?;
    for (name, document) in [
        (FILE, &text),
        ("github.json", &documents.github),
        ("oauth.json", &documents.oauth),
        ("llm.json", &documents.llm),
        ("forgejo.json", &documents.forgejo),
    ] {
        write_with(disk, &crate::path(&effective.config).join(name), document, name == FILE)?;
    }
    write_with(disk, &crate::path(&effective.data).join("providers.setting"), &documents.providers, false)?;
    write_with(
        disk,
        &crate::path(&effective.data).join("github-app.pem"),
        &forge.github_app_key().expect("generated GitHub App"),
        false,
    )?;
    write_with(disk, &crate::path(&effective.config).join(ADAPTER), adapter.as_os_str().as_bytes(), false)?;
    let deployment = validate_with(disk, &effective, &text, adapter)?;
    let document = disk.read_config(&effective)?.document;
    let peers = checked_peers(disk, &effective, &deployment)?;
    let steps = scripts_for_start(disk, &effective, &peers, options)?;
    // Last: a crash or refusal earlier is visibly partial and is never repaired by importing records.
    write_with(disk, &marker, &source.marker, false)?;
    Ok(Startup { directories: effective, bytes: document, listeners: held, reservations, peers, steps, lock })
}

fn private_root_with(disk: &impl Files, path: &Path, user: u32) -> Result<(), String> {
    temper_shell::directories::private_root_with(user, disk.metadata(path).map(|metadata| metadata.stat))
}
#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "tests/fake_box_ports.rs"]
mod port_tests;

#[cfg(test)]
mod source_lines_tests {
    use super::{Peers, peer_lines, source_forgejo};
    use serde_json::{Value, json};

    fn fixture() -> Value {
        let mut value = json!({
            "listen":{"socket":"127.0.0.1:9104"},"admin_token":"/fixture/admin.token","owner":"person",
            "repositories":[{"name":"existing","landing":"main","ci":null}],
            "sources":[]
        });
        value["sources"] = ["widget", "gadget"]
            .iter()
            .map(|name| {
                json!({
                    "address":format!("https://code.example.invalid/acme/{name}"),
                    "repository":{"owner":"acme","name":name,"landing":"main","tree":[]}
                })
            })
            .collect();
        value
    }

    #[test]
    fn copy_sources_follow_forgejo_once_in_document_order() {
        let peers = Peers {
            github: "127.0.0.1:9101".parse().expect("peer"),
            oauth: "127.0.0.1:9102".parse().expect("peer"),
            llm: "127.0.0.1:9103".parse().expect("peer"),
            forgejo: "127.0.0.1:9104".parse().expect("peer"),
        };
        let document = source_forgejo(&serde_json::to_vec(&fixture()).expect("fixture JSON")).expect("source document");
        assert_eq!(
            peer_lines(&peers, &document),
            [
                "box: fake github http://127.0.0.1:9101",
                "box: fake issuer http://127.0.0.1:9102",
                "box: fake model http://127.0.0.1:9103",
                "box: fake forgejo http://127.0.0.1:9104",
                "box: copy source https://code.example.invalid/acme/widget",
                "box: copy source https://code.example.invalid/acme/gadget",
            ]
        );
    }

    #[test]
    fn retained_forgejo_without_copy_sources_requires_reset() {
        let mut value = fixture();
        value.as_object_mut().expect("document object").remove("sources");
        for sources in [None, Some(json!([]))] {
            if let Some(sources) = sources {
                value["sources"] = sources;
            }
            assert_eq!(
                source_forgejo(&serde_json::to_vec(&value).expect("fixture JSON"))
                    .expect_err("missing sources refused")
                    .line()
                    .as_ref(),
                b"sources: missing copy sources; remove this sandbox directory explicitly to reset it\n"
            );
        }
    }
}

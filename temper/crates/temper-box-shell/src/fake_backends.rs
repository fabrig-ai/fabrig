//! Retained isolated startup for `temper-box --fake-backends` (protocol/configuration/directories.md, 1).
use crate::directories::jig_directories;
mod import;
pub(crate) mod json;
use crate::directories::{Directories, make_private, private_root};
use import::Source;
use json::Json;
use skein_io::kernel::{Exit, Fd};
use std::{
    ffi::OsStr,
    fs::{self, OpenOptions},
    io::Write,
    net::{SocketAddr, TcpListener},
    num::NonZeroU16,
    os::fd::IntoRawFd,
    os::unix::{
        ffi::OsStrExt,
        fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    },
    path::{Component, Path, PathBuf},
};
use temper_fake::backends::{self, Paths, Peers};
use temper_forge_setup_shell::process as locks;

const REQUIRED: &[&str] = &["temper.json", "agent.json", "github.json", "oauth.json", "llm.json", "forgejo.json"];

/// A new sandbox's nine ports, the process tier's (temper/docs/design/protocol/setup/box.md, 5).
pub(crate) const FAKE_BOX_PORTS: skein_shell::Variable = skein_shell::Variable {
    name: "TEMPER_INTERNAL_FAKE_BOX_PORTS",
    set: skein_shell::Set::Internal,
    form: "PORTS",
    line: "Choose the fake box's nine listener and peer ports.",
    absent: "Reserve available ports.",
    notes: "Nine comma-separated nonzero ports; used only for a new sandbox.",
    way: skein_shell::Way::Reads,
};
const CHOSEN_PORTS: &str = FAKE_BOX_PORTS.name;
fn ports_from(environment: &[crate::EnvironmentEntry]) -> Result<Option<Chosen>, String> {
    FAKE_BOX_PORTS.value(environment).map(OsStr::from_bytes).map(chosen_ports).transpose()
}
struct Chosen {
    listeners: [u16; 5],
    peers: [u16; 4],
}
fn chosen_ports(value: &OsStr) -> Result<Chosen, String> {
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

struct Listeners(Vec<(Fd, NonZeroU16)>);
impl Drop for Listeners {
    fn drop(&mut self) {
        crate::close(std::mem::take(&mut self.0).into_iter().map(|(fd, _)| fd).collect());
    }
}
struct Prepared {
    directories: Directories,
    bytes: Box<[u8]>,
    listeners: Listeners,
    peers: Peers,
    steps: Box<[temper_forge_setup::Step]>,
    // Held through drive; closing the descriptor releases the OS lock even after a crash.
    _lock: locks::Lock,
}
#[derive(Default)]
pub(crate) struct Options {
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
/// Per-start scripts and arguments; retained peer documents are never rewritten (protocol/setup/box.md, 5).
fn scripts_for_start(
    dirs: &Directories,
    peers: &Peers,
    options: &Options,
) -> Result<Box<[temper_forge_setup::Step]>, String> {
    let path = crate::path(&dirs.config).join("llm-scripts.json");
    if options.scripts.is_empty() {
        match fs::remove_file(&path) {
            Ok(()) => {}
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => {}
            Err(why) => return Err(format!("{}: {why}", path.display())),
        }
        return Ok(backends::steps(&dirs.config, &dirs.data, peers, None, options.strict));
    }
    let document = peer_document(dirs, "llm.json", temper_fake::DOCUMENT_BYTES)?;
    let joined = join_scripts(&options.scripts, &document)?;
    write(&path, &joined, path.exists())?;
    Ok(backends::steps(&dirs.config, &dirs.data, peers, Some(&bytes(&path)), options.strict))
}
fn join_scripts(files: &[PathBuf], document: &[u8]) -> Result<Box<[u8]>, String> {
    let mut joined = Json::Object(
        ["cued", "asked", "otherwise"]
            .into_iter()
            .map(|name| (name.as_bytes().into(), Json::Array(Vec::new())))
            .collect(),
    );
    let mut content = joined.encode()?;
    for file in files {
        let refuse = |why| format!("{}: {why}", file.display());
        let source =
            skein_shell::read_file(file, temper_fake::DOCUMENT_BYTES).map_err(|why| refuse(crate::unread(why)))?;
        let check = |scripts: &[u8]| {
            temper_fake::llm::Document::decode_with_scripts(document, Some(scripts))
                .map_err(|why| refuse(String::from_utf8_lossy(&why.line()).into_owned()))
        };
        check(&source)?;
        let Json::Object(members) = Json::decode(&source).map_err(&refuse)? else {
            unreachable!("script reader checked the object");
        };
        for (name, value) in members {
            let name = std::str::from_utf8(&name).expect("checked script member");
            let target = joined.at_mut(name).map_err(&refuse)?;
            if name == "otherwise" {
                *target = value;
            } else {
                let (Json::Array(target), Json::Array(mut given)) = (target, value) else {
                    unreachable!("script reader checked the arrays");
                };
                target.append(&mut given);
            }
        }
        content = joined.encode().map_err(&refuse)?;
        check(&content)?;
    }
    Ok(content)
}
fn bytes(path: &Path) -> Box<[u8]> {
    path.as_os_str().as_bytes().into()
}
fn public_path(path: &Path) -> Result<(), String> {
    // Public imports must not turn a link into an accidental source-secret copy.
    let mut prefix = PathBuf::new();
    for component in path.components() {
        prefix.push(component.as_os_str());
        let metadata = fs::symlink_metadata(&prefix).map_err(|why| format!("{}: {why}", prefix.display()))?;
        if metadata.file_type().is_symlink() {
            return Err(format!("{}: public documents must not be reached through links", prefix.display()));
        }
    }
    let metadata = fs::symlink_metadata(path).map_err(|why| format!("{}: {why}", path.display()))?;
    if !metadata.is_file() || metadata.nlink() != 1 {
        return Err(format!("{}: expected a regular public document with one link", path.display()));
    }
    Ok(())
}
fn public(path: &Path) -> Result<Box<[u8]>, String> {
    public_path(path)?;
    skein_shell::read_file(path, crate::CONFIG_BYTES)
        .map_err(|why| format!("{}: {}", path.display(), crate::unread(why)))
}
fn private_document(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|why| format!("{}: {why}", path.display()))?;
    if metadata.uid() != skein_shell::effective_user() || metadata.mode() & 0o077 != 0 {
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
fn write(path: &Path, content: &[u8], replace: bool) -> Result<(), String> {
    if replace {
        let metadata = fs::symlink_metadata(path).map_err(|why| format!("{}: {why}", path.display()))?;
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || metadata.uid() != skein_shell::effective_user()
            || metadata.nlink() != 1
        {
            return Err(format!("{}: expected this user's regular file", path.display()));
        }
    }
    let mut options = OpenOptions::new();
    options.write(true).mode(0o600);
    if replace {
        options.truncate(true);
    } else {
        options.create_new(true);
    }
    let mut file = options.open(path).map_err(|why| format!("{}: {why}", path.display()))?;
    file.set_permissions(fs::Permissions::from_mode(0o600)).map_err(|why| why.to_string())?;
    file.write_all(content).map_err(|why| format!("{}: {why}", path.display()))?;
    file.sync_all().map_err(|why| format!("{}: {why}", path.display()))
}
fn replace_bundle(
    dirs: &Directories,
    content: Box<[u8]>,
    bundle: Option<&Path>,
    adapter: &Path,
) -> Result<Box<[u8]>, String> {
    let mut updated = content.clone();
    if let Some(bundle) = bundle {
        let mut document = Json::decode(&content)?;
        let selection = Json::String(bytes(bundle));
        if document.at("application/clients/bundle")? != &selection {
            document.set("application/clients/bundle", selection)?;
            updated = document.encode()?;
        }
    }
    // Check the entire candidate, including its bundle, before touching retained bytes.
    validate(dirs, &updated, adapter)?;
    if updated == content {
        return Ok(content);
    }
    let path = dirs.document();
    let staging = crate::path(&dirs.config).join("temper.json.bundle-staging");
    let mut file = OpenOptions::new().write(true).create_new(true).mode(0o600).open(&staging).map_err(|why| {
        format!(
            "{}: {why}; an existing staging file must be removed explicitly before another bundle replacement",
            staging.display()
        )
    })?;
    let result = (|| {
        file.write_all(&updated)?;
        file.sync_all()?;
        fs::rename(&staging, &path)
    })();
    if let Err(why) = result {
        let _removed = fs::remove_file(&staging);
        return Err(format!("{}: bundle replacement: {why}", path.display()));
    }
    Ok(updated)
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

fn reserve(excluded: &[u16]) -> Result<TcpListener, String> {
    reserve_from(excluded, || TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)))
}
fn reserve_from(
    excluded: &[u16],
    mut bind: impl FnMut() -> std::io::Result<TcpListener>,
) -> Result<TcpListener, String> {
    for _ in 0..PORT_ATTEMPTS {
        let listener = match bind() {
            Ok(listener) => listener,
            Err(why) if why.kind() == std::io::ErrorKind::AddrInUse => continue,
            Err(why) => return Err(format!("fake peer reservation: {why}")),
        };
        if !excluded.contains(&listener.local_addr().map_err(|why| why.to_string())?.port()) {
            return Ok(listener);
        }
    }
    Err("fake peer reservation: could not choose a port distinct from the deployment and callback".into())
}
fn reserve_chosen(excluded: &[u16], port: Option<u16>) -> Result<TcpListener, String> {
    match port {
        Some(port) => TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port))
            .map_err(|why| format!("{CHOSEN_PORTS}: port {port}: {why}")),
        None => reserve(excluded),
    }
}
fn addr(listener: &TcpListener) -> Result<SocketAddr, String> {
    listener.local_addr().map_err(|why| why.to_string())
}
fn contained(root: &[u8], path: &[u8]) -> bool {
    crate::path(path)
        .strip_prefix(crate::path(root))
        .is_ok_and(|rest| !rest.as_os_str().is_empty() && rest.components().all(|p| matches!(p, Component::Normal(_))))
}
fn peer_document(dirs: &Directories, name: &str, bound: u32) -> Result<Box<[u8]>, String> {
    let path = crate::path(&dirs.config).join(name);
    public_path(&path)?;
    private_document(&path)?;
    skein_shell::read_private_file(&path, bound, skein_shell::effective_user())
        .map(|bytes| bytes.expose().into())
        .map_err(|why| format!("{}: {}", path.display(), crate::unread(why)))
}
/// The ports no peer may take: the callback's and the ordinary box's listeners and readiness
/// (temper/docs/design/protocol/setup/box.md, 5).
fn ordinary_ports(dirs: &Directories, bytes: &[u8]) -> Result<Vec<u16>, String> {
    let (ordinary, _) = crate::box_command::configuration_steps(dirs, bytes, Box::new([]))?;
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
fn checked_peers(dirs: &Directories, bytes: &[u8]) -> Result<Peers, String> {
    let mut ports = ordinary_ports(dirs, bytes)?;
    let refuse = |name: &str, why: temper_fake::Refusal| {
        format!("{}: {}", crate::path(&dirs.config).join(name).display(), String::from_utf8_lossy(&why.line()))
    };
    let github =
        temper_fake::forge::Document::decode(&peer_document(dirs, "github.json", temper_fake::DOCUMENT_BYTES)?)
            .map_err(|why| refuse("github.json", why))?;
    let oauth =
        temper_fake::oauth::Document::decode(&peer_document(dirs, "oauth.json", temper_fake::oauth::DOCUMENT_BYTES)?)
            .map_err(|why| refuse("oauth.json", why))?;
    let llm = temper_fake::llm::Document::decode(&peer_document(dirs, "llm.json", temper_fake::DOCUMENT_BYTES)?)
        .map_err(|why| refuse("llm.json", why))?;
    let forgejo = source_forgejo(&peer_document(dirs, "forgejo.json", temper_fake::DOCUMENT_BYTES)?)
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
fn validate(dirs: &Directories, bytes: &[u8], expected: &Path) -> Result<(), String> {
    let deployment = crate::setup::check(bytes, &jig_directories(dirs))?;
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
        || named.worker.as_ref().is_none_or(|worker| {
            !contained(&dirs.data, &worker.root)
                || worker.agent.as_ref() != crate::path(&dirs.config).join("agent.json").as_os_str().as_bytes()
        })
    {
        return Err("effective deployment, worker root and agent configuration must stay in the sandbox".into());
    }
    let tree = Json::decode(bytes)?;
    let configuration = tree.string("workers/0/agent/configuration")?;
    let argv = tree.at("workers/0/agent/arguments")?;
    let wanted = Json::Array(vec![
        Json::String(b"internal".as_slice().into()),
        Json::String(b"agent".as_slice().into()),
        Json::String(configuration.into()),
    ]);
    argv.same(&wanted, "workers/0/agent/arguments")?;
    let agent_document = public(&crate::path(&dirs.config).join("agent.json"))?;
    let agent_tree = Json::decode(&agent_document)?;
    let adapter = agent_tree.string("programs/claude_code/path")
        .map_err(|_| "agent.json/programs/claude_code/path: unsupported adapter; remove this sandbox directory explicitly to reset it".to_string())?;
    adapter_identity(adapter, expected)?;
    provisioned_program(expected)?;
    if tree.string("workers/0/agent/program")?
        != std::env::current_exe().map_err(|why| why.to_string())?.as_os_str().as_bytes()
    {
        return Err("workers/0/agent/program: expected this temper binary".into());
    }
    for file in &named.files {
        if !file.private {
            let asset = public(crate::path(&file.path))?;
            if asset.len() > usize::try_from(file.max).expect("file bound fits") {
                return Err(format!("{}: public asset past its byte bound", crate::file_path(file.at.clone())));
            }
        }
    }
    let agent = temper_worker_shell::agent::preflight_worker(
        named.worker.as_ref().ok_or("workers: standalone takes one worker")?,
    )?;
    for program in agent.programs {
        if !contained(&dirs.data, &program.directory) {
            return Err("agent.programs.directory: must stay under effective data".into());
        }
    }
    match &named.bundle {
        Some(bundle) => {
            let _checked = provisioned_bundle(Some(crate::path(bundle)), named.bundle_limits)?;
        }
        None => return Err("application.clients.bundle: effective document must name its bundle".into()),
    }
    Ok(())
}
fn provisioned_bundle(path: Option<&Path>, limits: temper_service::limits::Clients) -> Result<PathBuf, String> {
    let instructions =
        "run scripts/bundle.sh DIR && temper-box --fake-backends --bundle BUNDLE (directory flags as usual)";
    let path = path.ok_or_else(|| format!("application.clients.bundle: no provisioned bundle; {instructions}"))?;
    let page = path.join("index.html");
    public_path(&page)
        .map_err(|why| format!("application.clients.bundle: {}: {why}; {instructions}", page.display()))?;
    let _checked =
        crate::bundle::read(path.as_os_str().as_bytes(), limits).map_err(|why| format!("{why}; {instructions}"))?;
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
        return Err("agent.json/programs/claude_code/path: expected this temper's fake-claude sibling; remove this sandbox directory explicitly to reset it".into());
    }
    Ok(())
}
fn provisioned_program(path: &Path) -> Result<(), String> {
    let executable = fs::metadata(path).is_ok_and(|meta| meta.is_file() && meta.mode() & 0o111 != 0);
    if !executable {
        return Err(format!(
            "{}: missing or nonexecutable adapter; provision it with cargo build -p smith-fake-claude --bin fake-claude",
            path.display()
        ));
    }
    Ok(())
}
// The adapter is this temper's fake-claude sibling; it is a parameter so that tests can provision one.
#[expect(
    clippy::too_many_lines,
    reason = "startup owns its lock and listeners through validation and publishes the prepared marker last"
)]
fn prepare(
    source_dirs: &Directories,
    options: &Options,
    adapter: &Path,
    chosen: Option<&Chosen>,
    environment: &[crate::EnvironmentEntry],
) -> Result<Prepared, String> {
    let bundle = options.bundle.as_deref();
    let (root, effective) = directories(source_dirs)?;
    let source_config = resolved(Path::new("/"), &source_dirs.config)?;
    if source_config.starts_with(&root) {
        return Err(
            "source configuration overlaps the fake-backend sandbox; choose configuration outside D/box/fake-backends"
                .into(),
        );
    }
    let binary = std::env::current_exe().map_err(|why| why.to_string())?;
    let marker = root.join("prepared");
    let exists = match fs::symlink_metadata(&root) {
        Ok(_) => true,
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => false,
        Err(why) => return Err(format!("{}: {why}", root.display())),
    };
    let source = Source::read(source_dirs, environment).map_err(|why| if exists { reset(&root, &why) } else { why })?;
    // Reuse checks the retained effective bundle, which may differ from the source's directory.
    let provisioned = if exists {
        None
    } else {
        Some(provisioned_bundle(bundle.or(source.bundle.as_deref()), source.bundle_limits)?)
    };
    if !exists {
        provisioned_program(adapter)?;
    }
    let user = skein_shell::effective_user();
    make_private(crate::path(&source_dirs.data), user)?;
    make_private(root.parent().expect("box parent"), user)?;
    make_private(&root, user).map_err(|why| reset(&root, &why))?;
    let lock = lock(&root)?;
    if exists {
        let prepared = public(&marker)
            .map_err(|_| reset(&root, "partial preparation: prepared marker is absent or unreadable"))?;
        if prepared != source.marker {
            return Err(reset(&root, "incompatible source directories or changed public source documents"));
        }
        private_root(crate::path(&effective.config), user)
            .map_err(|why| reset(&root, &format!("unsafe effective config: {why}")))?;
        private_root(crate::path(&effective.data), user)
            .map_err(|why| reset(&root, &format!("unsafe effective data: {why}")))?;
        private_document(&marker)?;
        for name in REQUIRED {
            let path = crate::path(&effective.config).join(name);
            if !path.exists() {
                return Err(reset(&root, &format!("missing required {name}")));
            }
            public_path(&path).and_then(|()| private_document(&path)).map_err(|why| reset(&root, &why))?;
        }
        let content = public(&effective.document())?;
        let peers = checked_peers(&effective, &content)?;
        let steps = scripts_for_start(&effective, &peers, options)?;
        let bytes = replace_bundle(&effective, content, bundle, adapter)?;
        return Ok(Prepared {
            directories: effective,
            bytes,
            listeners: Listeners(Vec::new()),
            peers,
            steps,
            _lock: lock,
        });
    }
    let written = crate::init::write_with_program(
        &effective,
        environment,
        &binary,
        chosen.map_or([0; 5], |ports| ports.listeners),
        Some(adapter),
    )?;
    let mut held = Listeners(written.listeners);
    if let Some(chosen) = chosen {
        for ((_, port), wanted) in held.0.iter().zip(chosen.listeners) {
            if port.get() != wanted {
                return Err(format!("{CHOSEN_PORTS}: port {wanted}: init chose port {port} instead"));
            }
        }
    }
    // Init deliberately writes public documents 0644; every retained sandbox document is private.
    for path in written.paths {
        if fs::symlink_metadata(&path).map_err(|why| why.to_string())?.is_file() {
            fs::set_permissions(path, fs::Permissions::from_mode(0o600)).map_err(|why| why.to_string())?;
        }
    }
    if let Some((temper, agent)) = source.rebase(&effective)? {
        write(&effective.document(), &temper, true)?;
        write(&crate::path(&effective.config).join("agent.json"), &agent, true)?;
        // Imported listeners retain their source values; init's new reservations do not represent them.
        crate::close(std::mem::take(&mut held.0).into_iter().map(|(fd, _)| fd).collect());
    }
    let initial = public(&effective.document())?;
    let mut excluded = ordinary_ports(&effective, &initial)?;
    let github = reserve_chosen(&excluded, chosen.map(|ports| ports.peers[0]))?;
    excluded.push(addr(&github)?.port());
    let oauth = reserve_chosen(&excluded, chosen.map(|ports| ports.peers[1]))?;
    excluded.push(addr(&oauth)?.port());
    let llm = reserve_chosen(&excluded, chosen.map(|ports| ports.peers[2]))?;
    excluded.push(addr(&llm)?.port());
    let forgejo = reserve_chosen(&excluded, chosen.map(|ports| ports.peers[3]))?;
    let peers = Peers { github: addr(&github)?, oauth: addr(&oauth)?, llm: addr(&llm)?, forgejo: addr(&forgejo)? };
    let bundle = provisioned.expect("new preparation checked the bundle");
    let paths = Paths {
        config: effective.config.clone(),
        data: effective.data.clone(),
        cache: effective.cache.clone(),
        bundle: bytes(&bundle),
        claude: bytes(adapter),
    };
    let documents =
        backends::rewrite(&initial, &public(&crate::path(&effective.config).join("agent.json"))?, &paths, &peers)
            .map_err(|why| String::from_utf8_lossy(&why.line()).into_owned())?;
    // Existing peer decoders check their generated documents before any marker is published.
    let forge = temper_fake::forge::Document::decode(&documents.github)
        .map_err(|why| String::from_utf8_lossy(&why.line()).into_owned())?;
    let _forgejo = temper_fake::forge::Document::decode(&documents.forgejo)
        .map_err(|why| String::from_utf8_lossy(&why.line()).into_owned())?;
    let _model = temper_fake::llm::Document::decode(&documents.llm)
        .map_err(|why| String::from_utf8_lossy(&why.line()).into_owned())?;
    for (name, document) in [
        ("temper.json", &documents.temper),
        ("agent.json", &documents.agent),
        ("github.json", &documents.github),
        ("oauth.json", &documents.oauth),
        ("llm.json", &documents.llm),
        ("forgejo.json", &documents.forgejo),
    ] {
        write(&crate::path(&effective.config).join(name), document, name == "temper.json" || name == "agent.json")?;
    }
    write(
        &crate::path(&effective.data).join("github-app.pem"),
        &forge.github_app_key().expect("generated GitHub App"),
        false,
    )?;
    validate(&effective, &documents.temper, adapter)?;
    let peers = checked_peers(&effective, &documents.temper)?;
    let steps = scripts_for_start(&effective, &peers, options)?;
    // Last: a crash or refusal earlier is visibly partial and is never repaired by importing records.
    write(&marker, &source.marker, false)?;
    for (listener, address) in
        [(github, peers.github), (oauth, peers.oauth), (llm, peers.llm), (forgejo, peers.forgejo)]
    {
        held.0.push((Fd::new(listener.into_raw_fd()), NonZeroU16::new(address.port()).expect("checked peer port")));
    }
    Ok(Prepared { directories: effective, bytes: documents.temper, listeners: held, peers, steps, _lock: lock })
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

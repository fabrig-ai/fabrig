//! Startup refusals over named-file facts (temper/docs/design/testing/box/README.md, 3.9–3.10).
use super::super::{Inputs, start_with};
use crate::{Command, Directories, binary};
use skein_io::{
    file::Unsafe,
    kernel::{Kind, Stat},
};
use skein_lib::Secret;
use skein_shell::Unread;
use std::{
    collections::BTreeMap,
    io,
    path::{Path, PathBuf},
    process::ExitCode,
};
use temper_forge_setup::{setup::Entry, setup_file};

use super::super::disk::{Disk, Metadata};

const ROOT: &str = "/fixture";
const DOCUMENT: &str = "/fixture/document.json";
const TOKEN: &str = "/fixture/forge/admin_token";
const KEY: &str = "/fixture/key";
const ADDED_KEY: &str = "/fixture/second.key";
const ROOT_DER: &str = "/fixture/root.der";
const OBSERVATIONS: &str = "/fixture/observations";
const DEMO_KEY: &str = "/fixture/demo/llm.key";
const CONFIG: &str = "/fixture/temper.toml";
const STATE: &str = "/fixture/forge.state";
const EXISTING: &[u8] = b"existing";
const ADMIN: &[u8; 40] = b"0123456789abcdef0123456789abcdef01234567";
const USER: u32 = 42;
const DIFFERENT: &str = "retained output differs from expected seeded bytes";
const EXISTS: &str = "File exists (os error 17)";
const BAD_TOKEN: &str = "expected 40 lowercase hexadecimal digits";
const FORGE_USAGE: &str = concat!(
    "temper-box fake forge: invalid arguments\n",
    "usage: temper-box fake forge [DIRECTORIES] FORGE [--ready PREFIX] [--seed N] [--observations FILE] [--state STATE]\n",
    "usage: temper-box fake forge --document FILE [--ready PREFIX] [--seed N] [--observations FILE] [--state STATE]\n",
);
const LLM_USAGE: &str = concat!(
    "temper-box fake llm: invalid arguments\n",
    "usage: temper-box fake llm [DIRECTORIES] [--ready PREFIX] [--seed N] [--observations FILE] [--strict]\n",
    "usage: temper-box fake llm --document FILE [--ready PREFIX] [--seed N] [--observations FILE] [--scripts SCRIPTS] [--strict]\n",
);

#[derive(Clone, Debug, PartialEq, Eq)]
struct Node {
    stat: Stat,
    bytes: Vec<u8>,
}
fn node(kind: Kind, mode: u32, bytes: &[u8]) -> Node {
    Node {
        stat: Stat { kind, mode, owner: USER, links: 1, size: skein_lib::len_u64(bytes.len()) },
        bytes: bytes.into(),
    }
}
fn file(bytes: &[u8]) -> Node {
    node(Kind::File, 0o600, bytes)
}
fn directory(mode: u32) -> Node {
    node(Kind::Directory, mode, b"")
}

type Inventory = BTreeMap<PathBuf, Node>;
#[derive(Debug, PartialEq, Eq)]
enum Effect {
    Create(PathBuf, u32),
    Write(PathBuf),
    MakeDirectory(PathBuf, u32),
    RemoveFile(PathBuf),
    RemoveDirectory(PathBuf),
}
struct Facts {
    tree: Inventory,
    effects: Vec<Effect>,
    opened: Vec<PathBuf>,
    reads: Vec<(PathBuf, u32, bool)>,
}
impl Facts {
    fn new(entries: impl IntoIterator<Item = (&'static str, Node)>) -> Self {
        let tree =
            [(ROOT, directory(0o700))].into_iter().chain(entries).map(|(name, node)| (name.into(), node)).collect();
        Self { tree, effects: Vec::new(), opened: Vec::new(), reads: Vec::new() }
    }
    fn get(&self, path: &Path) -> io::Result<&Node> {
        self.tree.get(path).ok_or_else(|| io::Error::from_raw_os_error(2))
    }
}
impl Disk for Facts {
    type File = PathBuf;
    fn user(&self) -> u32 {
        USER
    }
    fn metadata(&mut self, path: &Path) -> io::Result<Metadata> {
        Ok(Metadata { stat: self.get(path)?.stat, identity: (1, 2) })
    }
    fn open(&mut self, path: &Path) -> io::Result<PathBuf> {
        self.opened.push(path.into());
        self.get(path)?;
        Ok(path.into())
    }
    fn opened_metadata(&mut self, path: &PathBuf) -> io::Result<Metadata> {
        self.metadata(path)
    }
    fn read(&mut self, path: PathBuf, bound: u64) -> io::Result<Box<[u8]>> {
        let bytes = &self.get(&path)?.bytes;
        Ok(bytes[..bytes.len().min(usize::try_from(bound).expect("bounded fixture read"))].into())
    }
    fn create(&mut self, path: &Path, mode: u32) -> io::Result<PathBuf> {
        self.effects.push(Effect::Create(path.into(), mode));
        self.get(path.parent().expect("output parent"))?;
        match self.tree.entry(path.into()) {
            std::collections::btree_map::Entry::Occupied(_) => Err(io::Error::from_raw_os_error(17)),
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(node(Kind::File, mode, b""));
                Ok(path.into())
            }
        }
    }
    fn write(&mut self, path: &mut PathBuf, bytes: &[u8]) -> io::Result<()> {
        self.effects.push(Effect::Write(path.clone()));
        let node = self.tree.get_mut(path).expect("created file");
        node.stat.size = skein_lib::len_u64(bytes.len());
        node.bytes = bytes.into();
        Ok(())
    }
    fn sync(&mut self, _: &PathBuf) -> io::Result<()> {
        panic!("no state stored on refused start")
    }
    fn rename(&mut self, _: &Path, _: &Path) -> io::Result<()> {
        panic!("no state stored on refused start")
    }
    fn create_directories(&mut self, path: &Path, mode: u32) -> io::Result<()> {
        let missing: Vec<_> = path.ancestors().take_while(|path| !self.tree.contains_key(*path)).collect();
        for path in missing.into_iter().rev() {
            self.effects.push(Effect::MakeDirectory(path.into(), mode));
            self.tree.insert(path.into(), directory(mode));
        }
        Ok(())
    }
    fn remove_file(&mut self, path: &Path) -> io::Result<()> {
        self.effects.push(Effect::RemoveFile(path.into()));
        self.tree.remove(path).expect("owned output");
        Ok(())
    }
    fn remove_directory(&mut self, path: &Path) -> io::Result<()> {
        self.effects.push(Effect::RemoveDirectory(path.into()));
        assert!(
            !self.tree.keys().any(|entry| entry != path && entry.starts_with(path)),
            "remove children before parent"
        );
        self.tree.remove(path).expect("owned directory");
        Ok(())
    }
}
impl Inputs for Facts {
    fn document(&mut self, path: &Path, bound: u32) -> Result<Box<[u8]>, Unread> {
        self.reads.push((path.into(), bound, false));
        bytes(self.tree.get(path).ok_or(Unread::Absent)?, bound)
    }
    fn private_file(&mut self, path: &Path, bound: u32) -> Result<Secret, Unread> {
        self.reads.push((path.into(), bound, true));
        let node = self.tree.get(path).ok_or(Unread::Absent)?;
        if let Some(why) = Unsafe::private_file(node.stat, USER) {
            return Err(Unread::Unsafe(why));
        }
        bytes(node, bound).map(Secret::new)
    }
    fn forge_entry(&mut self, directories: &Directories, forge: u16) -> Result<Entry, String> {
        assert_eq!(directories.data.as_ref(), ROOT.as_bytes());
        assert_eq!(forge, 1);
        let dirs = crate::directories::jig_directories(directories);
        let setup = setup_file::decode(
            br#"{"version":1,"kind":"fake","listen":{"socket":"127.0.0.1:9"},"remote":"http://127.0.0.1:9","git":"demo/bin/git"}"#,
            &dirs,
        ).expect("fixture setup");
        Ok(setup.entry(forge, &dirs).expect("setup entry"))
    }
}
fn bytes(node: &Node, bound: u32) -> Result<Box<[u8]>, Unread> {
    if node.stat.size > u64::from(bound) {
        return Err(Unread::TooLarge { max: bound });
    }
    Ok(node.bytes.as_slice().into())
}

fn refusal(facts: &mut Facts, kind: &str, words: &[&str], no_run: bool) -> (u8, String) {
    let before = facts.tree.clone();
    let mut stdout = String::new();
    let arguments = ["fake", kind].into_iter().chain(words.iter().copied());
    let environment = [("TEMPER_CONFIG", ROOT), ("TEMPER_DATA", ROOT), ("TEMPER_CACHE", "/fixture/cache")]
        .into_iter()
        .map(|(key, value)| (key.as_bytes().into(), value.as_bytes().into()))
        .collect();
    let result: Result<ExitCode, _> = binary::dispatch_fixture(
        arguments,
        environment,
        ROOT.into(),
        |line| stdout.push_str(line),
        |command, _| {
            assert!(!no_run, "usage refusal must not run");
            let Command::Fake { arguments } = command else { panic!("fake command") };
            start_with(&arguments, facts, Ok(Box::new([])), |_, _| panic!("refused before opening host"))
        },
    );
    assert!(stdout.is_empty(), "{stdout}");
    assert_eq!(facts.tree, before, "refusal leaves the full inventory unchanged");
    result.expect_err("refused fake start")
}
fn expected(kind: &str, path: Option<&str>, reason: &str) -> (u8, String) {
    let location = path.map_or_else(String::new, |path| format!("{path}: "));
    (1, format!("temper fake {kind}: {location}{reason}\n"))
}

fn llm_document(tls: bool, added: bool) -> serde_json::Value {
    serde_json::json!({"listen":{"socket":"127.0.0.1:0"}, "key":KEY,
        "tls":if tls {serde_json::json!({"root":ROOT_DER})} else {serde_json::Value::Null},
        "added_keys":if added {vec![ADDED_KEY]} else {vec![]},
        "models":[{"name":"qa","window":4096,"output":1024}], "cued":[], "asked":[], "otherwise":["fallback"]})
}
const FORGE_DOCUMENT: &[u8] = br#"{"listen":{"socket":"127.0.0.1:0"},"admin_token":"/fixture/forge/admin_token","owner":"fake","repositories":[{"name":"widget","landing":"main","ci":null}]}"#;
fn json_node(document: &serde_json::Value) -> Node {
    file(&serde_json::to_vec(document).expect("fixture document"))
}
fn refused_documents() {
    const FORGE_DIR: &str = "/fixture/forge";
    let llm = || json_node(&llm_document(true, false));
    let forge = || file(FORGE_DOCUMENT);
    for (name, kind, entries, observations, path, reason, effects) in [
        ("key", "llm", vec![(DOCUMENT, llm()), (KEY, file(EXISTING))], false, Some(KEY), DIFFERENT, vec![]),
        (
            "added_key",
            "llm",
            vec![(DOCUMENT, json_node(&llm_document(true, true))), (ADDED_KEY, file(EXISTING))],
            false,
            Some(ADDED_KEY),
            DIFFERENT,
            vec![Effect::Create(KEY.into(), 0o600), Effect::Write(KEY.into()), Effect::RemoveFile(KEY.into())],
        ),
        (
            "admin_token",
            "forge",
            vec![(DOCUMENT, forge()), (FORGE_DIR, directory(0o700)), (TOKEN, file(EXISTING))],
            false,
            Some(TOKEN),
            DIFFERENT,
            vec![],
        ),
        (
            "root",
            "llm",
            vec![(DOCUMENT, json_node(&llm_document(true, true))), (ROOT_DER, file(EXISTING))],
            false,
            Some(ROOT_DER),
            DIFFERENT,
            vec![
                Effect::Create(KEY.into(), 0o600),
                Effect::Write(KEY.into()),
                Effect::Create(ADDED_KEY.into(), 0o600),
                Effect::Write(ADDED_KEY.into()),
                Effect::RemoveFile(ADDED_KEY.into()),
                Effect::RemoveFile(KEY.into()),
            ],
        ),
        (
            "observations",
            "forge",
            vec![(DOCUMENT, forge()), (OBSERVATIONS, file(EXISTING))],
            true,
            Some(OBSERVATIONS),
            EXISTS,
            vec![
                Effect::MakeDirectory(FORGE_DIR.into(), 0o700),
                Effect::Create(TOKEN.into(), 0o600),
                Effect::Write(TOKEN.into()),
                Effect::Create(OBSERVATIONS.into(), 0o600),
                Effect::RemoveFile(TOKEN.into()),
                Effect::RemoveDirectory(FORGE_DIR.into()),
            ],
        ),
        (
            "directory",
            "forge",
            vec![(DOCUMENT, forge()), (FORGE_DIR, directory(0o755))],
            false,
            Some(FORGE_DIR),
            "must be a private directory of this user's, not reached through a link",
            vec![],
        ),
        ("missing", "llm", vec![], false, Some(DOCUMENT), "absent", vec![]),
        ("refused", "llm", vec![(DOCUMENT, file(b"{}"))], false, None, "listen: missing", vec![]),
    ] {
        let mut facts = Facts::new(entries);
        let mut words = vec!["--document", DOCUMENT];
        if observations {
            words.extend(["--observations", OBSERVATIONS]);
        }
        assert_eq!(refusal(&mut facts, kind, &words, false), expected(kind, path, reason), "{name}");
        assert_eq!(facts.effects, effects, "{name}: ordered output effects");
        assert_eq!(facts.reads, [(DOCUMENT.into(), temper_fake::DOCUMENT_BYTES, false)], "{name}");
    }
}
fn refused_setup() {
    const SETUP_TOKEN: &str = "/fixture/playground-1/admin_token";
    for (name, token, extra, expected, effects) in [
        ("absent", None, &[][..], expected("forge", Some(SETUP_TOKEN), "absent"), vec![]),
        ("short", Some(&ADMIN[..39]), &[], expected("forge", Some(SETUP_TOKEN), BAD_TOKEN), vec![]),
        (
            "uppercase",
            Some(b"A123456789abcdef0123456789abcdef01234567".as_slice()),
            &[],
            expected("forge", Some(SETUP_TOKEN), BAD_TOKEN),
            vec![],
        ),
        (
            "long",
            Some(b"0123456789abcdef0123456789abcdef01234567890".as_slice()),
            &[],
            expected("forge", Some(SETUP_TOKEN), "past 40 bytes"),
            vec![],
        ),
        ("both", None, &["--document", "unused.json"], (2, FORGE_USAGE.into()), vec![]),
        (
            "observations",
            Some(ADMIN.as_slice()),
            &["--observations", OBSERVATIONS],
            expected("forge", Some(OBSERVATIONS), EXISTS),
            vec![Effect::Create(OBSERVATIONS.into(), 0o600)],
        ),
    ] {
        let mut entries = vec![("/fixture/playground-1", directory(0o700))];
        entries.extend(token.map(|token| (SETUP_TOKEN, file(token))));
        entries.push((OBSERVATIONS, file(EXISTING)));
        let mut facts = Facts::new(entries);
        let words = ["1"].into_iter().chain(extra.iter().copied()).collect::<Vec<_>>();
        let usage = expected.0 == 2;
        assert_eq!(refusal(&mut facts, "forge", &words, usage), expected, "{name}");
        assert_eq!(facts.effects, effects, "{name}");
        let reads = if usage { vec![] } else { vec![(SETUP_TOKEN.into(), 40, true)] };
        assert_eq!(facts.reads, reads, "{name}");
    }
}
fn refused_demo() {
    let bound = temper_fake::demo::KEY_FILE_BYTES;
    let long_key = vec![b'k'; skein_lib::usize_of(bound) + 1];
    for (name, has_document, key, words, expected) in [
        ("llm_no_document", false, None, &[][..], expected("llm", Some(DEMO_KEY), "absent")),
        ("llm_absent", true, None, &[], expected("llm", Some(DEMO_KEY), "absent")),
        (
            "llm_public",
            true,
            Some(node(Kind::File, 0o644, ADMIN)),
            &[],
            expected("llm", Some(DEMO_KEY), "readable by others or reached through a link"),
        ),
        ("llm_empty", true, Some(file(b"\n")), &[], expected("llm", Some(DEMO_KEY), "holds no key")),
        ("llm_long", true, Some(file(&long_key)), &[], expected("llm", Some(DEMO_KEY), &format!("past {bound} bytes"))),
        ("llm_word", true, None, &["extra"], (2, LLM_USAGE.into())),
    ] {
        let config = has_document.then(|| (CONFIG, file(b"listen = \"127.0.0.1:0\"\n")));
        let mut facts = Facts::new(config.into_iter().chain(key.map(|key| (DEMO_KEY, key))));
        let usage = expected.0 == 2;
        assert_eq!(refusal(&mut facts, "llm", words, usage), expected, "{name}");
        assert!(facts.effects.is_empty(), "{name}");
        let reads = if usage { vec![] } else { vec![(DEMO_KEY.into(), bound, true)] };
        assert_eq!(facts.reads, reads, "{name}: reads no configuration");
    }
}

#[test]
fn refused_start() {
    refused_documents();
    refused_setup();
    refused_demo();
}

#[test]
fn forge_state_refusal_is_one_line_and_leaves_the_file_unchanged() {
    const IMAGE: &[u8] = b"not an image";
    const EXTRA: &str = "/fixture/extra";
    let mut linked = file(IMAGE);
    linked.stat.links = 2;
    for (name, image, alias, opens) in [
        ("bytes", file(IMAGE), None, vec![PathBuf::from(STATE)]),
        ("public", node(Kind::File, 0o644, IMAGE), None, vec![]),
        ("linked", linked.clone(), Some(linked), vec![]),
        ("symlink", node(Kind::Symlink, 0o777, EXTRA.as_bytes()), Some(file(IMAGE)), vec![]),
        ("directory", directory(0o700), None, vec![]),
    ] {
        let mut facts = Facts::new(
            [(DOCUMENT, file(FORGE_DOCUMENT)), (STATE, image)].into_iter().chain(alias.map(|extra| (EXTRA, extra))),
        );
        let (code, stderr) = refusal(&mut facts, "forge", &["--document", DOCUMENT, "--state", STATE], false);
        assert_eq!(code, 1, "{name}");
        let line = stderr.strip_suffix('\n').expect("terminated refusal line");
        assert!(
            !line.contains('\n') && line.starts_with("temper-box fake forge: state /fixture/forge.state refused: "),
            "{name}: {stderr}"
        );
        assert_eq!(facts.opened, opens, "{name}: never opens an unsafe image");
    }
}

#[test]
fn llm_refuses_a_hold_without_control() {
    let mut doc = llm_document(false, false);
    doc["otherwise"] =
        serde_json::json!([{"lines":[{"pieces":[{"text":"held","hold":"gate"}]}],"finish":"stop","tokens":1}]);
    let mut facts = Facts::new([(DOCUMENT, json_node(&doc))]);
    assert_eq!(
        refusal(&mut facts, "llm", &["--document", DOCUMENT], false),
        expected("llm", None, "control: required when a piece has a hold"),
    );
    assert!(facts.effects.is_empty(), "refusal creates no key");
}

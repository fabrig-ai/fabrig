//! The box's group files and steps by kind
//! (temper/docs/design/draft/playground-reset.md, 3; temper/docs/design/protocol/setup.md, 7.3).
use crate::{
    check::joined,
    given, problem,
    setup::{self, Mode},
    step::{self, Group, Kind, Stage, Step},
};
use alloc::boxed::Box;
use jig_protocol_config::{self as config, Path, read as member};
use skein_lib::{Decimal, List, Writer, bytes};

/// The file's bound, the box's read buffer (contract 3).
pub const FILE_BYTES: u32 = 65_536;
/// Most steps a group runs: an owned Forgejo's fetch, prepare, forgejo, runner.
pub const STEPS: u32 = 4;
/// Step names fit 24 bytes; the longest is `fake forge 65535` (16 bytes).
pub const NAME_BYTES: u32 = 24;
/// The longest label is `Forge 65535 (Forgejo)` (21 bytes).
pub const LABEL_BYTES: u32 = 21;
/// The directory beneath the data directory.
pub const DIRECTORY: &[u8] = b"box/groups";
/// One group's file beneath the data directory.
#[must_use]
pub fn path(forge: u16) -> Box<[u8]> {
    joined(&[DIRECTORY, b"/", Decimal::of(u64::from(forge)).as_bytes()])
}

/// One group's file, checked: the forge's setup entry, numbered, its paths resolved.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GroupFile {
    pub entry: setup::Entry,
    pub playground: bool,
}
/// The children's directory flags, path bases, setup limits, fake LLM port,
/// and whether Forgejo inherits its listener or binds it itself.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Machine {
    pub flags: Box<[Box<[u8]>]>,
    pub directories: config::Directories,
    pub limits: skein_forge_setup::limits::Limits,
    pub llm: Option<core::net::SocketAddr>,
    pub forgejo_listens: bool,
}
/// A group and the once step that removes its forge's directory.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Built {
    pub group: Group,
    pub remove: Step,
}

fn refusal(forge: u16, problems: &problem::Problems) -> Box<[u8]> {
    let lines = problem::lines(problems);
    let end = match bytes::find(&lines, b"\n") {
        Some(end) => end,
        None => lines.len(),
    };
    let line = lines.get(..end).expect("first problem line");
    // The kit's entry locations use the document's application.setup[0].
    let line = match line.strip_prefix(b"application.setup[0]") {
        Some(tail) => joined(&[b"setup", tail]),
        None => bytes::copy_of(line),
    };
    if line.starts_with(b": ") { joined(&[&path(forge), &line]) } else { joined(&[&path(forge), b": ", &line]) }
}
fn config_refusal(forge: u16, problems: config::Problems) -> Box<[u8]> {
    let mut ps = problem::Problems::empty();
    ps.append(problems);
    refusal(forge, &ps)
}
fn refused(forge: u16, path: Path, what: config::What) -> Box<[u8]> {
    let mut ps = config::Problems::empty();
    ps.push(path, what);
    config_refusal(forge, ps)
}
fn document(bytes: &[u8], forge: u16) -> Result<skein_json::Document, Box<[u8]>> {
    if bytes.len() > usize::try_from(FILE_BYTES).expect("file bound fits") {
        return Err(refused(forge, Path::root(), config::What::Bound));
    }
    match config::document(bytes, &config::DocumentLimits { bytes: FILE_BYTES, tokens: FILE_BYTES, depth: 32 }) {
        Ok(document) => Ok(document),
        Err(problem) => Err(refused(forge, problem.path, problem.what)),
    }
}

/// `{"setup": S + "forge": N, "playground": P}`, S one JSON object without `forge`.
pub fn compose(setup: &[u8], forge: u16, playground: bool) -> Result<Box<[u8]>, Box<[u8]>> {
    let d = document(setup, forge)?;
    let p = Path::root().member(b"setup");
    let mut object = match member::Object::new(&d, 0) {
        Ok(object) => object,
        Err(what) => return Err(refused(forge, p, what)),
    };
    if object.member(&d, b"forge").is_some() {
        return Err(refused(forge, p.member(b"forge"), config::What::Unknown));
    }
    let opening = bytes::find(setup, b"{").expect("JSON object opening brace");
    let tail = setup.get(opening.checked_add(1).expect("bounded offset")..).expect("object tail");
    let comma = if d.len() > 2 { b",".as_slice() } else { b"".as_slice() };
    let digits = Decimal::of(u64::from(forge));
    let playground = if playground { b"true".as_slice() } else { b"false".as_slice() };
    let parts =
        [b"{\"setup\":{\"forge\":".as_slice(), digits.as_bytes(), comma, tail, b",\"playground\":", playground, b"}"];
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("bounded group file");
    }
    if length > usize::try_from(FILE_BYTES).expect("file bound fits") {
        return Err(refused(forge, Path::root(), config::What::Bound));
    }
    Ok(joined(&parts))
}

/// Checks a file's bytes as forge N's group; a refusal is one line, no LF.
pub fn read(bytes: &[u8], forge: u16, machine: &Machine) -> Result<GroupFile, Box<[u8]>> {
    let d = document(bytes, forge)?;
    let p = Path::root();
    let mut ps = config::Problems::empty();
    let mut object = match member::Object::new(&d, 0) {
        Ok(object) => object,
        Err(what) => return Err(refused(forge, p, what)),
    };
    let setup_at = object.required(&d, b"setup", &p, &mut ps);
    let playground_at = object.required(&d, b"playground", &p, &mut ps);
    object.finish(&d, &p, &mut ps);
    let mut entry = match setup_at {
        Some(at) => setup::entry(&d, at, &machine.limits, &p.member(b"setup"), &mut ps),
        None => None,
    };
    let playground = match playground_at {
        Some(at) => match member::boolean(&d, at) {
            Ok(value) => Some(value),
            Err(what) => {
                ps.push(p.member(b"playground"), what);
                None
            }
        },
        None => None,
    };
    if ps.is_empty()
        && let Some(entry) = &mut entry
    {
        setup::resolve_entry(entry, &machine.directories, &p.member(b"setup"), &mut ps);
        if entry.forge != forge {
            ps.push(p.member(b"setup").member(b"forge"), config::What::Form);
        }
    }
    if !ps.is_empty() {
        return Err(config_refusal(forge, ps));
    }
    let entry = entry.expect("clean entry parse");
    if let Err(why) = skein_forge_setup::given::check(&given::values(&entry, None, &machine.limits)) {
        return Err(refusal(forge, &given::refusal(0, why)));
    }
    Ok(GroupFile { entry, playground: playground.expect("clean playground parse") })
}

/// The group's steps by its kind; a managed forge has none.
pub fn build(file: &GroupFile, machine: &Machine) -> Result<Built, Box<[u8]>> {
    let entry = &file.entry;
    let flags = &machine.flags;
    let digits = Decimal::of(u64::from(entry.forge));
    let number = digits.as_bytes();
    let mut steps = List::with_capacity(STEPS);
    let (label, recorded) = match &entry.mode {
        Mode::Managed { .. } => {
            return Err(joined(&[b"forge ", number, b" is managed: the box runs no group for it"]));
        }
        Mode::Owned(owned) => {
            if owned.fetch.forgejo || owned.fetch.runner {
                steps.push((Stage::Fetch, step::forge(b"fetch", flags, entry.forge, None))).expect("fetch");
            }
            steps.push((Stage::Prepare, step::forge(b"prepare", flags, entry.forge, None))).expect("prepare");
            let label = joined(&[b"Forge ", number, b" (Forgejo)"]);
            let mut forgejo = step::forge(b"run", flags, entry.forge, Some(b"forgejo")).with_ready(owned.listen);
            if machine.forgejo_listens {
                forgejo = forgejo.listening(Box::new([owned.listen]));
            }
            forgejo.set_started_notice(joined(&[
                &label,
                b": started\n",
                &label,
                b": api http://",
                &notice_address(owned.listen),
                b"/api/v1\n",
                &label,
                b": person ",
                &owned.person.login,
                b", read header ",
                &entry.directory,
                b"/person_read.header, write header ",
                &entry.directory,
                b"/person_write.header\n",
            ]));
            steps.push((Stage::Run, forgejo)).expect("forgejo");
            if owned.runner.is_some() {
                steps.push((Stage::Run, step::forge(b"run", flags, entry.forge, Some(b"runner")))).expect("runner");
            }
            (label, true)
        }
        Mode::Fake(fake) => {
            if file.playground {
                let Some(llm) = machine.llm else {
                    return Err(joined(&[b"forge ", number, b" is the playground: the machine has no fake LLM port"]));
                };
                steps
                    .push((
                        Stage::Run,
                        command(b"fake llm", &[b"fake", b"llm"], flags, None, Kind::Long).listening(Box::new([llm])),
                    ))
                    .expect("fake LLM");
            }
            steps
                .push((
                    Stage::Prepare,
                    command(b"prepare", &[b"forge", b"prepare"], flags, Some(entry.forge), Kind::Once),
                ))
                .expect("fake preparation");
            steps
                .push((
                    Stage::Run,
                    command(b"fake forge", &[b"fake", b"forge"], flags, Some(entry.forge), Kind::Long)
                        .listening(Box::new([fake.listen])),
                ))
                .expect("fake forge");
            if file.playground {
                (bytes::copy_of(b"Playground"), false)
            } else {
                (joined(&[b"Forge ", number, b" (fake)"]), true)
            }
        }
    };
    Ok(Built {
        group: Group { forge: entry.forge, label, recorded, before: 0, steps: steps.into_boxed() },
        remove: step::forge(b"remove", flags, entry.forge, None),
    })
}

fn command(name: &[u8], words: &[&[u8]], flags: &[Box<[u8]>], forge: Option<u16>, kind: Kind) -> Step {
    let capacity = u32::try_from(words.len())
        .expect("command words")
        .checked_add(u32::try_from(flags.len()).expect("bounded flags"))
        .expect("bounded arguments")
        .checked_add(u32::from(forge.is_some()))
        .expect("forge number");
    let mut arguments = List::with_capacity(capacity);
    for word in words {
        arguments.push(bytes::copy_of(word)).expect("command words counted");
    }
    for flag in flags {
        arguments.push(flag.clone()).expect("flags counted");
    }
    let name = match forge {
        Some(number) => {
            let digits = Decimal::of(u64::from(number));
            let number = digits.as_bytes();
            arguments.push(bytes::copy_of(number)).expect("forge number counted");
            let length =
                name.len().checked_add(1).expect("name and space").checked_add(number.len()).expect("bounded name");
            let mut writer = Writer::new(length);
            writer.put(name).expect("name counted");
            writer.put(b" ").expect("space counted");
            writer.put(number).expect("number counted");
            writer.finish()
        }
        None => bytes::copy_of(name),
    };
    Step::new(name, arguments.into_boxed(), kind)
}

// SocketAddr's public spelling: compressed IPv6, with its scope when present.
fn notice_address(address: core::net::SocketAddr) -> Box<[u8]> {
    let core::net::SocketAddr::V6(address) = address else {
        return crate::ini::socket(address);
    };
    let ip = address.ip();
    let mut parts = List::with_capacity(22);
    parts.push(bytes::copy_of(b"[")).expect("address pieces");
    if let Some(mapped) = ip.to_ipv4_mapped() {
        parts.push(bytes::copy_of(b"::ffff:")).expect("mapped prefix");
        parts.push(crate::ini::ip(core::net::IpAddr::V4(mapped))).expect("mapped address");
    } else {
        let segments = ip.segments();
        let mut start = 0_usize;
        let mut length = 0_usize;
        let mut longest_start = 0_usize;
        let mut longest = 1_usize;
        for (index, segment) in segments.iter().enumerate() {
            if *segment == 0 {
                if length == 0 {
                    start = index;
                }
                length = length.checked_add(1).expect("eight segments");
                if length > longest {
                    longest_start = start;
                    longest = length;
                }
            } else {
                length = 0;
            }
        }
        let end = longest_start.checked_add(longest).expect("eight segments");
        for (index, segment) in segments.iter().enumerate() {
            if longest > 1 && index >= longest_start && index < end {
                if index == longest_start {
                    parts.push(bytes::copy_of(b"::")).expect("compression");
                }
                continue;
            }
            if index > 0 && !(longest > 1 && index == end) {
                parts.push(bytes::copy_of(b":")).expect("separator");
            }
            let mut digits = List::with_capacity(4);
            for shift in [12_u32, 8, 4, 0] {
                let digit = u8::try_from((segment >> shift) & 15).expect("hex digit");
                if digit != 0 || !digits.is_empty() || shift == 0 {
                    let byte = if digit < 10 {
                        b'0'.checked_add(digit)
                    } else {
                        b'a'.checked_add(digit.checked_sub(10).expect("hex letter"))
                    }
                    .expect("ASCII hex digit");
                    digits.push(byte).expect("four hex digits");
                }
            }
            parts.push(digits.into_boxed()).expect("segment");
        }
    }
    if address.scope_id() != 0 {
        parts.push(bytes::copy_of(b"%")).expect("scope prefix");
        parts.push(bytes::copy_of(Decimal::of(u64::from(address.scope_id())).as_bytes())).expect("scope");
    }
    parts.push(bytes::copy_of(b"]:")).expect("port prefix");
    parts.push(bytes::copy_of(Decimal::of(u64::from(address.port())).as_bytes())).expect("port");
    crate::ini::flatten(parts.as_slice())
}

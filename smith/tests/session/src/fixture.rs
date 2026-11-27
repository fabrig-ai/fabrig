//! The tree each session's tools work on, seeded in the fake checkout, and
//! how the referee joins a returned result to its call. Menu writes and
//! edits preserve the path marker on every file line; a result in the wrong
//! batch slot is caught by [`fits`].

use std::time::Duration;

use skein_fake_checkout::{self as fake, Checkout, Program};
use smith_domain_tools::{Call, Exit, Op, Outcome, Part, Path, Resolved, Yield};

/// The files each session's repository starts with: their content starts
/// with their path.
const FILES: [(&[u8], &[u8]); 4] = [
    (b"README.md", b"README.md: hello, world\n"),
    (b"src/lib.rs", b"src/lib.rs: pub fn answer() -> u32 { 42 }\nsrc/lib.rs: source\n"),
    (b"docs/guide.md", b"docs/guide.md: the guide\ndocs/guide.md: example\n"),
    (b"docs/other.md", b"docs/other.md: another example\n"),
];

/// The file on the menu that is missing until a write makes it.
const MISSING: &[u8] = b"notes.md";

/// The commands on the menu, and how each ends. What each writes starts with
/// the command. (The world draws how long a command runs, as it does for every
/// operation.)
const COMMANDS: [(&[u8], u8); 6] =
    [(b"cargo test", 0), (b"ls", 0), (b"true", 0), (b"cat README.md", 0), (b"false", 1), (b"rewrite README.md", 0)];

/// Scripts the commands on the menu, once for the whole checkout.
pub fn script(checkout: &mut Checkout) {
    for (command, code) in COMMANDS {
        let output = output(command);
        let changes = if command == b"rewrite README.md" {
            vec![(b"README.md".to_vec(), Some(b"README.md: generated\n".to_vec()))]
        } else {
            Vec::new()
        };
        let program = Program { duration: Duration::from_millis(50), output, exit: fake::Exit::Code(code), changes };
        checkout.program(command, program);
    }
}

/// What `command` writes.
fn output(command: &[u8]) -> Vec<u8> {
    [command, b": done\n"].concat()
}

/// Seeds a repository at `at` in the checkout, and makes it a root: io's name
/// for it.
pub fn seed(checkout: &mut Checkout, at: &[u8]) -> u64 {
    checkout.mkdir(at);
    for (path, content) in FILES {
        checkout.write(&[at, b"/", path].concat(), content);
    }
    checkout.root(at)
}

pub(super) fn seed_reference(checkout: &mut Checkout, at: &[u8]) -> u64 {
    checkout.mkdir(at);
    checkout.write(&[at, b"/README.md"].concat(), b"README.md: reference\n");
    checkout.root(at)
}

/// Register the fixture's selected files immediately before listing so a
/// removed path cannot be rendered as present. The menu uses no filter or `*`.
pub(super) fn prepare_list(checkout: &mut Checkout, op: &Op) {
    let Op::List { at, glob, .. } = op else { return };
    if !matches!(glob.as_deref(), None | Some(b"*")) {
        return;
    }
    let root = checkout.root_path(at.root.raw());
    let mut selected = FILES
        .iter()
        .map(|&(file, _)| file)
        .chain([MISSING])
        .filter_map(|file| {
            let relative = if at.path.is_empty() { file } else { beneath(file, &at.path)? };
            checkout.exists(&[root, b"/", file].concat()).then_some(relative)
        })
        .collect::<Vec<_>>();
    selected.sort_unstable();
    smith_tools_world::translate::listing_fixture(checkout, at.root, &at.path, glob.as_deref(), &selected);
}

/// Whether `text` holds `pattern` whole.
fn holds(text: &[u8], pattern: &[u8]) -> bool {
    skein_lib::bytes::find(text, pattern).is_some()
}

/// Where `path` is, relative to the working directory at the repository's
/// root, or `None` if it climbs out of it.
fn place(path: &Path) -> Option<Vec<u8>> {
    let mut names: Vec<&[u8]> = Vec::new();
    for part in &path.parts {
        match part {
            Part::Current => {}
            Part::Parent => {
                names.pop()?;
            }
            Part::Name { name } => names.push(name.as_bytes()),
        }
    }
    Some(names.join(&b'/'))
}

/// Whether `outcome` may come of `call` on a session's tree, as it is or as
/// the menu's writes and edits leave it. `created_at_store` is the checkout's
/// evidence for this call and path, taken immediately before a successful store.
/// Failures any call may meet (a fault, a deadline, a cancel, a family not
/// granted, a full kit) fit every call.
#[must_use]
pub fn fits(call: &Call, outcome: &Outcome, created_at_store: Option<bool>) -> bool {
    if matches!(
        outcome,
        Outcome::Failed { .. } | Outcome::TimedOut | Outcome::Cancelled | Outcome::Busy | Outcome::NotGranted
    ) {
        return true;
    }
    let path = match call {
        Call::Read { path, .. }
        | Call::List { path, .. }
        | Call::Search { path, .. }
        | Call::Remove { path }
        | Call::Write { path, .. }
        | Call::Edit { path, .. } => path,
        Call::Job { job, .. } => {
            return match outcome {
                Outcome::Job { job: answered, .. } => job == answered,
                Outcome::Interrupted { .. } => true,
                _ => false,
            };
        }
        Call::Shell { command, after, .. } => {
            return match outcome {
                Outcome::Running { head, tail, dropped, .. } => {
                    matches!(after, Yield::Asked(_)) && head.is_empty() && tail.is_empty() && *dropped == 0
                }
                Outcome::Exited { exit: Exit::TimedOut, head, .. } | Outcome::Interrupted { head, .. } => {
                    output(command).starts_with(head)
                }
                Outcome::Exited { head, tail, .. } => {
                    let expected = output(command);
                    expected.starts_with(head) && expected.ends_with(tail)
                }
                Outcome::NulByte => command.contains(&0),
                _ => false,
            };
        }
    };
    let Some(place) = place(path) else {
        return matches!(outcome, Outcome::Outside { .. });
    };
    let reference = path.absolute.then(|| beneath(&place, b"ref")).flatten();
    fits_path(reference.unwrap_or(&place), reference.is_some(), call, outcome, created_at_store)
}

fn fits_path(place: &[u8], reference: bool, call: &Call, outcome: &Outcome, created_at_store: Option<bool>) -> bool {
    if reference {
        match outcome {
            Outcome::ReadOnly { path: Resolved::Other { directory, path } } => {
                return directory.as_bytes() == b"ref"
                    && path.as_ref() == place
                    && matches!(call, Call::Write { .. } | Call::Edit { .. } | Call::Remove { .. });
            }
            Outcome::Outside { path: Resolved::Outside { path } } => return beneath(path, b"ref") == Some(place),
            Outcome::NotFound { path: Resolved::Other { directory, path } } => {
                return directory.as_bytes() == b"ref" && path.as_ref() == place;
            }
            _ => {}
        }
    }
    let file = |path: &[u8]| path == MISSING || FILES.iter().any(|(file, _)| *file == path);
    let directory = |path: &[u8]| path.is_empty() || FILES.iter().any(|(file, _)| beneath(file, path).is_some());
    let in_directory = |entry: &[u8]| {
        if place.is_empty() { entry.to_vec() } else { [place, b"/", entry].concat() }
    };
    let marked = |content: &[u8], path: &[u8]| {
        let prefix = [path, b":"].concat();
        content.starts_with(&prefix) || prefix.starts_with(content)
    };
    match (call, outcome) {
        (Call::Read { offset, .. }, Outcome::Read { first, total, content, .. }) => {
            file(place) && *first == (*offset).min(total.saturating_add(1)) && marked(content, place)
        }
        (Call::List { .. }, Outcome::Listed { entries, .. }) => {
            directory(place)
                && entries.iter().all(|entry| {
                    let path = in_directory(&entry.name);
                    match entry.kind {
                        smith_domain_tools::Kind::File => file(&path),
                        smith_domain_tools::Kind::Directory => directory(&path),
                        smith_domain_tools::Kind::Link | smith_domain_tools::Kind::Other => false,
                    }
                })
        }
        // A share below the line may cut the last hit before its pattern.
        (Call::Search { pattern, .. }, Outcome::Found { hits, .. }) => hits.iter().enumerate().all(|(index, hit)| {
            let path = if hit.path.is_empty() { place.to_vec() } else { in_directory(&hit.path) };
            file(&path)
                && marked(&hit.text, &path)
                && (holds(&hit.text, pattern)
                    || (index + 1 == hits.len()
                        && FILES
                            .iter()
                            .filter(|(file, _)| *file == &*path)
                            .flat_map(|(_, content)| content.split(|byte| *byte == b'\n'))
                            .any(|line| line.starts_with(&hit.text) && holds(line, pattern))))
        }),
        (Call::Remove { .. }, Outcome::Removed) => file(place) || directory(place),
        (Call::Write { .. }, Outcome::Written { created }) => {
            !reference && file(place) && created_at_store == Some(*created)
        }
        (Call::Edit { .. }, Outcome::Edited { .. } | Outcome::EditFailed { .. })
        | (Call::List { .. }, Outcome::NotDirectory { .. }) => file(place),
        (Call::Remove { .. }, Outcome::NotEmpty)
        | (Call::Read { .. } | Call::Write { .. } | Call::Edit { .. }, Outcome::NotFile) => directory(place),
        (_, Outcome::NotFound { path: Resolved::Working { path } }) => path.as_ref() == place,

        (Call::Search { pattern, glob, .. }, Outcome::NulByte) => {
            pattern.contains(&0) || glob.as_ref().is_some_and(|glob| glob.contains(&0))
        }
        (Call::Remove { .. } | Call::Write { .. } | Call::Edit { .. }, Outcome::Protected) => {
            place.is_empty() || place.split(|&byte| byte == b'/').any(|part| part.eq_ignore_ascii_case(b".git"))
        }
        (
            Call::Write { content, .. },
            Outcome::TooLarge { what: smith_domain_tools::SizePart::WriteContent, size, bound },
        ) => *size == skein_lib::len_u64(content.len()) && *size > u64::from(*bound),
        (Call::Remove { .. } | Call::Write { .. } | Call::Edit { .. }, Outcome::NotRead | Outcome::Stale)
        | (_, Outcome::TooLong) => true,
        _ => false,
    }
}

fn beneath<'a>(path: &'a [u8], directory: &[u8]) -> Option<&'a [u8]> {
    path.strip_prefix(directory)?.strip_prefix(b"/")
}

/// Owned bytes supplied by a scripted neighbour.
#[must_use]
pub fn bytes(text: &[u8]) -> Box<[u8]> {
    text.into()
}

/// One ordinary provider text block, without an opaque replay envelope.
#[must_use]
pub fn said(text: &[u8]) -> Box<[smith_domain_session::llm::Block]> {
    Box::new([smith_domain_session::llm::Block::Text { text: bytes(text), replay: None }])
}

/// A scripted provider turn offering its calls with the supplied token charge.
#[must_use]
pub fn calls(lines: Box<[skein_fake_llm_domain::api::Line]>, tokens: u64) -> skein_fake_llm_domain::api::Turn {
    skein_fake_llm_domain::api::Turn { lines, finish: skein_fake_llm_domain::api::Finish::ToolCalls, tokens }
}

/// A scripted provider turn stopping after its supplied lines and token charge.
#[must_use]
pub fn says(lines: Box<[skein_fake_llm_domain::api::Line]>, tokens: u64) -> skein_fake_llm_domain::api::Turn {
    skein_fake_llm_domain::api::Turn { lines, finish: skein_fake_llm_domain::api::Finish::Stop, tokens }
}

//! Observed rg selection and the public machine entry points for tree listings.

use skein_io::kernel;
use skein_lib::{Env, Queue, Time, Token, Wall};
use smith_domain::{run, tools};
use smith_protocol_machine::{Below, BelowEvent, Component, FromDomain, Limits, ToDomain};

pub(super) fn limits() -> Limits {
    Limits { entries: 8, ..smith_protocol_machine::fixture::SMALL }
}

fn env() -> Env<Limits> {
    Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() }
}

fn component() -> Component {
    configured_component(&[])
}

fn configured_component(variables: &[smith_protocol_machine::Variable]) -> Component {
    let mut component = Component::new(
        &limits(),
        smith_protocol_machine::Environment::encode(variables, &limits()).expect("configured environment fits"),
    );
    component.workspace(&run::Workspace {
        directories: Box::new([run::Directory {
            host: Box::from(b"/host".as_slice()),
            name: Box::from(&b"code"[..]),
            root: Token::new(10),
            writable: true,
            git: true,
            conflicts: Box::new([]),
        }]),
    });
    component
}

fn place(path: &[u8]) -> tools::Place {
    tools::Place { root: Token::new(10), path: Box::from(path) }
}
#[expect(clippy::too_many_lines, reason = "one listing follows directory open, close and bounded output")]
fn listed(output: &[u8], depth: tools::Depth, maximum: u32, bytes: u64, glob: Option<&[u8]>) -> tools::Done {
    let mut component = component();
    let mut terminals = Queue::with_capacity(4);
    let mut below = Queue::with_capacity(4);
    let owner = Token::new(50);
    component.from_domain(
        &env(),
        FromDomain::Op {
            owner,
            op: tools::Op::List { at: place(b"src"), depth, glob: glob.map(Box::from), max: maximum, max_bytes: bytes },
            deadline: Time::from_nanos(u64::MAX),
        },
        &mut terminals,
        &mut below,
    );
    assert_eq!(
        below.pop(),
        Some(Below::OpenDirectory {
            owner,
            root: Token::new(10),
            path: Box::from(b"src".as_slice()),
            no_follow: false,
            deadline: Time::from_nanos(u64::MAX)
        })
    );
    component.from_below(
        &env(),
        BelowEvent::File(skein_io::file::Event::Opened { owner, file: Token::new(49), len: 0 }),
        &mut terminals,
        &mut below,
    );
    assert_eq!(
        below.pop(),
        Some(Below::File {
            request: skein_io::file::Request::Close { owner, file: Token::new(49) },
            deadline: Time::from_nanos(u64::MAX)
        })
    );
    assert!(terminals.is_empty(), "directory closure is owed before spawning");
    component.from_below(&env(), BelowEvent::File(skein_io::file::Event::Closed { owner }), &mut terminals, &mut below);
    match below.pop().expect("listing starts rg") {
        Below::Spawn { spawn, root, .. } => {
            assert_eq!(root, Token::new(10));
            assert_eq!(spawn.dir.as_ref(), b"src");
            assert_eq!(spawn.program.as_ref(), b"/usr/bin/rg");
            assert!(spawn.env.is_empty());
            assert!(spawn.args.iter().any(|word| word.as_ref() == b"--files"));
            assert!(spawn.args.iter().any(|word| word.as_ref() == b"--null"));
            assert!(spawn.args.iter().any(|word| word.as_ref() == b"--sort=path"));
            if let Some(glob) = glob {
                assert!(spawn.args.iter().any(|word| word.strip_prefix(b"--glob=") == Some(glob)));
            }
        }
        Below::File { .. }
        | Below::Remove { .. }
        | Below::CancelFile { .. }
        | Below::OpenRead { .. }
        | Below::OpenDirectory { .. }
        | Below::Process(_) => {
            panic!("listing is a process")
        }
    }
    let child = Token::new(51);
    let stdout = Token::new(52);
    let stderr = Token::new(53);
    component.from_below(
        &env(),
        BelowEvent::Process(skein_io::Event::Spawned { owner, child, pipes: Box::new([stdout, stderr]) }),
        &mut terminals,
        &mut below,
    );
    while below.pop().is_some() {}
    for byte in output {
        let chunk = Box::new([*byte]);
        component.from_below(
            &env(),
            BelowEvent::Pipe(skein_io::Event::Stream { owner: stdout, up: skein_lib::stream::Up::Bytes(chunk) }),
            &mut terminals,
            &mut below,
        );
        while below.pop().is_some() {}
    }
    component.from_below(
        &env(),
        BelowEvent::Process(skein_io::Event::Exited { owner, exit: kernel::Exit::Code(0) }),
        &mut terminals,
        &mut below,
    );
    while below.pop().is_some() {}
    for event in [
        BelowEvent::Process(skein_io::Event::Closed { owner }),
        BelowEvent::Pipe(skein_io::Event::Closed { owner: stdout }),
        BelowEvent::Pipe(skein_io::Event::Closed { owner: stderr }),
    ] {
        component.from_below(&env(), event, &mut terminals, &mut below);
    }
    match terminals.pop().expect("one listing terminal") {
        ToDomain::Done { done, .. } => done,
        ToDomain::Read { .. } | ToDomain::Probed { .. } | ToDomain::Checked { .. } | ToDomain::Aborted { .. } => {
            panic!("listing terminal")
        }
    }
}

fn listed_entry(path: &[u8], kind: tools::Kind) -> tools::Entry {
    tools::Entry { name: Box::from(path), kind }
}

fn selected_native_output(suffix: &str) -> Vec<u8> {
    use std::fs;
    let root = std::env::temp_dir().join(format!("smith-list-tree-{suffix}-{}", std::process::id()));
    fs::create_dir(&root).expect("unique scratch");
    for directory in ["src/deep", "ignored", "empty", ".git", "unmatched"] {
        fs::create_dir_all(root.join(directory)).expect("fixture directory");
    }
    fs::write(root.join(".ignore"), b"ignored/\n").expect("ignore rule");
    for file in
        ["src/empty.rs", "src/deep/only.rs", "src/not.txt", "ignored/drop.rs", ".git/config", "unmatched/only.txt"]
    {
        fs::write(root.join(file), b"").expect("empty selected file fixture");
    }
    let output = native_output(&root);
    fs::remove_dir_all(&root).expect("scratch reclaimed");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.stdout.as_slice(), b"./src/deep/only.rs\0./src/empty.rs\0");
    output.stdout
}

fn native_output(root: &std::path::Path) -> std::process::Output {
    let mut component = component();
    let mut up = Queue::with_capacity(2);
    let mut down = Queue::with_capacity(2);
    component.from_domain(
        &env(),
        FromDomain::Op {
            owner: Token::new(90),
            op: tools::Op::List {
                at: place(b""),
                depth: tools::Depth::Levels(1),
                glob: Some(Box::from(b"*.rs".as_slice())),
                max: 2,
                max_bytes: 512,
            },
            deadline: Time::from_nanos(u64::MAX),
        },
        &mut up,
        &mut down,
    );
    assert!(matches!(down.pop(), Some(Below::OpenDirectory { .. })));
    component.from_below(
        &env(),
        BelowEvent::File(skein_io::file::Event::Opened { owner: Token::new(90), file: Token::new(91), len: 0 }),
        &mut up,
        &mut down,
    );
    assert!(matches!(down.pop(), Some(Below::File { request: skein_io::file::Request::Close { .. }, .. })));
    component.from_below(
        &env(),
        BelowEvent::File(skein_io::file::Event::Closed { owner: Token::new(90) }),
        &mut up,
        &mut down,
    );
    let spawn = match down.pop().expect("native listing") {
        Below::Spawn { spawn, .. } => spawn,
        Below::File { .. }
        | Below::Remove { .. }
        | Below::CancelFile { .. }
        | Below::OpenRead { .. }
        | Below::OpenDirectory { .. }
        | Below::Process(_) => {
            panic!("list spawns rg")
        }
    };
    let mut command = std::process::Command::new(std::str::from_utf8(&spawn.program).expect("fixed program"));
    command.current_dir(root).env_clear();
    for word in &spawn.args {
        command.arg(std::str::from_utf8(word).expect("fixture arguments"));
    }
    command.output().expect("native rg fixture")
}

#[test]
fn native_rg_selects_empty_files_and_ignores_empty_unmatched_and_ignored_directories() {
    const DIRECT_CHILDREN: u32 = 1;
    const ENTRY_CAP: u32 = 2;
    const ENTRY_BYTES: u64 = 512;
    let output = selected_native_output("selection");
    assert_eq!(
        listed(&output, tools::Depth::Levels(DIRECT_CHILDREN), ENTRY_CAP, ENTRY_BYTES, Some(b"*.rs")),
        tools::Done::Listed { entries: Box::new([listed_entry(b"src", tools::Kind::Directory)]), more: 0 }
    );
}

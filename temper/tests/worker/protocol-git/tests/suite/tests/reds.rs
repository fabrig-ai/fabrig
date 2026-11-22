//! Check markers and bounded results (temper/docs/design/domain/forge/reds.md, 5).
use skein_io::kernel::Error;
use skein_lib::Token;
use std::collections::BTreeMap;
use temper_worker_domain_checkout::git;
use temper_worker_protocol_git::{
    invocation::{self, Command},
    reds::CheckCommand,
};
use temper_worker_protocol_git_world::{
    fixtures,
    io::{Fault, Invocation, Node, Script},
    referee::Expected,
    transcripts::Transcript,
    world::{Scenario, Stop, World},
};

struct Check {
    at: git::Place,
    directory: Vec<u8>,
    world: World,
}

impl Check {
    fn new() -> Self {
        let at = git::Place { workspace: Token::new(7), repository: b"app".as_slice().into(), git: None };
        let checkout = invocation::place(&at);
        let metadata = [checkout.as_ref(), b"/.git"].concat();
        let directory = [metadata.as_slice(), b"/temper"].concat();
        let tree = BTreeMap::from([
            (checkout.into_vec(), Node::Directory),
            (metadata, Node::Directory),
            (directory.clone(), Node::Directory),
            (invocation::git_dir(&at).into_vec(), Node::Directory),
        ]);
        Self { at, directory, world: World::new(fixtures::SEED, tree) }
    }

    fn path(&self, name: &[u8]) -> Vec<u8> {
        [self.directory.as_slice(), b"/", name].concat()
    }

    fn put(&mut self, name: &[u8], bytes: impl Into<Box<[u8]>>) {
        self.world.below.tree.insert(self.path(name), Node::File(bytes.into()));
    }

    fn bytes(&self, name: &[u8]) -> Option<&[u8]> {
        match self.world.below.tree.get(&self.path(name)) {
            Some(Node::File(bytes)) => Some(bytes),
            Some(Node::Directory | Node::Link) => panic!("check artifact must be a file"),
            None => None,
        }
    }

    fn run(&mut self, action: git::Reds, fail_read: bool, expected: git::Done) {
        let clear = matches!(action, git::Reds::Clear);
        let op = git::Op::Reds { at: self.at.clone(), action };
        let mut script = Script::default();
        if !clear {
            let mut argv = b"git\n".to_vec();
            for arg in invocation::arguments_for(&op, &Command::Reds(CheckCommand::Pin)) {
                argv.extend_from_slice(&arg);
                argv.push(b'\n');
            }
            script.invocations.push_back(Invocation {
                transcript: Transcript {
                    argv: argv.into_boxed_slice(),
                    exit: 0,
                    stdout: [invocation::hex(fixtures::ONE).as_ref(), b"\n"].concat().into_boxed_slice(),
                    stderr: Box::new([]),
                },
                fault: Fault::None,
            });
        }
        if fail_read {
            // The result reader opens the directory, removes reds, and opens check-results before reading.
            script.file_failure = Some((4, Error::Permission));
        }
        self.world
            .run(&Scenario {
                op,
                script,
                stop: Stop::None,
                expected: Expected::done(expected),
                identity: false,
                files: BTreeMap::new(),
            })
            .expect("check file steps settle");
    }
}

#[test]
fn prepare_and_start_clear_results_and_install_the_rerun_marker() {
    let mut check = Check::new();
    for (processes, expected) in
        [(4, b"temper-rerun 1 4\n".as_slice()), (0, b"temper-rerun 1 0\n"), (16, b"temper-rerun 1 16\n")]
    {
        check.put(b"reds", b"stale".as_slice());
        check.put(b"check-results", b"stale".as_slice());
        check.put(b"rerun.tmp", b"unfinished".as_slice());
        check.run(git::Reds::Prepare { wait_ms: 6000, rerun_processes: processes }, false, git::Done::Succeeded);
        assert_eq!(check.bytes(b"rerun"), Some(expected));
        assert!(check.bytes(b"rerun.tmp").is_none());
        assert!(check.bytes(b"reds").is_none());
        assert!(check.bytes(b"check-results").is_none());
        check.put(b"check-results", b"another check".as_slice());
        check.run(git::Reds::Clear, false, git::Done::Succeeded);
        assert!(check.bytes(b"check-results").is_none());
        assert_eq!(check.bytes(b"rerun"), Some(expected));
    }
    check.world.shutdown();
}

#[test]
fn results_are_read_to_the_bound_then_removed_even_on_read_failure() {
    let mut check = Check::new();
    for (size, fail) in [
        (None, false),
        (Some(0), false),
        (Some(31), false),
        (Some(262_144), false),
        (Some(262_145), false),
        (Some(300_000), false),
        (Some(31), true),
    ] {
        check.put(b"reds", b"stale".as_slice());
        if let Some(size) = size {
            check.put(b"check-results", vec![b'x'; size].into_boxed_slice());
        }
        let expected = size.filter(|_| !fail).map(|size| vec![b'x'; size.min(262_145)].into_boxed_slice());
        check.run(git::Reds::Read { check: 1 }, fail, git::Done::Results { bytes: expected });
        assert!(check.bytes(b"check-results").is_none());
        assert!(check.bytes(b"reds").is_none());
        check.run(git::Reds::Read { check: 1 }, false, git::Done::Results { bytes: None });
    }
    check.world.shutdown();
}

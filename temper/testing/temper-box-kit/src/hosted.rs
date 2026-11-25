//! One real loop for serve and its serial ctl one-shots (temper/docs/design/client/web/testing.md, 3).
use crate::{
    Ended,
    boxed::BOX_DEADLINE,
    ctl::{Calls, Step},
};
use skein_io::kernel::{Exit, ServiceSignal, Spawn};
use skein_lib::Time;
use skein_shell::{Clock, Delegate, Host};
use skein_world::{Controls, HostedProgram, Inherited, Referee, StartupRoot, real::World};
use std::{
    cell::RefCell,
    ffi::{OsStr, OsString},
    io,
    os::unix::ffi::OsStrExt,
    path::Path,
    rc::Rc,
};
use temper_box_shell::{Service, testing::serve};
use temper_client_shell_cli::{
    arguments,
    host::{Invocation, OneShot, Startup},
};

pub(super) const CTL: &[u8] = b"/hosted/temper-ctl";

/// The result after serve and every admitted call have settled.
pub struct Sitting {
    pub calls: Vec<Ended>,
    pub left: usize,
    pub exit: Exit,
    pub ending: String,
    pub failure: Option<String>,
}

/// Start serve in the scratch deployment and run `Run` and `Then` ctl steps in order.
/// The client directory is `client` under `client_root`. A refused call leaves the
/// remaining steps unrun, but serve still receives its first termination signal.
pub fn run_ctl(
    directory: &Path,
    flags: Vec<OsString>,
    client_root: &Path,
    steps: Vec<Step>,
) -> Result<Sitting, String> {
    let working = directory.as_os_str().as_bytes();
    let flags: Vec<Box<[u8]>> = flags.into_iter().map(|flag| flag.into_encoded_bytes().into_boxed_slice()).collect();
    let (directories, rest) =
        temper_box_shell::directories::resolve(&flags, &[], working).map_err(|why| why.to_string())?;
    assert!(rest.is_empty(), "settings sitting takes only directory flags");
    let clock = Clock::new();
    let mut world = World::new_controlled(|controls| Judge { controls, stopping: false });
    let output = LinkOutput::default();
    let root = skein_shell::open_root(client_root).map_err(|errno| format!("ctl client root: errno {errno}"))?;
    let (mut server, fds) = match serve::hosted(&[], working, directories, output.clone()) {
        Ok(prepared) => prepared,
        Err(why) => {
            skein_shell::keeper::close_keeper_fd(root);
            return Err(why);
        }
    };
    let count = steps.len();
    assert_eq!(
        world.spawn_signals_with_fds(fds, |signals| {
            (serve::SERVED.adopt_signals)(&mut server.service, signals).expect("serve accepts its signal pipe");
            Proc::Serve(Box::new(server))
        }),
        0
    );
    world.spawn_with_fds(vec![root], || Proc::Calls(Box::new(Calls::hosted(root, client_root, steps, output))));
    let limits = temper_client_shell_cli::limits::native();
    world.host_roots(
        HostedProgram {
            program: CTL.into(),
            make: make_ctl,
            instances: 1,
            operations: skein_io::operations(&limits.io).expect("ctl operations") * 2 + 5,
        },
        ctl_roots,
    );
    let mut ended = world.try_run(&clock, BOX_DEADLINE)?.procs.into_iter();
    let (Some(Proc::Serve(server)), Some(Proc::Calls(calls))) = (ended.next(), ended.next()) else {
        unreachable!("settled sitting's two roots");
    };
    let (ending, code) = temper_service_shell::testing::hosted_ending(&server);
    let calls = calls.into_ends();
    let left = count - calls.len();
    let failure = refused(&calls)
        .or_else(|| (code != 0).then(|| format!("serve exit {code}: {}", crate::without_secrets(ending.as_bytes()))));
    Ok(Sitting { calls, left, exit: Exit::Code(code), ending, failure })
}

fn refused(calls: &[Ended]) -> Option<String> {
    calls.last().filter(|call| call.exit != Some(Exit::Code(0))).map(|call| {
        format!("ctl call {} exited {:?}: {}", calls.len(), call.exit, crate::without_secrets(&call.stderr))
    })
}

#[derive(Clone, Default)]
pub(super) struct LinkOutput(Rc<RefCell<Vec<u8>>>);
impl LinkOutput {
    pub(super) fn take_link(&self) -> Vec<u8> {
        let mut bytes = self.0.borrow_mut();
        if crate::printed_link(&bytes).is_some() { std::mem::take(&mut *bytes) } else { Vec::new() }
    }
}
impl io::Write for LinkOutput {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.borrow_mut().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn ctl_directory(spawn: &Spawn) -> &Path {
    assert_eq!(spawn.args[0].as_ref(), b"ctl");
    let path = Path::new(OsStr::from_bytes(&spawn.args[1]));
    assert!(path.is_absolute(), "absolute hosted client directory");
    path
}
fn ctl_roots(spawn: &Spawn) -> Vec<StartupRoot> {
    let directory = ctl_directory(spawn);
    let words: Vec<&[u8]> = spawn.args[2..].iter().map(AsRef::as_ref).collect();
    let parsed = arguments::parse(&words).expect("settings ctl arguments");
    let mut roots = vec![StartupRoot {
        name: b"parent".as_slice().into(),
        path: directory.parent().expect("client parent").as_os_str().as_bytes().into(),
    }];
    if let Some(path) = arguments::file_root(&parsed) {
        roots.push(StartupRoot { name: b"file".as_slice().into(), path });
    }
    for (number, path) in arguments::text_files(&parsed) {
        roots.push(StartupRoot { name: format!("text-{number}").into_bytes().into(), path });
    }
    roots
}
fn make_ctl(spawn: &Spawn, inherited: &Inherited) -> Proc {
    let words: Vec<&[u8]> = spawn.args[2..].iter().map(AsRef::as_ref).collect();
    let parsed = arguments::parse(&words).expect("settings ctl arguments");
    let pipe = |child| inherited.pipes.iter().find(|(number, _)| *number == child).expect("standard pipe").1;
    let root = |name: &[u8]| inherited.roots.iter().find(|(found, _)| found.as_ref() == name).map(|(_, fd)| *fd);
    let files = inherited
        .roots
        .iter()
        .filter_map(|(name, fd)| {
            let number = name.strip_prefix(b"text-")?;
            Some((std::str::from_utf8(number).expect("text argument").parse().expect("text index"), *fd))
        })
        .collect();
    let ctl = OneShot::with_text_files(
        Invocation {
            directory: Some(ctl_directory(spawn).file_name().expect("client leaf").as_bytes().into()),
            parsed,
        },
        Startup {
            parent: root(b"parent".as_slice()),
            file_parent: root(b"file".as_slice()),
            stdin: pipe(0),
            stdout: pipe(1),
            stderr: pipe(2),
            signals: inherited.signal,
            user: skein_shell::effective_user(),
            seed: skein_shell::seed().expect("ctl seed"),
        },
        files,
    )
    .expect("shipped ctl limits");
    Proc::Ctl(Box::new(ctl))
}

enum Proc {
    Serve(Box<temper_service_shell::Host<Service>>),
    Calls(Box<Calls>),
    Ctl(Box<OneShot>),
}
impl Delegate for Proc {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Serve(host) => host.as_ref(),
            Self::Calls(host) => host.as_ref(),
            Self::Ctl(host) => host.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Serve(host) => host.as_mut(),
            Self::Calls(host) => host.as_mut(),
            Self::Ctl(host) => host.as_mut(),
        }
    }
}
struct Judge {
    controls: Controls,
    stopping: bool,
}
impl Referee<Proc> for Judge {
    fn watch(&mut self, _now: Time, procs: &[Proc]) -> Result<(), String> {
        let [Proc::Serve(server), Proc::Calls(calls), ..] = procs else { unreachable!("sitting roots") };
        if !self.stopping {
            if server.is_empty() {
                let (words, code) = temper_service_shell::testing::hosted_ending(server);
                return Err(format!("serve ended early (exit {code}): {}", crate::without_secrets(words.as_bytes())));
            }
            if calls.is_empty() {
                self.controls.signal(0, ServiceSignal::Terminate);
                self.stopping = true;
            }
        }
        Ok(())
    }
    fn passed(&self) -> bool {
        self.stopping
    }
}

//! Two startup processes race an absent-file store using the shared real loop.
use super::*;
use crate::credentials::tests::Files;
use skein_io::kernel;
use skein_lib::{Duration, Time};
use skein_shell::{Clock, Delegate, Host};
use skein_world::{
    Referee,
    end_to_end::{Binary, Command, Mode, Streams},
    real,
};
use smith_local_process_world::terminal::{Action, Terminal};

#[derive(Default)]
struct Judge {
    finding: Option<String>,
}
impl Judge {
    fn act_checked(procs: &mut [Process]) -> Result<(), String> {
        for (binary, terminal) in [(0, 1), (2, 3)] {
            let exit = match procs.get(binary).ok_or("binary process")? {
                Process::Binary(binary) => binary.exit_status(),
                Process::Terminal(_) => None,
            };
            if let Some(exit) = exit {
                match procs.get_mut(terminal).ok_or("terminal process")? {
                    Process::Terminal(terminal) => terminal.finished(exit),
                    Process::Binary(_) => return Err("internal error: entered unreachable code".into()),
                }
            }
        }
        Ok(())
    }
}

impl Referee<Process> for Judge {
    fn act(&mut self, _now: Time, procs: &mut [Process]) {
        if self.finding.is_none() {
            self.finding = Self::act_checked(procs).err();
        }
    }
    fn watch(&mut self, _now: Time, _procs: &[Process]) -> Result<(), String> {
        if let Some(why) = &self.finding {
            return Err(why.clone());
        }
        Ok(())
    }
    fn passed(&self) -> bool {
        self.finding.is_none()
    }
}

#[test]
fn the_judge_keeps_an_act_finding_until_watch() {
    let mut judge = Judge::default();
    judge.act(Time::ZERO, &mut []);
    assert!(!judge.passed(), "an act finding prevents a pass");
    assert_eq!(judge.watch(Time::ZERO, &[]), Err("binary process".into()));
    assert!(!judge.passed(), "the finding remains held");
    assert_eq!(judge.watch(Time::ZERO, &[]), Err("binary process".into()));
}

enum Process {
    Binary(Box<Binary>),
    Terminal(Box<Terminal>),
}
impl Delegate for Process {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Binary(host) => host.as_ref(),
            Self::Terminal(host) => host.as_ref(),
        }
    }

    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Binary(host) => host.as_mut(),
            Self::Terminal(host) => host.as_mut(),
        }
    }
}
#[test]
fn host_id_drawn_by_two_processes_is_one_value() {
    // This is the child process's edge: all inputs below are explicit arguments.
    if let Some(base) = std::env::var_os("SMITH_HOST_ID_TEST") {
        let directory = Directory::from_base(base.into()).unwrap();
        let serial = std::env::var("SMITH_HOST_ID_SERIAL").unwrap();
        let byte = serial.parse::<u8>().unwrap();
        let value = read_or_draw(&directory, &[byte; 32]).unwrap();
        std::fs::write(directory.host_id().with_file_name(format!("seen-{serial}")), value).unwrap();
        return;
    }
    let files = Files::new("smith-credentials");
    let base = files.path().join("installation");
    let clock = Clock::new();
    let mut world = real::World::new(Judge::default());
    for serial in ["3", "4"] {
        let mut binary = Binary::start(
            Command {
                program: std::env::current_exe().unwrap(),
                arguments: vec![
                    "--exact".into(),
                    "credentials::host_id::process_tests::host_id_drawn_by_two_processes_is_one_value".into(),
                ],
                environment: vec![
                    ("HOME".into(), files.path().to_path_buf().into()),
                    ("XDG_CONFIG_HOME".into(), files.path().join("config").into()),
                    ("SMITH_HOST_ID_TEST".into(), base.clone().into()),
                    ("SMITH_HOST_ID_SERIAL".into(), serial.into()),
                ],
                directory: files.path().to_path_buf(),
            },
            Mode::Pipes,
            4096,
        )
        .unwrap();
        let streams = binary.take_streams().unwrap();
        let descriptors = streams.descriptors();
        let Streams::Pipes { input, output } = streams else { unreachable!() };
        world.spawn_with_fds(binary.descriptors(), || Process::Binary(Box::new(binary)));
        world.spawn_with_fds(descriptors, || {
            Process::Terminal(Box::new(Terminal::piped(input, output, vec![Action::Eof])))
        });
    }
    let outcome = world.run(&clock, Duration::from_secs(1));
    for process in outcome.procs {
        if let Process::Binary(binary) = process {
            assert_eq!(binary.exit_status(), Some(kernel::Exit::Code(0)));
        }
    }
    let first = std::fs::read(base.join("seen-3")).unwrap();
    let second = std::fs::read(base.join("seen-4")).unwrap();
    assert_eq!(first, second);
    assert!(
        first == b"urn:uuid:03030303-0303-4303-8303-030303030303"
            || first == b"urn:uuid:04040404-0404-4404-8404-040404040404"
    );
}

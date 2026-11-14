//! A process login over the fake issuer and the shared scripted browser.
use super::{Process, Ran};
use skein_io::kernel::Exit;
use skein_lib::{Duration, Time};
use skein_shell::{Clock, Host};
use skein_world::{
    Referee,
    end_to_end::{Binary, Command, Mode},
    real,
};
use smith_local_process_world::process::{BrowserProcess, IssuerProcess, Proc};
use smith_real_world::Scratch;

struct LoginJudge {
    finding: Option<String>,
    deadline: Time,
    visited: bool,
}
impl LoginJudge {
    fn act_checked(&mut self, _now: Time, procs: &mut [Process]) -> Result<(), String> {
        let binary = procs
            .iter()
            .find_map(|proc| if let Process::Binary(binary) = proc { Some(binary.as_ref()) } else { None })
            .ok_or("binary")?;
        let exit = binary.exit_status();
        let issuer_listening = procs.iter().any(|proc| match proc {
            Process::Script(Proc::Issuer(issuer)) => issuer.peer().address().is_some(),
            Process::Binary(_) | Process::Script(_) | Process::Sample(_) => false,
        });
        let url = if self.visited || !issuer_listening { None } else { sign_in_url(binary.stderr()) };
        if url.is_some() {
            self.visited = true;
        }
        for proc in procs {
            match proc {
                Process::Script(Proc::Browser(browser)) => {
                    if let Some(url) = &url {
                        browser.visit(url);
                    }
                    if exit.is_some() {
                        browser.shutdown();
                    }
                }
                Process::Script(Proc::Issuer(issuer)) if exit.is_some() => issuer.shutdown(),
                Process::Binary(_) | Process::Script(_) | Process::Sample(_) => {}
            }
        }
        Ok(())
    }
}

impl Referee<Process> for LoginJudge {
    fn act(&mut self, now: Time, procs: &mut [Process]) {
        if self.finding.is_none() {
            self.finding = self.act_checked(now, procs).err();
        }
    }
    fn watch(&mut self, _now: Time, _procs: &[Process]) -> Result<(), String> {
        if let Some(why) = &self.finding {
            return Err(why.clone());
        }
        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        Some(self.deadline)
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (now >= self.deadline).then(|| "fake login did not settle".into())
    }
    fn passed(&self) -> bool {
        self.finding.is_none() && (self.visited)
    }
}

pub(crate) fn sign_in_url(stderr: &[u8]) -> Option<Vec<u8>> {
    stderr
        .split_inclusive(|byte| *byte == b'\n')
        .find_map(|line| line.strip_suffix(b"\n").and_then(|line| line.strip_prefix(b"Sign in: ")).map(Vec::from))
}

#[must_use]
pub fn sign_in(scratch: &Scratch) -> Ran {
    let clock = Clock::new();
    let now = clock.now().now;
    let binary = Binary::start(
        Command {
            program: skein_scratch::paths::binary("smith"),
            arguments: vec!["login".into(), "test".into()],
            environment: vec![
                ("HOME".into(), scratch.path().into()),
                ("XDG_CONFIG_HOME".into(), scratch.path().join("config").into()),
                ("XDG_STATE_HOME".into(), scratch.path().join("state").into()),
                ("PATH".into(), "/usr/bin:/bin".into()),
            ],
            directory: scratch.path().into(),
        },
        Mode::Pipes,
        1_048_576,
    )
    .expect("login pipes");
    let mut world = real::World::new(LoginJudge {
        finding: None,
        deadline: now.saturating_add(Duration::from_secs(2)),
        visited: false,
    });
    world.spawn_with_fds(binary.descriptors(), || Process::Binary(Box::new(binary)));
    world.spawn(|| {
        Process::Script(Proc::Issuer(Box::new(IssuerProcess::at(skein_fake_peers::Transport::Plaintext, 34_444))))
    });
    world.spawn(|| Process::Script(Proc::Browser(Box::new(BrowserProcess::new()))));
    let outcome = world.run(&clock, Duration::from_secs(4));
    assert!(outcome.procs.iter().all(Host::is_empty), "login and neighbours drain");
    let Process::Binary(binary) = outcome.procs.first().expect("binary") else { unreachable!("binary") };
    assert_eq!(binary.exit_status(), Some(Exit::Code(0)), "native login succeeds");
    let Process::Script(Proc::Issuer(issuer)) = outcome.procs.get(1).expect("issuer") else { unreachable!("issuer") };
    let Process::Script(Proc::Browser(browser)) = outcome.procs.get(2).expect("browser") else {
        unreachable!("browser")
    };
    assert_eq!(smith_local_process_world::oauth::posts(issuer.peer()), 1, "one code exchange");
    assert!(browser.replied(), "the native listener answered the browser");
    smith_real_world::settled::assert_no_children();
    Ran { code: 0, stdout: Vec::new(), stderr: binary.stderr().into() }
}

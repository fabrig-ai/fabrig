//! Outside terminal observations on the existing shipped-binary world.
use super::{Process, Settling};
use skein_io::kernel::{Fd, Size};
use skein_lib::{Duration, Time};
use skein_shell::{Modes, resize_terminal, terminal_modes};
use skein_world::Referee;
use smith_local_process_world::process::Proc;

/// How long non-program shipped-binary worlds may take to settle.
pub(crate) const WORLD_PATIENCE: Duration = Duration::from_secs(2);

#[derive(Clone, Copy)]
pub(crate) enum Story {
    None,
    Modes,
    Resize,
    ProgramCheck,
    ProgramRun,
}

enum Resize {
    None,
    Response,
    Done,
}

struct Watch {
    master: Fd,
    before: Modes,
    during: Option<Modes>,
    after: Option<Modes>,
    resize: Resize,
}

pub(super) struct Judge {
    finding: Option<String>,
    pub common: Settling,
    watch: Option<Watch>,
    program: Option<super::program_interrupt::Watch>,
}

impl Judge {
    pub fn new(common: Settling, story: Story, master: Option<Fd>, scratch: &std::path::Path) -> Self {
        let watch = match story {
            Story::None | Story::ProgramRun | Story::ProgramCheck => None,
            Story::Modes | Story::Resize => {
                let master = master.expect("terminal story has a master");
                Some(Watch {
                    master,
                    before: terminal_modes(master).expect("initial terminal modes"),
                    during: None,
                    after: None,
                    resize: match story {
                        Story::Resize => Resize::Response,
                        Story::None | Story::Modes | Story::ProgramRun | Story::ProgramCheck => Resize::None,
                    },
                })
            }
        };
        let program = matches!(story, Story::ProgramCheck).then(|| {
            super::program_interrupt::Watch::new(scratch.join("state/smith/programs/work/fake-claude.observations"))
        });
        Self { finding: None, common, watch, program }
    }
}

impl Watch {
    fn inspect(&mut self, procs: &[Process]) -> Result<(), String> {
        let binary = procs
            .iter()
            .find_map(|process| match process {
                Process::Binary(binary) => Some(binary),
                Process::Script(_) | Process::Sample(_) => None,
            })
            .ok_or("shipped binary")?;
        if binary.exit_status().is_some() {
            if self.after.is_none() {
                self.after = Some(terminal_modes(self.master).map_err(|_| String::from("terminal modes after exit"))?);
            }
            return Ok(());
        }
        let shown = procs
            .iter()
            .find_map(|process| match process {
                Process::Script(Proc::Terminal(person)) => Some(person.shown()),
                Process::Binary(_) | Process::Script(_) | Process::Sample(_) => None,
            })
            .ok_or("scripted person")?;
        let modes = terminal_modes(self.master).map_err(|_| String::from("running terminal modes"))?;
        if modes != self.before {
            return Err(format!(
                "smith leaves running terminal modes alone\nleft: {:?}\nright: {:?}",
                modes, self.before
            ));
        }
        self.during = Some(modes);
        match self.resize {
            Resize::Response if skein_lib::bytes::find(shown, b"openai/gpt-6-luna; API key: in ").is_some() => {
                resize_terminal(self.master, Size { rows: 24, columns: 20 })
                    .map_err(|_| String::from("second window change"))?;
                self.resize = Resize::Done;
            }
            Resize::None | Resize::Response | Resize::Done => {}
        }
        Ok(())
    }

    fn passed(&self) -> bool {
        self.after.is_some()
            && self.during == Some(self.before)
            && self.after == Some(self.before)
            && matches!(self.resize, Resize::None | Resize::Done)
    }

    fn check(&self) -> Result<(), String> {
        if self.after.is_none() {
            return Ok(());
        }
        if self.during != Some(self.before) {
            return Err(format!(
                "running smith leaves terminal modes alone\nleft: {:?}\nright: {:?}",
                self.during,
                Some(self.before)
            ));
        }
        if self.after != Some(self.before) {
            return Err(format!(
                "exited smith leaves terminal modes alone\nleft: {:?}\nright: {:?}",
                self.after,
                Some(self.before)
            ));
        }
        Ok(())
    }
}

impl Judge {
    fn act_checked(&mut self, now: Time, procs: &mut [Process]) -> Result<(), String> {
        if let Some(watch) = &mut self.watch {
            watch.inspect(procs)?;
        }
        if let Some(program) = &mut self.program {
            program.act(now, procs)?;
        }
        self.common.act(now, procs);
        Ok(())
    }
}

impl Referee<Process> for Judge {
    fn act(&mut self, now: Time, procs: &mut [Process]) {
        if self.finding.is_none() {
            self.finding = self.act_checked(now, procs).err();
        }
    }

    fn watch(&mut self, now: Time, procs: &[Process]) -> Result<(), String> {
        if let Some(why) = &self.finding {
            return Err(why.clone());
        }
        self.common.watch(now, procs)?;
        if let Some(watch) = &self.watch {
            watch.check()?;
        }
        Ok(())
    }

    fn next_deadline(&self) -> Option<Time> {
        self.common
            .next_deadline()
            .into_iter()
            .chain(self.program.as_ref().and_then(super::program_interrupt::Watch::next_deadline))
            .min()
    }

    fn overdue(&self, now: Time) -> Option<String> {
        self.common.overdue(now)
    }

    fn passed(&self) -> bool {
        self.finding.is_none()
            && (self.common.passed()
                && self.watch.as_ref().is_none_or(Watch::passed)
                && self.program.as_ref().is_none_or(super::program_interrupt::Watch::passed))
    }
}

impl Story {
    pub(crate) fn budget(self, sign_in: bool) -> Duration {
        match self {
            // Widths are this story's subject, so its referee uses the world's
            // patience (skein/docs/foundation/testing-strategy.md, 2.9).
            Self::Resize => WORLD_PATIENCE,
            Self::ProgramRun | Self::ProgramCheck => Duration::from_secs(30),
            Self::None | Self::Modes => Duration::from_secs(if sign_in { 5 } else { 1 }),
        }
    }
}

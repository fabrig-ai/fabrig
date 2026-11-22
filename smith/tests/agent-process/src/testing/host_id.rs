//! Referee for processes publishing one installation identity.

use skein_io::kernel;
use skein_lib::Time;
use skein_shell::{Delegate, Host};
use skein_world::{Referee, end_to_end::Binary};

pub trait Settles {
    fn host(&self) -> &dyn Host;
    fn host_mut(&mut self) -> &mut dyn Host;
    fn finished(&mut self, exit: kernel::Exit);
}

#[derive(Debug, Default)]
pub struct Judge {
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

pub enum Process {
    Binary(Box<Binary>),
    Terminal(Box<dyn Settles>),
}
impl Delegate for Process {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Binary(host) => host.as_ref(),
            Self::Terminal(host) => host.host(),
        }
    }

    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Binary(host) => host.as_mut(),
            Self::Terminal(host) => host.host_mut(),
        }
    }
}

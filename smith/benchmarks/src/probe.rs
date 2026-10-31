//! Commands prepare frozen requests and print settled evidence; the shared
//! dispatcher owns execution and judging (benchmarks.md, sections 5.1 and 11.5).

use std::fmt::{self, Display};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::arm::ArmRef;
use crate::guard::Guard;
use crate::{Agent, AttemptResult, End, Identity, Judgment, Measure, ModelIdentity, Suite};

pub mod gating;

/// The committed source selected by the person running a command.
#[derive(Clone, Debug)]
pub enum Source {
    /// The clean checkout's committed tip.
    Head,
    /// An explicitly named source commit.
    Commit(String),
}

/// One probe command's task, agent, attempt ceiling and committed source.
#[derive(Debug)]
pub struct Order {
    pub probe: String,
    pub agent: Agent,
    pub attempts: u32,
    pub source: Source,
}

/// Immutable requests prepared outside the checkout for the dispatcher.
#[derive(Debug)]
pub struct Planned {
    pub run: String,
    pub suite: PathBuf,
    pub directory: PathBuf,
    pub requests: Vec<crate::run::Prepared>,
}

mod calibrate;
mod command;
mod plan;
mod report;

pub use calibrate::record_calibration;
use calibrate::{finish_calibration, selected_task};
pub use command::command;
pub use plan::plan;
use plan::{archived, check_parent, codex, prepare, task};
pub(crate) use report::judgment;
pub use report::{Counted, Line, count, line, model_deviation};
use report::{calibration_attempts, summary, verdict_name};

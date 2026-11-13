//! The redeploy's steps (temper/docs/design/redeploy.md, 4, 7).
use crate::{description::Description, hosts::Hosts, record::Journal};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Engine,
    Workers,
    Together,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    Done,
    Refused,
    Failed,
}

pub fn redeploy(
    _description: &Description,
    _commit: &str,
    _mode: Mode,
    journal: &mut Journal,
    hosts: &mut dyn Hosts,
) -> Outcome {
    let _ = journal.line(hosts.now(), "temper-redeploy run: not built");
    Outcome::Failed
}

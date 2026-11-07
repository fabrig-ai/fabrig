//! A native client's ask for an approval (client/README.md, 7.1).
use crate::{Domain, Limits, Request};
use jig_client::ApprovalCode;
use skein_lib::{Env, Queue, Wall};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State { Asking, Asked { code: ApprovalCode, expires: Wall }, NotAsked(jig_client::NotAsked) }
pub(crate) fn ask(_root: &mut Domain, _env: &Env<Limits>, _out: &mut Queue<Request>) {}
pub(crate) fn answered(_root: &mut Domain, _env: &Env<Limits>, _answer: jig_client::Approval, _out: &mut Queue<Request>) {}
pub(crate) fn progress(_root: &mut Domain) {}

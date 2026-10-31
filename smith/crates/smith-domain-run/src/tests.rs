//! Feed the domain events, inspect the requests that come out.

mod guide_room;

mod brief_room;

mod program;

mod open;

mod opening;

use alloc::boxed::Box;

use skein_lib::{Duration, Env, List, Queue, ReplyTo, Time, Token, Wall};

use crate::charter::{Endpoint, Grants, HostTool, Llm, Tools};
use crate::facts::{Answered, Asked, FactKind as Fact, Return};
use crate::outcome::{
    Change, ChangeSpec, Declared, Field, FieldRule, Item, OutcomeSpec, Problem, Problems, Verdict, VerdictRule,
};
use crate::prepare::{Found, Guide};
use crate::{
    Answer, Ask, AskRefusal, Budget, Charter, Delivery, Domain, End, Event, Exhausted, Exit, Failure, Fault, Invalid,
    Limits, Opening, Place, Policy, Ran, Read, Refusal, Request, Returned, Spend, Stop, fire, step, valid,
};
use crate::{Directory, Workspace};

mod support;
pub(crate) use support::{
    BUDGET, EXPIRY, Harness, LIMITS, PEER, agents, answered, ask, ask_by, bytes, cancelled, carried, carried_start,
    change, charter, coding, coding_workspace, comment, delivered, end_turn, ended, facts, failed, families, finish,
    finish_by, host_ask, host_submission, identity_open, message_driven_charter, mid_ask, mid_report, mid_running,
    mid_running_policy, mixed_workspace, observed_facts, pass_checks, priced, ran, receipts, refuse_workspace, refused,
    report, repository, returned, rule, spec, spend, submit_mid, text_charter, text_result, verdict, workspace,
    workspace_directory,
};

mod admission;
mod agents;
mod budget;
mod cancel;
mod discovery;
mod host;
mod landing;
mod lifecycle;
mod messages;
mod observations;
mod room;
mod workspace;

//! People as parties (programming-model.md, 4.5; domain/people/README.md).
//! Keeps identities, secret-free sign-ins, project roles, keyed answers,
//! task-derived bounded inbox references and each person's result and reply read position.
//! Knows tasks only by number; the parent checks authority and makes tasks.
//! `Save`/`Erase` join the parent's decision; the parent holds replies until
//! durable (domain/engine/README.md, 5.6).
//!
//! The boundary admits adoption and other owner requests by key, checks their
//! shape, routes its current project role to the parent, and retains committed
//! answers for replay after a restart
//! (domain/people/acts.md, section 1). This child never sees provider
//! credentials, protocol bytes, task internals or authority policy state.
//! The root supplies authenticated identities and authoritative role changes.
//! It mints fresh deployment person candidates and sign-in numbers; the protocol
//! keeps cookie secrets or their digests separately, named by the root-issued
//! sign-in number (domain/people/README.md, section 3; domain/engine/README.md, sections 4 and 5.4).
//!
//! `Domain::new` starts restoring. `Domain::start_restore` begins load/fit/link before input, capturing one wall
//! instant for both expiry walks. `start_link` leaves admission closed; the caller establishes all
//! links, drives `start_sweep` and its borrowed rows, and releases sweep writes durably before
//! `start_complete` projects live deadlines and opens admission (domain/engine/restart.md).
//! Ready requests share person-scoped keys across sign-ins: identical pending
//! copies join bounded waiters, saved copies replay the outcome, and conflicting
//! payloads refuse (domain/people/README.md, section 3, and domain/people/acts.md, section 1).
//! Application requests carry only scope, connector and the supplied 32-byte
//! digest. Their saved `Applied` result names the connector's outcome by token;
//! the connector owns its value and revision checks (client/domain/contract/requests.md,
//! section 3). They share every key, waiter, pending and retention bound with
//! jig's requests. A role in their scope admits them, or the sign-in alone for
//! the party's own; connector policy decides
//! whether that role may make the request.
//!
//! | Key state | Event | Next state | Emits |
//! | --- | --- | --- | --- |
//! | Absent | Authenticated application ask admitted by its scope | Flight | Route |
//! | Flight | Same metadata and digest | Flight | Joins bounded waiter |
//! | Flight | Another ask or digest | Flight | Key conflict |
//! | Flight | Applied token | Kept | Save and each waiter's reply marker |
//! | Kept | Same metadata and digest | Kept | Saved reply marker |
//! | Kept | Another ask or digest | Kept | Key conflict |
//! | Kept | Retention expires | Absent | Erase |
//!
//! Each `Route` requires exactly one matching `Decided`; permanent outcomes
//! save one keyed answer, while `Busy`/`NotReady` closes the flight without saving
//! so its key can retry . `Save` and `Erase` are
//! root decision outputs, not IO operations; task changes and keyed answers
//! share one atomic commit (domain/people/acts.md, section 1).
//! `fire` expires at most one due sign-in per call. The caller supplies an uncapped
//! output queue, accounts output bytes separately, and reclaims
//! retired flights at the iteration boundary. `Domain::role` supplies current membership for root
//! escalation coverage/standing without authentication or copying. `Domain::has_project` is a pure bounded startup
//! lookup; `Domain::person` is a pure sign-in lookup after restoration, checking
//! both its projected monotonic deadline and saved wall expiry for root result
//! routing, even before timer firing.
//! Optional diagnostic facts do not affect behavior (domain/people/README.md, section 3).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
mod restore;
pub use restore::{RestoreBound, RestoreCause, RestoreFailure, RestoreProjection};
mod settings;
pub use settings::{Running, Setting, SettingDetail, SettingFault, SettingMember, SettingRule};
mod scope;
pub use scope::ApplicationScope;
mod templates;
pub use templates::{TemplateBudget, TemplateDeadline, TemplateExecutor, TemplateOverride};
mod amendment;
mod names;
pub use names::{NAME_BYTES, NameRule, name_rule, same_name};
mod models;
pub use models::{
    MODEL_DESCRIPTION_BYTES, MODEL_NAME_BYTES, Models, remap_models, valid_model_context, valid_model_names,
};
mod batch;
pub use batch::{
    Action, Budget, Closer, Contract, Deadline, Dependency, EndedBy, FieldRule, Flags, Given, Member, Retention,
    Symbol, TaskExecutor, Verdict, member_bytes, shaped,
};
mod boundary;
mod results;
pub use results::{Forgettable, ReplyRef, ResultRef, Seen, Update};
mod capacity;
mod compound;
pub use compound::Operation;
mod decisions;
mod domain;
pub use domain::EndedMark;
mod facts;
mod limits;
mod llm;
pub use llm::{
    AccountMoveWhy, AccountWhy, LlmChoice, LlmLater, LlmWhy, MODELS, ModelEffort, SignInRedirectWhy, llm_bytes,
};
mod policy_value;
mod refusals;
#[cfg(test)]
mod tests;
mod words;
pub use amendment::{
    AddressChange, Amendment, Authority, Delegation, Executor, Grant, Last, Parameter, Pattern, PersonAddress,
    ResourceScope, ResultsWake, Spec, Tracking, WakePolicy, WakeRule, amendment_bytes, authority_bytes,
};
pub use boundary::{
    ADDRESS, AccountChange, AccountHolder, AccountHostFailure, AccountHostWhy, Act, ActGrant, ActKey, ActKind,
    ActStatus, Adoption, Ask, AskFamily, Counts, DeletionStep, Entry, EntryKind, EscalationChoice, EscalationDecision,
    Event, Holding, Identity, IdentityKey, InitialOwner, KeptDeletion, Key, Kind, LoginExit, Needs, NoteChange,
    NoteScope, Outcome, OutcomeFamily, PersonResult, ProjectRole, ProposalChoice, ProposalDecision, Proposer, Reply,
    Request, RequestKey, ResourceName, ResourceRole, Role, Seed, Signer, Stored, USES, Use, Whom,
};
pub use capacity::{CapacityAction, CapacityCause, CapacityRefusal};
pub use decisions::{Bound, Exceeded, FailureClass, HoldReason, Moved};
pub use domain::{Domain, account_holder_reads, application_part, application_part_done, fire, outputs_bound, step};
pub use facts::Fact;
pub use limits::{Limits, outcome_bytes, valid};
pub use policy_value::{
    Guard, Judge, PermissionRole, PolicyChange, PolicyRole, PolicyValue, Requirement, policy_bytes,
    policy_change_bytes, requirements_bytes,
};
pub use refusals::{
    Accepter, Again, CeilingDimension, CeilingHolder, DependencyStatus, Ending, HoldOwner, Lack, LimitKind, Malformed,
    NAMED_TASKS, Party, Refusal,
};

/// The deployment’s roster, not a project (domain/people/README.md, 3: the deployment’s roles).
pub const DEPLOYMENT: u32 = 0;

//! The forge connector's top (jig's domain/connectors.md, sections 2–8;
//! domain/forge/README.md, section 3 and domain/forge/adoption.md and domain/forge/keeping-up.md and domain/forge/README.md, sections 4–5 and domain/forge/change/README.md and domain/forge/queues.md and domain/forge/README.md, section 6 and domain/forge/branches.md and domain/forge/issues.md). It keeps adopted repositories, resource
//! holds, writer slots, subscriptions, procedure and projection rows, and
//! outbox entries. The client alone executes API calls; change and issues
//! decide policy without retained state.
//!
//! `step` accepts one parent or client event. Saves and erases join the
//! parent's decision. `Committed` is the only way a newly enqueued effect
//! reaches the client. `resume` and `fire` drive the client after earlier
//! saves have crossed that barrier. The top never knows tasks beyond numbers,
//! authorization grants. Its public projections encode the visitor records
//! (temper/docs/design/client/forge/README.md, 9).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![cfg_attr(test, expect(clippy::disallowed_macros, reason = "step tests use ordinary Rust assertions and fixtures"))]
#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
extern crate alloc;

pub use address::{PULL_ADDRESS_BYTES, host, pull_address};
#[expect(hidden_glob_reexports, reason = "the top keeps its module visibility around the shared vocabulary")]
use temper_engine_domain_forge_vocabulary::address;
pub use temper_engine_domain_forge_vocabulary::*;
mod adoption;
#[expect(hidden_glob_reexports, reason = "the top keeps its module visibility around the shared vocabulary")]
mod approvals;
pub mod history;
mod readiness;
pub use approvals::{ApprovalWait, ShortApproval};
pub use temper_engine_domain_forge_vocabulary::boundary;
#[expect(hidden_glob_reexports, reason = "the top keeps its module visibility around the shared vocabulary")]
mod brief;
mod brief_ci;
mod brief_document;
pub use brief_words::{BriefFate, BriefUnread};
#[expect(hidden_glob_reexports, reason = "the top keeps its module visibility around the shared vocabulary")]
use temper_engine_domain_forge_vocabulary::brief_words;
mod capabilities;
#[expect(hidden_glob_reexports, reason = "the top keeps its module visibility around the shared vocabulary")]
mod workflows;
pub use workflows::{WORKFLOW_BYTES, WORKFLOWS};
mod ci;
#[expect(hidden_glob_reexports, reason = "the top keeps its module visibility around the shared vocabulary")]
mod closer;
#[expect(hidden_glob_reexports, reason = "the top keeps its module visibility around the shared vocabulary")]
use temper_engine_domain_forge_vocabulary::commit;
mod connect;
pub mod member_message;
pub use closer::CloserRow;
#[expect(hidden_glob_reexports, reason = "the top keeps its module visibility around the shared vocabulary")]
mod status;
pub use status::{
    BRIEF_LEDE_BYTES, BRIEF_SENTENCE_BYTES, Brief, Looked, LookedAnswer, ProjectFacts, SITTING, StatusRunning,
    StatusView, status_paragraphs, status_subject,
};
#[expect(hidden_glob_reexports, reason = "the top keeps its module visibility around the shared vocabulary")]
mod drafts;
pub use drafts::{
    DRAFTS, DraftChange, DraftRefusal, DraftRepository, DraftView, DraftsView, LandedDraft, SetDraft, SetDraftAnswer,
    draft_path, drafts_subject,
};
#[expect(hidden_glob_reexports, reason = "the top keeps its module visibility around the shared vocabulary")]
mod away;
#[expect(hidden_glob_reexports, reason = "the top keeps its module visibility around the shared vocabulary")]
mod devices;
pub use devices::{
    AddDevice, DEVICES, DEVICES_SUBJECT, Device, DeviceAnswer, DeviceRefusal, DeviceView, DevicesView, Push, PushAnswer,
};
pub mod findings;
pub use away::{
    AWAY_SUBJECT, AskedView, Away, AwaysView, DECIDED_DAY_NS, DECIDED_KEPT_NS, DECIDED_OPTION_BYTES, DECIDED_OPTIONS,
    DECIDED_QUESTION_BYTES, DECIDED_WHY_BYTES, DECISIONS_SHOWN, Decided, DecidedCall, DecisionView, DecisionsView,
    OverturnAnswer, SetAwayAnswer, decided_subject,
};
pub use findings::{
    AreaRow, FileFinding, FileFindingAnswer, Filer, Finding, FindingAnswer, FindingDone, FindingKind, FindingRefusal,
    FindingReply, FindingRow, FindingState, FindingWrite, Findings, HandFinding, HandFindingAnswer, Holder, Severity,
    Triage, TriageFinding, TriageFindingAnswer, findings_subject, owner,
};
#[expect(hidden_glob_reexports, reason = "the top keeps its module visibility around the shared vocabulary")]
mod dismissed;
pub use dismissed::{DISMISSED_SUBJECT, DismissAnswer, PersonFact};
mod change_ci;
#[expect(hidden_glob_reexports, reason = "the top keeps its module visibility around the shared vocabulary")]
mod domain;
pub use domain::make::{BOT_EMAIL, BOT_LOGIN};
#[expect(hidden_glob_reexports, reason = "the top keeps its module visibility around the shared vocabulary")]
use temper_engine_domain_forge_vocabulary::effect_words;
#[expect(hidden_glob_reexports, reason = "the top keeps its module visibility around the shared vocabulary")]
use temper_engine_domain_forge_vocabulary::forge_user;
mod forges;
pub use forge_user::{FORGE_PROVIDER, key_user, user_key};
#[expect(hidden_glob_reexports, reason = "the top keeps its module visibility around the shared vocabulary")]
mod follow_check;
pub mod follows;
#[expect(hidden_glob_reexports, reason = "the top keeps its module visibility around the shared vocabulary")]
use temper_engine_domain_forge_vocabulary::gates;
#[expect(hidden_glob_reexports, reason = "the top keeps its module visibility around the shared vocabulary")]
mod held;
pub mod items;
#[expect(hidden_glob_reexports, reason = "the top keeps its module visibility around the shared vocabulary")]
use temper_engine_domain_forge_vocabulary::judge;
mod kinds;
pub use follows::{
    FollowAct, FollowCommit, FollowEntry, FollowLag, FollowParity, FollowQuestion, FollowRefusal, FollowRow,
    FollowSkip, FollowState, FollowVerdict, followed,
};
mod forming;
pub use forming::{Selection, WaitingUnit, select_units};
#[expect(hidden_glob_reexports, reason = "the top keeps its module visibility around the shared vocabulary")]
mod landing;
#[expect(hidden_glob_reexports, reason = "the top keeps its module visibility around the shared vocabulary")]
mod landing_log;
pub mod preparation;
pub use landing_log::LOGGED_TITLE_BYTES;
#[expect(hidden_glob_reexports, reason = "the top keeps its module visibility around the shared vocabulary")]
use temper_engine_domain_forge_vocabulary::limits;
mod link;
#[expect(hidden_glob_reexports, reason = "the top keeps its module visibility around the shared vocabulary")]
mod making;
pub use making::{
    MakeFailure, MakeFrom, MakeInto, MakeRefusal, MakeStep, MakingFailed, MakingKey, MakingState, RepositoryMaking,
};
#[expect(hidden_glob_reexports, reason = "the top keeps its module visibility around the shared vocabulary")]
mod panel;
#[expect(hidden_glob_reexports, reason = "the top keeps its module visibility around the shared vocabulary")]
mod public;
pub use public::{PublicAsk, PublicMade, PublicTask};

mod playground;
pub use red::{JobLog, RedNews, RedOn, red_news_words};
pub use result::{ChangeResult, ResultText};
pub use temper_engine_domain_forge_vocabulary::purpose;
#[expect(hidden_glob_reexports, reason = "the top keeps its module visibility around the shared vocabulary")]
use temper_engine_domain_forge_vocabulary::red;
#[expect(hidden_glob_reexports, reason = "the top keeps its module visibility around the shared vocabulary")]
use temper_engine_domain_forge_vocabulary::result;
#[expect(hidden_glob_reexports, reason = "the top keeps its module visibility around the shared vocabulary")]
mod reconcile;
#[expect(hidden_glob_reexports, reason = "the top keeps its module visibility around the shared vocabulary")]
use temper_engine_domain_forge_vocabulary::refusal;
mod removal;
pub use refusal::{RefusalCause, bound_words};
pub mod runs;
#[expect(hidden_glob_reexports, reason = "the top keeps its module visibility around the shared vocabulary")]
mod views;
pub use reconcile::{MissingLink, Need, RecordKind};
pub mod resources;
pub use settings::ForgeCharters;
pub use temper_engine_domain_forge_vocabulary::settings;
#[cfg(test)]
mod fuzzy;
#[cfg(test)]
#[path = "tests/public.rs"]
mod public_tests;
mod settle;
mod standing;
#[cfg(test)]
mod tests;
pub use temper_engine_domain_forge_vocabulary::text;
#[expect(hidden_glob_reexports, reason = "the top keeps its module visibility around the shared vocabulary")]
mod tiers;
mod topics;
#[expect(hidden_glob_reexports, reason = "the top keeps its module visibility around the shared vocabulary")]
mod verification;
pub use brief::{HOME_SECTION_BYTES, hex, home_access_reason};
pub use commit::commit_text;
pub use domain::{DEFAULT_CHANGE_TITLE, Domain, RestoreFit, RestoreSet, fire, resume, step, tag, untag, writer};
pub use gates::{GateCandidate, GateLast, GatePattern, Requires, Touched, covers, covers_branch, touched};
pub use judge::{Criterion, Freshness as JudgeFreshness, Reviewer, Verdict as JudgeVerdict};
pub use limits::{
    BOT_LOGIN_SUFFIX, CONNECT_WHY_BYTES, FIRST_NAME_BYTES, Limits, STOPPED_TEXT_BYTES, TierLimits, valid,
};
pub use panel::{
    MARKED, PANEL_AHEAD, PANEL_GATES, PANEL_NAME_BYTES, Panel, PanelApproval, PanelGate, PanelQuestion, PanelQuestions,
    PanelStands, PanelStep, PanelUsed, ReviewFacts, ReviewRead, UnmappedReview,
};
pub use temper_engine_domain_forge_landing::CHECK_NAME_BYTES;
pub use temper_engine_domain_forge_landing::{
    Combine as LandingCombine, Hold as LandingHold, Leave as LandingLeave, Phase as LandingPhase, Round as LandingRound,
};
pub use verification::LandingRed;
pub use views::{
    APPROVALS_SUBJECT, ChangesLanded, ChangesQueue, ChangesQueued, ChangesView, DependencyView, Document,
    FollowLagEntry, FollowView, FollowsView, GateProof, LandingGate, LandingLine, LandingMember, LandingStep,
    LandingTrain, LandingView, LandingsView, PrefixJudgement, TrainLanding, changes_subject, follows_subject,
    landings_subject,
};

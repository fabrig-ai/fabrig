//! The forge family's client records (client/forge/records.md and client/engine.md).
//! Forge creation and playground reads: temper/docs/design/client/forge/records.md.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

#[rustfmt::skip]
#[path = "generated/v1.rs"]
#[expect(clippy::similar_names, reason = "the schema names the approval fields rule and role")]
pub mod v1;

/// Kinds with typed settings values at temper's entrances
/// (temper/docs/design/protocol/configuration/settings.md, 2).
#[must_use]
pub const fn settings_kind(kind: u32) -> bool {
    kind == 1
        || kind == 2
        || kind == 3
        || kind == 6
        || kind == 7
        || kind == 8
        || kind == 9
        || kind == 10
        || kind == 11
        || kind == 12
        || kind == 13
}

/// The forge's visitor records (temper/docs/design/client/forge/README.md, 9.4).
#[rustfmt::skip]
pub mod public_v1 {
    pub use crate::{PUBLIC_CHANGE, PUBLIC_LANDING, PUBLIC_LANDINGS, PUBLIC_VERSION};
    include!("generated/public_v1.rs");
}

pub mod source;

mod public;
pub use public::{public_bounds, public_limits};

/// The visitor records' version (temper/docs/design/client/forge/README.md, 9.4).
pub const PUBLIC_VERSION: u16 = 1;
/// A public change (temper/docs/design/client/forge/README.md, 9.1).
pub const PUBLIC_CHANGE: u16 = 1;
/// A project's public landings (temper/docs/design/client/forge/README.md, 9.1).
pub const PUBLIC_LANDINGS: u16 = 2;
/// A public landing through a member (temper/docs/design/client/forge/README.md, 9.1).
pub const PUBLIC_LANDING: u16 = 3;

use alloc::boxed::Box;
use skein_lib::{Reader, Writer};

/// The forge connector's number on jig's client wire (client/forge/records.md).
pub const FORGE: u16 = 1;
/// The application's stable owners on jig's client wire (client/forge/records.md).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
    /// The forge's requests and records.
    Forge,
    /// The application's pages, using jig's operations and subjects.
    Pages,
    /// Calls made by the tools runner.
    Runner,
}
/// The owner's number, never assigned to another owner.
#[must_use]
pub const fn number(owner: Owner) -> u16 {
    match owner {
        Owner::Forge => FORGE,
        Owner::Pages => 2,
        Owner::Runner => 3,
    }
}
/// The owner a number names, if this build knows it.
#[must_use]
pub const fn owner(number: u16) -> Option<Owner> {
    match number {
        FORGE => Some(Owner::Forge),
        2 => Some(Owner::Pages),
        3 => Some(Owner::Runner),
        _ => None,
    }
}
/// Every record's version, served by both ends (client/forge/records.md).
pub const CLIENT_VERSION: u32 = 16;

/// The deployment's maximum number of forges (client/forge/records.md).
pub const FORGES: u32 = 16;
/// A party's dismissals on the wire (client/forge/records.md).
pub const DISMISSED: u32 = 8192;
/// The maximum installations per GitHub forge (client/forge/records.md).
pub const INSTALLATIONS: u32 = 64;
/// The maximum repository makings per forge (temper/docs/design/client/forge/records.md).
pub const MAKINGS: u32 = 64;
/// The set up's maximum number of deferred copies (protocol/setup/owned.md, 7).
pub const FIRST_COPIES: u32 = 16;
/// The last connect's maximum stop reason length (client/forge/records.md).
pub const CONNECT_WHY_BYTES: u32 = 256;
/// The box's maximum setup failure line length (client/forge/records.md).
pub const SET_UP_WHY_BYTES: u32 = 256;

/// The playground's maximum number of suggested requests (client/forge/records.md).
pub const CUES: u32 = 8;
/// Each suggested request's maximum UTF-8 byte length (client/forge/records.md).
pub const CUE_BYTES: u32 = 256;

/// The pull request address's maximum byte length (client/forge/records.md).
pub const ADDRESS_BYTES: u32 = 2048;
/// The change panel's maximum number of gates (client/forge/records.md).
pub const GATES: u32 = 64;
/// The queued change's maximum number of tasks ahead (client/forge/records.md).
pub const TASKS_AHEAD: u32 = 64;

/// A side's bounds on the forge family's fields (client/forge/family.md, 4 and client/forge/records.md).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bounds {
    pub segments: u32,
    pub segment_bytes: u32,
    pub gates: u32,
    pub approvals: u32,
    pub field_bytes: u32,
    pub checks: u32,
    pub contexts: u32,
    pub refusal_bytes: u32,
}

/// Set the schema limits from their source fields, refusing past a ceiling.
#[must_use]
pub fn limits(bounds: &Bounds) -> Option<v1::Limits> {
    let limits = v1::Limits {
        landing_exact_value: bounds.segment_bytes,
        landing_open_value: bounds.segment_bytes,
        landing_pattern_segments: bounds.segments,
        landing_pattern_segments_item: bounds.segment_bytes,
        landing_rule_gates: bounds.gates,
        landing_rule_approvals: bounds.approvals,
        landing_rule_sets: 6,
        adoption_owner: bounds.field_bytes,
        adoption_name: bounds.field_bytes,
        forge_row_host: ADDRESS_BYTES,
        forge_row_web_some: ADDRESS_BYTES,
        forge_row_repositories: 16,
        forge_row_makings: MAKINGS,
        make_copy_address: 512,
        dismiss_making_org: bounds.field_bytes,
        dismiss_making_name: bounds.field_bytes,
        make_repository_org: bounds.field_bytes,
        make_repository_name: bounds.field_bytes,
        making_made_branch: bounds.field_bytes,
        making_failed_words: 256,
        repository_making_org: bounds.field_bytes,
        repository_making_name: bounds.field_bytes,
        reached_name: bounds.field_bytes,
        link_address: ADDRESS_BYTES,
        forges_forges: FORGES,
        dismissals_dismissed: DISMISSED,
        set_up_failed_why: SET_UP_WHY_BYTES,
        group_failed_why: SET_UP_WHY_BYTES,
        connected_login: bounds.field_bytes,
        account_login: bounds.field_bytes,
        forge_row_installations: INSTALLATIONS,
        connect_stopped_why: CONNECT_WHY_BYTES,
        stopped_reason: bounds.field_bytes,
        pull_address: ADDRESS_BYTES,
        queued_ahead: TASKS_AHEAD,
        change_panel_owner: bounds.field_bytes,
        change_panel_name: bounds.field_bytes,
        change_panel_branch: bounds.field_bytes,
        change_panel_landing: bounds.field_bytes,
        change_panel_gates: GATES,
        managed_forge_git: ADDRESS_BYTES,
        managed_forge_web_some: ADDRESS_BYTES,
        playground_answer_owner: bounds.field_bytes,
        playground_answer_name: bounds.field_bytes,
        playground_answer_cues: CUES,
        playground_answer_cues_item: CUE_BYTES,
        first_ask_org: bounds.field_bytes,
        first_ask_repository: bounds.field_bytes,
        first_empty_org: bounds.field_bytes,
        first_empty_repository: bounds.field_bytes,
        first_repository_org: bounds.field_bytes,
        first_repository_repository: bounds.field_bytes,
        adopt_owner: bounds.field_bytes,
        adopt_name: bounds.field_bytes,
        adopt_prefix: bounds.field_bytes,
        adopt_landing: bounds.field_bytes,
        adopted_owner: bounds.field_bytes,
        adopted_name: bounds.field_bytes,
        rule_branch: bounds.field_bytes,
        adopt_checks: bounds.checks,
        rule_contexts: bounds.contexts,
        rule_contexts_item: bounds.field_bytes,
        core_refused_refusal: bounds.refusal_bytes,
        ..v1::CEILINGS
    };
    match v1::check_limits(&limits) {
        Ok(()) => Some(limits),
        Err(_) => None,
    }
}

/// Encode the request for jig's keyed record and the client's small store.
#[must_use]
pub fn requesting(record: &v1::Requesting) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}

/// Encode the outcome for jig's owned record.
#[must_use]
pub fn answer(record: &v1::Answer) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}

#[cfg(test)]
mod tests;

/// Encode a project's follows (temper/docs/design/domain/forge/follows.md, 7).
#[must_use]
pub fn follows_panel(record: &v1::FollowsPanel) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}
/// Decode one complete follows document.
#[must_use]
pub fn decode_follows_panel(bytes: &[u8], limits: &v1::Limits) -> Option<v1::FollowsPanel> {
    let mut reader = Reader::new(bytes);
    let record = v1::FollowsPanel::decode(limits, &mut reader).ok()?;
    if reader.is_empty() { Some(record) } else { None }
}

/// Encode which of the family's reads is requested.
#[must_use]
pub fn reading(record: &v1::Reading) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}
/// Decode one complete family reading.
#[must_use]
pub fn decode_reading(bytes: &[u8], limits: &v1::Limits) -> Option<v1::Reading> {
    let mut reader = Reader::new(bytes);
    let record = v1::Reading::decode(limits, &mut reader).ok()?;
    if reader.is_empty() { Some(record) } else { None }
}
/// Encode the playground's answer; no playground is represented by no record.
#[must_use]
pub fn playground_answer(record: &v1::PlaygroundAnswer) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}
/// Decode a complete playground answer, refusing empty cues (temper/docs/design/client/forge/records.md).
#[must_use]
pub fn decode_playground_answer(bytes: &[u8], limits: &v1::Limits) -> Option<v1::PlaygroundAnswer> {
    let mut reader = Reader::new(bytes);
    let record = v1::PlaygroundAnswer::decode(limits, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    for cue in record.cues() {
        if cue.is_empty() {
            return None;
        }
    }
    Some(record)
}

/// Encode the deployment's answer (temper/docs/design/client/forge/records.md).
#[must_use]
pub fn deployment_answer(record: &v1::DeploymentAnswer) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}
/// Decode one complete deployment answer (temper/docs/design/client/forge/records.md).
#[must_use]
pub fn decode_deployment_answer(bytes: &[u8], limits: &v1::Limits) -> Option<v1::DeploymentAnswer> {
    let mut reader = Reader::new(bytes);
    let record = v1::DeploymentAnswer::decode(limits, &mut reader).ok()?;
    if reader.is_empty() { Some(record) } else { None }
}

/// Encode the program's environment (temper/docs/design/client/forge/records.md).
#[must_use]
pub fn program_environment_answer(record: &v1::ProgramEnvironmentAnswer) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}
/// Decode one complete program environment answer (temper/docs/design/client/forge/records.md).
#[must_use]
pub fn decode_program_environment_answer(bytes: &[u8], limits: &v1::Limits) -> Option<v1::ProgramEnvironmentAnswer> {
    let mut reader = Reader::new(bytes);
    let record = v1::ProgramEnvironmentAnswer::decode(limits, &mut reader).ok()?;
    if reader.is_empty() { Some(record) } else { None }
}

/// Encode which of the family's subjects is watched (client/forge/records.md).
#[must_use]
pub fn watching(record: &v1::Watching) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}
/// Decode one complete family watch (client/forge/records.md).
#[must_use]
pub fn decode_watching(bytes: &[u8], limits: &v1::Limits) -> Option<v1::Watching> {
    let mut reader = Reader::new(bytes);
    let record = v1::Watching::decode(limits, &mut reader).ok()?;
    if reader.is_empty() { Some(record) } else { None }
}
/// Encode the change's whole panel (client/forge/records.md; client/forge/served.md, 2).
#[must_use]
pub fn change_panel(record: &v1::ChangePanel) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}
/// Decode a complete panel, refusing a queued place before 1 (client/forge/records.md).
#[must_use]
pub fn decode_change_panel(bytes: &[u8], limits: &v1::Limits) -> Option<v1::ChangePanel> {
    let mut reader = Reader::new(bytes);
    let record = v1::ChangePanel::decode(limits, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    if let v1::Step::Queued(queued) = record.step()
        && queued.place() == 0
    {
        return None;
    }
    Some(record)
}

/// Decode one complete requesting (client/forge/records.md).
#[must_use]
pub fn decode_requesting(bytes: &[u8], limits: &v1::Limits) -> Option<v1::Requesting> {
    let mut reader = Reader::new(bytes);
    let record = v1::Requesting::decode(limits, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    match record.request() {
        v1::Request::Connect(connect) => {
            if !valid_making(connect.forge()) {
                return None;
            }
            match connect.how() {
                v1::How::GithubApp(app) if app.id() == 0 => return None,
                v1::How::GithubApp(_) | v1::How::GithubManifest(_) | v1::How::ForgejoTokens(_) => {}
                v1::How::ForgejoAdmin(admin) => {
                    if let Some(first) = admin.first()
                        && !runner_label(first.label())
                    {
                        return None;
                    }
                }
            }
        }
        v1::Request::SetUp(value) => {
            if !valid_making(value.forge()) {
                return None;
            }
        }
        v1::Request::Accept(value) if value.installation() == 0 => return None,
        v1::Request::Reachable(value) if value.page() == 0 => return None,
        v1::Request::Reachable(_)
        | v1::Request::Accept(_)
        | v1::Request::Install(_)
        | v1::Request::DismissMaking(_)
        | v1::Request::MakeRepository(_)
        | v1::Request::Adopt(_)
        | v1::Request::AddDevice(_)
        | v1::Request::RemoveDevice(_)
        | v1::Request::SetDraft(_)
        | v1::Request::FollowAct(_)
        | v1::Request::TryAccount(_)
        | v1::Request::Dismiss(_)
        | v1::Request::Looked(_)
        | v1::Request::SetAway(_)
        | v1::Request::Overturn(_)
        | v1::Request::Replace(_)
        | v1::Request::RemoveForge(_)
        | v1::Request::StartPlayground
        | v1::Request::EndPlayground => {}
    }
    Some(record)
}

// temper/docs/design/client/forge/records.md: managed addresses enter whole.
fn valid_making(making: &v1::Making) -> bool {
    match making {
        v1::Making::Kept(_) => true,
        v1::Making::New(value) => match value.forge() {
            v1::ForgeAsk::Github | v1::ForgeAsk::Owned => true,
            v1::ForgeAsk::Managed(value) | v1::ForgeAsk::Connected(value) => {
                !value.host().is_empty()
                    && !value.host().contains(&0)
                    && !value.host().contains(&b'/')
                    && value.port() != 0
                    && match value.git_program() {
                        Some(program) => !program.is_empty() && !program.contains(&0),
                        None => true,
                    }
            }
        },
    }
}

/// Encode the family's forges (client/forge/records.md).
#[must_use]
pub fn forges(record: &v1::Forges) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}

/// Decode one complete forges (client/forge/records.md).
#[must_use]
pub fn decode_forges(bytes: &[u8], limits: &v1::Limits) -> Option<v1::Forges> {
    let mut reader = Reader::new(bytes);
    let record = v1::Forges::decode(limits, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    let mut previous = None;
    for row in record.forges() {
        if let Some(number) = previous
            && number >= row.number()
        {
            return None;
        }
        previous = Some(row.number());
    }
    Some(record)
}

/// Encode a party's dismissals (client/forge/records.md).
#[must_use]
pub fn dismissals(record: &v1::Dismissals) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}

/// Decode one complete dismissals document (client/forge/records.md).
#[must_use]
pub fn decode_dismissals(bytes: &[u8], limits: &v1::Limits) -> Option<v1::Dismissals> {
    let mut reader = Reader::new(bytes);
    let record = v1::Dismissals::decode(limits, &mut reader).ok()?;
    if reader.is_empty() { Some(record) } else { None }
}

/// Whether a runner label can be used as one workflow scalar (client/engine.md).
#[must_use]
pub fn runner_label(label: &[u8]) -> bool {
    if label.is_empty() || label.len() > 64 {
        return false;
    }
    for byte in label {
        if !byte.is_ascii_alphanumeric() && !b"._-".contains(byte) {
            return false;
        }
    }
    true
}

/// Encode the watched landing view (domain/landing.md, 14).
#[must_use]
pub fn landing_panel(record: &v1::LandingPanel) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}
/// Decode one complete landing view (domain/landing.md, 14).
#[must_use]
pub fn decode_landing_panel(bytes: &[u8], limits: &v1::Limits) -> Option<v1::LandingPanel> {
    let mut reader = Reader::new(bytes);
    let record = v1::LandingPanel::decode(limits, &mut reader).ok()?;
    if reader.is_empty() { Some(record) } else { None }
}

/// Encode the watched landing view (domain/landing.md, 14).
#[must_use]
pub fn landings_panel(record: &v1::LandingsPanel) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}
/// Decode one complete landing view (domain/landing.md, 14).
#[must_use]
pub fn decode_landings_panel(bytes: &[u8], limits: &v1::Limits) -> Option<v1::LandingsPanel> {
    let mut reader = Reader::new(bytes);
    let record = v1::LandingsPanel::decode(limits, &mut reader).ok()?;
    if reader.is_empty() { Some(record) } else { None }
}

/// Encode a project's watched changes (temper/docs/design/client/forge/records.md).
#[must_use]
pub fn changes_panel(record: &v1::ChangesPanel) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}
/// Decode one complete changes view (temper/docs/design/client/forge/records.md).
#[must_use]
pub fn decode_changes_panel(bytes: &[u8], limits: &v1::Limits) -> Option<v1::ChangesPanel> {
    let mut reader = Reader::new(bytes);
    let record = v1::ChangesPanel::decode(limits, &mut reader).ok()?;
    if reader.is_empty() { Some(record) } else { None }
}

/// Encode a project's landed drafts (temper/docs/design/client/forge/records.md).
#[must_use]
pub fn drafts_panel(record: &v1::DraftsPanel) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}
/// Decode one complete landed-drafts view (temper/docs/design/client/forge/records.md).
#[must_use]
pub fn decode_drafts_panel(bytes: &[u8], limits: &v1::Limits) -> Option<v1::DraftsPanel> {
    let mut reader = Reader::new(bytes);
    let record = v1::DraftsPanel::decode(limits, &mut reader).ok()?;
    if reader.is_empty() { Some(record) } else { None }
}

/// Encode a project's watched ways (temper/docs/design/client/forge/records.md).
#[must_use]
pub fn ways_panel(record: &v1::WaysPanel) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}
/// Decode a project's watched ways, whole (temper/docs/design/client/forge/records.md).
#[must_use]
pub fn decode_ways_panel(bytes: &[u8], limits: &v1::Limits) -> Option<v1::WaysPanel> {
    let mut reader = Reader::new(bytes);
    let record = v1::WaysPanel::decode(limits, &mut reader).ok()?;
    if reader.is_empty() { Some(record) } else { None }
}

/// Encode a requested view (temper/docs/design/client/forge/records.md).
#[must_use]
pub fn view_answer(record: &v1::ViewAnswer) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}
/// Decode a requested view, whole (temper/docs/design/client/forge/records.md).
#[must_use]
pub fn decode_view_answer(bytes: &[u8], limits: &v1::Limits) -> Option<v1::ViewAnswer> {
    let mut reader = Reader::new(bytes);
    let record = v1::ViewAnswer::decode(limits, &mut reader).ok()?;
    if reader.is_empty() { Some(record) } else { None }
}

/// The approvals document's bytes (temper/docs/design/client/forge/records.md).
#[must_use]
pub fn approvals(record: &v1::Approvals) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}
/// The approvals document, whole, with no trailing byte (temper/docs/design/client/forge/records.md).
#[must_use]
pub fn decode_approvals(bytes: &[u8], limits: &v1::Limits) -> Option<v1::Approvals> {
    let mut reader = Reader::new(bytes);
    let record = v1::Approvals::decode(limits, &mut reader).ok()?;
    if reader.is_empty() { Some(record) } else { None }
}

/// Encode a party's project status (temper/docs/design/client/forge/records.md).
#[must_use]
pub fn project_status(record: &v1::ProjectStatus) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}
/// Decode one complete project status (temper/docs/design/client/forge/records.md).
#[must_use]
pub fn decode_project_status(bytes: &[u8], limits: &v1::Limits) -> Option<v1::ProjectStatus> {
    let mut reader = Reader::new(bytes);
    let record = v1::ProjectStatus::decode(limits, &mut reader).ok()?;
    if reader.is_empty() { Some(record) } else { None }
}

/// Encode a party's away rows (temper/docs/design/client/forge/records.md).
#[must_use]
pub fn aways(record: &v1::Aways) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}
/// Decode one complete aways document (temper/docs/design/client/forge/records.md).
#[must_use]
pub fn decode_aways(bytes: &[u8], limits: &v1::Limits) -> Option<v1::Aways> {
    let mut reader = Reader::new(bytes);
    let record = v1::Aways::decode(limits, &mut reader).ok()?;
    if reader.is_empty() { Some(record) } else { None }
}

/// Encode a project's decisions (temper/docs/design/client/forge/records.md).
#[must_use]
pub fn decisions(record: &v1::Decisions) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}
/// Decode one complete decisions document (temper/docs/design/client/forge/records.md).
#[must_use]
pub fn decode_decisions(bytes: &[u8], limits: &v1::Limits) -> Option<v1::Decisions> {
    let mut reader = Reader::new(bytes);
    let record = v1::Decisions::decode(limits, &mut reader).ok()?;
    if reader.is_empty() { Some(record) } else { None }
}

/// Encode the current value of a forge setting (client/forge/served.md).
#[must_use]
pub fn setting_answer(record: &v1::SettingAnswer) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}
/// Decode one complete forge setting answer.
#[must_use]
pub fn decode_setting_answer(bytes: &[u8], limits: &v1::Limits) -> Option<v1::SettingAnswer> {
    let mut reader = Reader::new(bytes);
    let record = v1::SettingAnswer::decode(limits, &mut reader).ok()?;
    if reader.is_empty() { Some(record) } else { None }
}
/// Encode the items of one pending proposal (client/forge/served.md).
#[must_use]
pub fn proposal_items_answer(record: &v1::ProposalItemsAnswer) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}
/// Decode one complete proposal items answer.
#[must_use]
pub fn decode_proposal_items_answer(bytes: &[u8], limits: &v1::Limits) -> Option<v1::ProposalItemsAnswer> {
    let mut reader = Reader::new(bytes);
    let record = v1::ProposalItemsAnswer::decode(limits, &mut reader).ok()?;
    if reader.is_empty() { Some(record) } else { None }
}

/// A goal reads its work as a goal (temper/docs/design/domain/agent/tools.md, 2).
pub const GOAL_SCOPE: &[u8] =
    b"You are a goal: the work your person asked for, yours to plan and to change; you start no goal but an advisor.";
/// A goal asks for agreement before changes (temper/docs/design/domain/agent/agents.md, 2).
pub const GOAL_AGREED: &[u8] =
    b"The work is to be agreed: before anything is changed, ask your requester your plan with message, as a question, and wait for the answer.";

/// Encode a person's devices (temper/docs/design/client/forge/records.md).
#[must_use]
pub fn devices(record: &v1::Devices) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured devices size fits"));
    record.encode(&mut writer).expect("measured devices room");
    writer.finish()
}
/// Decode one whole devices document, refusing trailing bytes.
#[must_use]
pub fn decode_devices(bytes: &[u8], limits: &v1::Limits) -> Option<v1::Devices> {
    let mut reader = Reader::new(bytes);
    let record = v1::Devices::decode(limits, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    Some(record)
}

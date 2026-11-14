//! Client heartbeat and local sign-in entrances (configuration.md, 3).
use crate::{Directories, Path, Problems, check, read};
mod issuer;
mod provider;
use alloc::boxed::Box;
pub use issuer::{ProviderUrls, provider_issuer};
use jig_protocol_engine_clients as kit;
pub use provider::{Provider, ProviderClient, provider_key};
use skein_json::Document;
use skein_lib::{Duration, List, Secret};
#[derive(Clone, Debug)]
pub struct Clients {
    pub heartbeat: Duration,
    pub link: bool,
    pub approval: bool,
    pub provider: Option<Provider>,
}
pub(crate) fn parse(
    document: &Document,
    at: u32,
    path: &Path,
    directories: &Directories,
    problems: &mut Problems,
) -> Option<Clients> {
    let mut object = check::object(document, at, path, problems, "clients")?;
    let heartbeat = check::duration(&mut object, document, b"heartbeat_ms", path, problems);
    let mut link = None;
    if let Some(at) = object.required(document, b"link", path, problems) {
        match read::boolean(document, at) {
            Ok(value) => link = Some(value),
            Err(what) => problems.push(path.member(b"link"), what),
        }
    }
    let mut approval = Some(false);
    if let Some(at) = object.member(document, b"approval") {
        approval = match read::boolean(document, at) {
            Ok(value) => Some(value),
            Err(what) => {
                problems.push(path.member(b"approval"), what);
                None
            }
        };
    }
    let provider = provider::parse(&mut object, document, path, directories, problems);
    object.finish(document, path, problems);
    Some(Clients { heartbeat: heartbeat?, link: link?, approval: approval?, provider })
}
/// The startup catalogue in the clients' numbered positions (domain/engine/settings.md, 11).
#[must_use]
pub fn models(llm: &crate::Llm) -> Box<[jig_protocol_engine_clients::Model]> {
    jig_protocol_engine_clients::translate::llm::catalogue(&llm.models)
}

/// What an application adds to the clients' settings (protocol/service.md, 11.6).
#[derive(Debug)]
pub struct Application {
    pub link: Option<jig_domain_people::Identity>,
    pub served: List<kit::Served>,
    /// Its own public kinds, after jig's.
    pub public: List<kit::PublicKind>,
    pub moved_bytes: u64,
}
/// The provider of kind link, for the printed link's identity.
#[must_use]
pub fn link_provider(config: &crate::Engine) -> Option<u16> {
    let mut link = None;
    for provider in &config.providers {
        match provider.kind {
            crate::ProviderKind::Link => link = Some(provider.number),
            crate::ProviderKind::Deployment | crate::ProviderKind::Application => {}
        }
    }
    link
}
/// The problem of the engine's clients' settings, at `engine.clients`.
#[must_use]
pub fn invalid(what: crate::What) -> Problems {
    let mut problems = Problems::empty();
    problems.push(Path::root().member(b"engine").member(b"clients"), what);
    problems
}
/// The owner the printed link signs in, the first with an identity at `provider`.
#[must_use]
#[expect(clippy::manual_find, reason = "step code uses no closures (programming-model.md, 10.3)")]
pub fn link_owner(config: &crate::Engine, provider: u16) -> Option<&jig_domain_people::InitialOwner> {
    for owner in &config.owners {
        if owner.identity.provider == provider {
            return Some(owner);
        }
    }
    None
}
#[must_use]
pub fn public_bounds(config: &crate::Engine) -> jig_client_protocol_wire::PublicIndexBounds {
    jig_client_protocol_wire::PublicIndexBounds {
        goals: config.limits.core.public.goals,
        ended: config.limits.core.public.ended,
        lines: config.limits.core.public.lines,
        depth: config.limits.core.public.depth,
        projects: config.limits.core.people.projects,
    }
}
/// The clients' settings, with no provider and no addresses (protocol/client/openid.md).
#[must_use]
pub fn settings(config: &crate::Engine, application: &Application, seed: Secret, stream: u64) -> kit::Settings {
    let mut providers = List::with_capacity(u32::try_from(config.providers.len()).expect("bounded providers"));
    let mut deployment_provider = 0;
    for provider in &config.providers {
        match provider.kind {
            crate::ProviderKind::Link => {}
            crate::ProviderKind::Deployment => deployment_provider = provider.number,
            crate::ProviderKind::Application => {
                providers.push(provider.number).expect("provider count");
            }
        }
    }
    let mut public = List::with_capacity(3_u32.checked_add(application.public.len()).expect("public kinds"));
    for kind in kit::jig_kinds(&public_bounds(config)) {
        public.push(kind).expect("public kinds");
    }
    for kind in &application.public {
        public.push(*kind).expect("public kinds");
    }
    kit::Settings {
        public,
        private: jig_protocol_engine_clients::Private::none(),
        public_bounds: public_bounds(config),
        provider: None,
        addresses: Box::new([]),
        link: application.link.clone(),
        link_secret: None,
        local_secret: None,
        approval: config.clients.approval,
        deployment_provider,
        heartbeat: config.clients.heartbeat,
        rows: config.limits.core.documents.rows,
        served: application.served.clone(),
        providers,
        seed,
        stream,
        models: models(&config.llm),
        redirect_bytes: config.limits.driver.client.uri_bytes,
        key_bytes: config.limits.driver.client.document.token_bytes.min(config.limits.clients.wire.keyed_value_some),
        moved_bytes: application.moved_bytes,
    }
}

pub(crate) fn check_provider(engine: &crate::Engine, problems: &mut Problems) {
    let Some(provider) = &engine.clients.provider else { return };
    let path = Path::root().member(b"clients").member(b"provider").member(b"addresses");
    for (index, number) in provider.addresses.iter().enumerate() {
        let mut accepted = false;
        for configured in &engine.providers {
            accepted |= configured.number == *number
                && (configured.kind == crate::ProviderKind::Application
                    || configured.kind == crate::ProviderKind::Link);
        }
        if !accepted {
            problems.push(check::index(&path, index), crate::What::Refers);
        }
    }
}

/// Refuse configurations whose admitted settings exceed client wire 22.
pub(crate) fn check_settings(limits: &crate::Limits, problems: &mut Problems) {
    let core = &limits.core;
    let wire = &limits.clients.wire;
    let path = Path::root().member(b"limits");
    // settings-v1 bounds a proposal to 128 item heads; its review shares the document envelope.
    if wire.proposal_action_settings_items < 128 || wire.retained_action_settings_items < 128 {
        problems.push(path.member(b"clients.wire.proposal_heads"), crate::What::Bound);
    }
    if wire.settings_review_items < wire.owned_record {
        problems.push(path.member(b"clients.wire.settings_review"), crate::What::Bound);
    }
    if core.settings.proposals > wire.settings_document_pending {
        problems.push(path.member(b"core.settings.proposals"), crate::What::Bound);
    }
    // Defaults and changes may name disjoint settings; model numbers share one 64-row range.
    let inventory = match jig_domain_core_vocabulary::settings::capacity(core) {
        Some(count) => inventory_settings(count),
        None => None,
    };
    if match inventory {
        Some(count) => count > wire.settings_document_settings,
        None => true,
    } {
        problems.push(path.member(b"tasks.charters"), crate::What::Bound);
        problems.push(path.member(b"core.templates"), crate::What::Bound);
        problems.push(path.member(b"authority.projects"), crate::What::Bound);
    }
    for (name, value, maximum) in [
        (b"accounts.accounts".as_slice(), core.accounts.accounts, wire.me_model_accounts),
        (b"core.templates".as_slice(), core.templates, wire.me_project_defaults),
        (b"authority.roles".as_slice(), core.authority.roles, wire.setting_policy_roles),
        (
            b"authority.requirements".as_slice(),
            core.authority.requirements,
            wire.setting_policy_requirements.min(wire.rules_requirements),
        ),
        (b"authority.grants".as_slice(), core.authority.grants, wire.setting_policy_projections),
        (b"authority.implications".as_slice(), core.authority.implications, wire.rules_implies),
        (b"core.templates".as_slice(), core.templates, wire.setting_policy_templates),
        (b"core.settings.permissions".as_slice(), core.settings.permissions, wire.setting_policy_permissions),
        (b"tasks.charter_documents".as_slice(), core.tasks.charter_documents, wire.charter_documents),
        (b"core.settings.alternatives".as_slice(), core.settings.alternatives, wire.charter_alternatives),
        (b"core.run_bytes".as_slice(), core.run_bytes, wire.charter_instructions),
        (b"documents.text_bytes".as_slice(), core.documents.text_bytes, wire.document_text_bytes),
        (b"notes.body_bytes".as_slice(), core.notes.body_bytes, wire.document_text_bytes),
        (b"people.words".as_slice(), core.people.words, wire.document_text_bytes),
        (b"documents.rows".as_slice(), core.documents.rows, wire.revisions_page_rows),
        (b"documents.lines".as_slice(), core.documents.lines, wire.revision_row_items),
    ] {
        if value > maximum {
            problems.push(path.member(name), crate::What::Bound);
        }
    }
}
fn inventory_settings(count: u32) -> Option<u32> {
    count.checked_mul(2)?.checked_sub(jig_domain_tasks::MODELS)
}

#[cfg(test)]
mod settings_tests {
    use super::*;
    fn ceiling(name: &[u8], maximum: u32) {
        let mut limits = crate::profile(crate::Profile::Small);
        crate::overrides::apply(&mut limits, name, u64::from(maximum)).unwrap();
        let mut problems = Problems::empty();
        check_settings(&limits, &mut problems);
        assert!(problems.is_empty(), "at ceiling: {name:?}: {problems:?}");
        crate::overrides::apply(&mut limits, name, u64::from(maximum) + 1).unwrap();
        check_settings(&limits, &mut problems);
        assert_eq!(problems.problems.len(), 1, "one past: {name:?}: {problems:?}");
        assert_eq!(problems.problems.get(0).unwrap().path, Path::root().member(b"limits").member(name));
    }
    #[test]
    fn settings_proposal_reasons_fit_the_wire() {
        let mut limits = crate::profile(crate::Profile::Small);
        limits.clients.wire.document_text_bytes = limits.core.people.words;
        limits.core.documents.text_bytes = 128;
        limits.core.notes.body_bytes = 128;
        let mut problems = Problems::empty();
        check_settings(&limits, &mut problems);
        assert!(problems.is_empty(), "at ceiling: {problems:?}");
        limits.core.people.words += 1;
        check_settings(&limits, &mut problems);
        assert_eq!(problems.problems.len(), 1);
        assert_eq!(
            problems.problems.get(0).expect("one problem").path,
            Path::root().member(b"limits").member(b"people.words")
        );
    }
    #[test]
    fn pending_settings_fit_the_wire_inventory() {
        let maximum = crate::profile(crate::Profile::Small).clients.wire.settings_document_pending;
        ceiling(b"core.settings.proposals", maximum);
    }
    #[test]
    fn offer_accounts_and_template_previews_fit_the_wire() {
        for (name, field) in [(b"accounts.accounts".as_slice(), "accounts"), (b"core.templates".as_slice(), "defaults")]
        {
            let mut limits = crate::profile(crate::Profile::Small);
            let value = if field == "accounts" { limits.core.accounts.accounts } else { limits.core.templates };
            if field == "accounts" {
                limits.clients.wire.me_model_accounts = value;
            } else {
                limits.clients.wire.me_project_defaults = value;
            }
            let mut problems = Problems::empty();
            check_settings(&limits, &mut problems);
            assert!(problems.is_empty(), "at bound: {problems:?}");
            if field == "accounts" {
                limits.clients.wire.me_model_accounts -= 1;
            } else {
                limits.clients.wire.me_project_defaults -= 1;
            }
            check_settings(&limits, &mut problems);
            assert_eq!(problems.problems.len(), 1);
            assert_eq!(
                problems.problems.get(0).unwrap(),
                &crate::Problem { path: Path::root().member(b"limits").member(name), what: crate::What::Bound }
            );
        }
    }
    #[test]
    fn policy_roles_fit_the_wire() {
        ceiling(b"authority.roles", 32);
    }
    #[test]
    fn policy_and_rule_requirements_fit_the_wire() {
        ceiling(b"authority.requirements", 32);
    }
    #[test]
    fn policy_projections_fit_the_wire() {
        ceiling(b"authority.grants", 32);
    }
    #[test]
    fn rule_implications_fit_the_wire() {
        ceiling(b"authority.implications", 32);
    }
    #[test]
    fn template_overrides_fit_the_wire() {
        ceiling(b"core.templates", 16);
    }
    #[test]
    fn policy_permissions_fit_the_wire() {
        ceiling(b"core.settings.permissions", 32);
    }
    #[test]
    fn charter_alternatives_fit_the_wire() {
        ceiling(b"core.settings.alternatives", 63);
    }
    #[test]
    fn charter_instructions_fit_the_wire() {
        ceiling(b"core.run_bytes", 1_114_240);
    }
    #[test]
    fn setting_names_fit_the_wire() {
        ceiling(b"documents.text_bytes", 65_536);
    }
    #[test]
    fn note_bodies_fit_the_wire() {
        ceiling(b"notes.body_bytes", 65_536);
    }
    #[test]
    fn revision_page_rows_fit_the_wire() {
        ceiling(b"documents.rows", 32);
    }
    #[test]
    fn revision_item_previews_fit_the_wire() {
        ceiling(b"documents.lines", 32);
    }
    #[test]
    fn inventory_counts_both_defaults_and_changes() {
        let mut limits = crate::profile(crate::Profile::Small);
        // Small has eleven other slots in each set and one shared 64-row model range.
        limits.core.tasks.charters = 85;
        let mut problems = Problems::empty();
        check_settings(&limits, &mut problems);
        assert!(problems.is_empty());
        limits.core.tasks.charters = 86;
        check_settings(&limits, &mut problems);
        assert_eq!(problems.problems.len(), 3);
    }
    #[test]
    fn profile_core_shares_remain_representable() {
        for profile in [crate::Profile::Small, crate::Profile::Standard] {
            let limits = crate::profile(profile);
            assert!(jig_domain_core_vocabulary::valid(&limits.core), "a profile's core limits are representable");
        }
    }
}

#[cfg(test)]
mod proposal_settings_tests {
    use super::*;
    #[test]
    fn proposal_heads_and_review_fit_their_wire_ceilings() {
        // settings-v1's proposal holds 128 heads; the review shares Owned.record's document ceiling.
        for (proposal, retained, review, expected) in [
            (128, 128, jig_client_protocol_wire::v22::CEILINGS.owned_record, None),
            (
                127,
                128,
                jig_client_protocol_wire::v22::CEILINGS.owned_record,
                Some(b"clients.wire.proposal_heads".as_slice()),
            ),
            (
                128,
                127,
                jig_client_protocol_wire::v22::CEILINGS.owned_record,
                Some(b"clients.wire.proposal_heads".as_slice()),
            ),
            (
                128,
                128,
                jig_client_protocol_wire::v22::CEILINGS.owned_record - 1,
                Some(b"clients.wire.settings_review".as_slice()),
            ),
        ] {
            let mut limits = crate::profile(crate::Profile::Small);
            limits.clients.wire.proposal_action_settings_items = proposal;
            limits.clients.wire.retained_action_settings_items = retained;
            limits.clients.wire.settings_review_items = review;
            let mut problems = Problems::empty();
            check_settings(&limits, &mut problems);
            match expected {
                None => assert!(problems.is_empty(), "{problems:?}"),
                Some(name) => {
                    assert_eq!(problems.problems.len(), 1);
                    assert_eq!(
                        problems.problems.get(0).expect("one problem").path,
                        Path::root().member(b"limits").member(name)
                    );
                }
            }
        }
    }
}

// PublicLine's bound is 128 bytes until public-wire supplies its encoded bound
// (jig/docs/design/domain/people/public-reading.md, 9).
const PUBLIC_LINE_BYTES: u64 = 128;

/// Public admission relations checked at start (protocol/limits.md).
pub(crate) fn check_public(limits: &crate::Limits, problems: &mut Problems) {
    let public = &limits.clients.public;
    if public.max_age > 60 {
        problems.push(crate::domain::codec_path(b"clients.public.max_age"), crate::What::Bound);
    }
    if !public.on() {
        return;
    }
    for (name, holds) in [
        (b"clients.public.rate_burst".as_slice(), public.rate_burst >= public.rate),
        (b"clients.public.bytes_burst".as_slice(), public.bytes_burst >= public.bytes),
        (b"clients.public.answer_bytes".as_slice(), u64::from(public.answer_bytes) <= public.bytes_burst),
        (b"clients_http.public_calls".as_slice(), limits.clients_http.public_calls <= public.reads),
        (b"clients.public.reads".as_slice(), public.reads > 0),
        (b"core.public.goals".as_slice(), limits.core.public.goals > 0),
        (b"core.public.ended".as_slice(), limits.core.public.ended > 0),
        (b"core.public.lines".as_slice(), limits.core.public.lines > 0),
        (b"core.public.depth".as_slice(), limits.core.public.depth > 0),
    ] {
        if !holds {
            problems.push(crate::domain::codec_path(name), crate::What::Bound);
        }
    }
    // Configuration has no per-process estimate; the index must fit u32 bytes.
    if match public_index_bytes(&limits.core) {
        Some(bytes) => bytes > u64::from(u32::MAX),
        None => true,
    } {
        problems.push(crate::domain::codec_path(b"core.public.lines"), crate::What::Bound);
    }
}

fn public_index_bytes(limits: &jig_domain_core_vocabulary::Limits) -> Option<u64> {
    u64::from(limits.public.goals)
        .checked_add(u64::from(limits.public.ended))?
        .checked_mul(u64::from(limits.public.lines))?
        .checked_mul(PUBLIC_LINE_BYTES)?
        .checked_mul(u64::from(limits.people.projects))
}

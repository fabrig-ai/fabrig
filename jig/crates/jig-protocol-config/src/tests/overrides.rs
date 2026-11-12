use super::fixtures::{document, path};
use crate::{Limits, Problems, Profile, What, overrides, profile};

fn small() -> Limits {
    profile(Profile::Small)
}
fn parse(bytes: &[u8]) -> Result<Limits, Problems> {
    let parsed = overrides::parse(&document(bytes), 0, &path(&[b"limits"]));
    if parsed.problems.is_empty() { Ok(parsed.limits) } else { Err(parsed.problems) }
}
fn problem(bytes: &[u8], segments: &[&[u8]], what: What) {
    let errors = parse(bytes).unwrap_err();
    assert_eq!(errors.problems.len(), 1);
    assert_eq!(errors.problems.get(0).unwrap().path, path(segments));
    assert_eq!(errors.problems.get(0).unwrap().what, what);
}

#[test]
fn an_override_replaces_one_limit_of_the_profile() {
    for (name, value, json) in [
        ("tasks.inbox_messages", 9, br#"{"profile":"small","overrides":{"tasks.inbox_messages":9}}"#.as_slice()),
        ("notes.party_entries", 11, br#"{"profile":"small","overrides":{"notes.party_entries":11}}"#.as_slice()),
        ("notes.party_bytes", 119, br#"{"profile":"small","overrides":{"notes.party_bytes":119}}"#.as_slice()),
    ] {
        let mut expected = small();
        match name {
            "tasks.inbox_messages" => expected.core.tasks.inbox_messages = value,
            "notes.party_entries" => expected.core.notes.party_entries = value,
            "notes.party_bytes" => expected.core.notes.party_bytes = value,
            _ => unreachable!("table names"),
        }
        assert_eq!(parse(json), Ok(expected));
    }
}

#[test]
fn io_output_overrides_only_a_positive_representable_room() {
    for (json, output) in [
        (br#"{"profile":"small","overrides":{"io.output":1}}"#.as_slice(), Some(1)),
        (br#"{"profile":"small","overrides":{"io.output":4294967295}}"#, Some(u32::MAX)),
        (br#"{"profile":"small","overrides":{"io.output":0}}"#, None),
        (br#"{"profile":"small","overrides":{"io.output":4294967296}}"#, None),
    ] {
        if let Some(output) = output {
            let mut expected = small();
            expected.io.output = output;
            assert_eq!(parse(json), Ok(expected));
        } else {
            problem(json, &[b"limits", b"overrides", b"io.output"], What::Override);
        }
    }
}

#[test]
fn the_charter_documents_limit_can_be_overridden() {
    let mut expected = small();
    expected.core.tasks.charter_documents = 0;
    assert_eq!(parse(br#"{"profile":"small","overrides":{"tasks.charter_documents":0}}"#), Ok(expected));
}
fn an_override_of_an_unknown_name_is_refused() {
    assert_eq!(overrides::apply(&mut small(), b"tasks.absent", 1), Err(What::Override));
}
fn an_override_past_its_type_is_refused() {
    assert_eq!(overrides::apply(&mut small(), b"hub.slots", 4_294_967_296), Err(What::Override));
}
fn an_override_that_breaks_a_bound_is_refused_naming_it() {
    problem(
        br#"{"profile":"small","overrides":{"tasks.tasks":0}}"#,
        &[b"limits", b"overrides", b"tasks.tasks"],
        What::Override,
    );
}
#[test]
fn overrides_past_a_familys_bound_are_each_reported_once() {
    let errors =
        parse(br#"{"profile":"small","overrides":{"tasks.tasks":0,"tasks.inbox_messages":0,"tasks.run_inbox":0}}"#)
            .unwrap_err();
    assert_eq!(errors.problems.len(), 3);
    assert_eq!(errors.past, 0);
    for (index, name) in
        [b"tasks.tasks".as_slice(), b"tasks.inbox_messages", b"tasks.run_inbox"].into_iter().enumerate()
    {
        let found = errors.problems.get(u32::try_from(index).unwrap()).unwrap();
        assert_eq!(found.what, What::Override);
        assert_eq!(found.path, path(&[b"limits", b"overrides", name]));
    }
}

#[test]
fn every_limit_of_the_small_profile_has_a_name() {
    for row in SMALL {
        let name = row.name.as_bytes();
        let value = (row.value)(&small());
        let mut limits = small();
        assert_eq!(overrides::apply(&mut limits, name, value), Ok(()), "{name:?}");
        assert_eq!(limits, small(), "{name:?}");
    }
}
#[test]
fn duration_overrides_read_as_milliseconds() {
    let mut expected = small();
    expected.link.ping = skein_lib::Duration::from_millis(6_000);
    assert_eq!(parse(br#"{"profile":"small","overrides":{"link.ping_ms":6000}}"#), Ok(expected));
}
fn a_duration_override_past_its_type_is_refused() {
    assert_eq!(overrides::apply(&mut small(), b"link.ping_ms", u64::MAX), Err(What::Override));
}
#[test]
fn a_limit_with_a_u64_type_keeps_its_width() {
    let mut limits = small();
    assert_eq!(overrides::apply(&mut limits, b"core.fill_mark", u64::MAX), Ok(()));
    assert_eq!(limits.core.fill_mark, u64::MAX);
}
fn an_unknown_profile_is_refused_naming_it() {
    problem(br#"{"profile":"other"}"#, &[b"limits", b"profile"], What::Override);
}
#[test]
fn the_standard_profile_can_be_selected() {
    assert_eq!(parse(br#"{"profile":"standard"}"#), Ok(profile(Profile::Standard)));
}

#[test]
fn limits_problems_keep_the_named_profile_and_applied_overrides_for_connectors() {
    use super::fixtures::{ENGINE_SMALL, engine, with};

    let bytes = with(ENGINE_SMALL, br#""connectors": [1]"#, br#""connectors": [1, 2, 3]"#);
    for limits in [
        br#"{"profile":"standard","overrides":{"bogus":1}}"#.as_slice(),
        br#"{"profile":"small","overrides":{"core.connectors":3,"bogus":1}}"#.as_slice(),
    ] {
        let bytes = with(&bytes, br#"{"profile": "small"}"#, limits);
        let problems = engine(&bytes).unwrap_err();
        assert_eq!(problems.problems.len(), 1, "{limits:?}: {problems:?}");
        assert_eq!(problems.past, 0);
        assert_eq!(problems.problems.get(0).unwrap().path, path(&[b"limits", b"overrides", b"bogus"]));
        assert_eq!(problems.problems.get(0).unwrap().what, What::Override);
    }
}
#[test]
fn limits_problems_without_a_profile_add_no_bound_of_small() {
    use super::fixtures::{ENGINE_SMALL, engine, with};

    let bytes = with(ENGINE_SMALL, br#""connectors": [1]"#, br#""connectors": [1, 2, 3]"#);
    for (case, limits, expected) in [
        ("unknown profile", Some(br#"{"profile":"standrd"}"#.as_slice()), path(&[b"limits", b"profile"])),
        ("profile of the wrong kind", Some(br#"{"profile":7}"#.as_slice()), path(&[b"limits", b"profile"])),
        ("missing profile", Some(b"{}".as_slice()), path(&[b"limits", b"profile"])),
        ("limits of the wrong kind", Some(b"7".as_slice()), path(&[b"limits"])),
        ("missing limits", None, path(&[b"limits"])),
    ] {
        let bytes = match limits {
            Some(limits) => with(&bytes, br#"{"profile": "small"}"#, limits),
            None => with(&bytes, br#""limits": {"profile": "small"},"#, b""),
        };
        let problems = engine(&bytes).unwrap_err();
        assert_eq!(problems.problems.len(), 1, "{case}: {problems:?}");
        assert_eq!(problems.past, 0, "{case}: {problems:?}");
        assert_eq!(problems.problems.get(0).unwrap().path, expected, "{case}: {problems:?}");
    }
}
fn a_missing_profile_is_refused_naming_it() {
    problem(b"{}", &[b"limits", b"profile"], What::Missing);
}
fn an_unknown_limits_member_is_refused_naming_it() {
    problem(br#"{"profile":"small","absent":1}"#, &[b"limits", b"absent"], What::Unknown);
}
fn an_override_named_twice_is_refused_naming_it() {
    problem(
        br#"{"profile":"small","overrides":{"hub.slots":2,"hub.slots":3}}"#,
        &[b"limits", b"overrides", b"hub.slots"],
        What::Duplicate,
    );
}
#[test]
fn overrides_are_checked_together() {
    let mut expected = small();
    expected.core.tasks.run_inbox = 1;
    expected.core.tasks.inbox_messages = 1;
    expected.core.people.inbox_entries = 1;
    assert_eq!(
        parse(br#"{"profile":"small","overrides":{"tasks.inbox_messages":1,"tasks.run_inbox":1,"people.inbox_entries":1}}"#),
        Ok(expected)
    );
}

#[test]
fn stream_overrides_keep_four_exchanges_beside_every_stream() {
    let limits = parse(br#"{"profile":"small","overrides":{"clients.streams":8,"clients_http.streams":8,"clients_http.calls":12,"clients.exchanges":12}}"#).unwrap();
    assert_eq!((limits.clients.streams, limits.clients_http.streams), (8, 8));
    assert_eq!((limits.clients.exchanges, limits.clients_http.calls), (12, 12));
    for (name, bytes) in [
        (b"clients.exchanges".as_slice(), br#"{"profile":"small","overrides":{"clients.streams":8,"clients_http.streams":8,"clients_http.calls":12,"clients.exchanges":11}}"#.as_slice()),
        (b"clients_http.calls", br#"{"profile":"small","overrides":{"clients.streams":8,"clients_http.streams":8,"clients_http.calls":11,"clients.exchanges":12}}"#),
        (b"clients.exchanges", br#"{"profile":"small","overrides":{"clients.streams":8,"clients_http.streams":8,"clients_http.calls":13,"clients.exchanges":12}}"#),
    ] {
        problem(bytes, &[b"limits", b"overrides", name], What::Override);
    }
}

#[test]
fn stream_and_carrier_overrides_are_positive_representable_counts() {
    for name in [b"clients.streams".as_slice(), b"clients.exchanges", b"clients_http.streams", b"clients_http.calls"] {
        for value in [0, u64::from(u32::MAX) + 1] {
            assert_eq!(overrides::apply(&mut small(), name, value), Err(What::Override));
        }
    }
    problem(
        br#"{"profile":"small","overrides":{"clients.exchanges":4294967295}}"#,
        &[b"limits", b"overrides", b"clients.exchanges"],
        What::Override,
    );
}
fn a_non_integer_override_is_refused_naming_it() {
    problem(
        br#"{"profile":"small","overrides":{"hub.slots":{}}}"#,
        &[b"limits", b"overrides", b"hub.slots"],
        What::Override,
    );
}

fn a_whole_compound_limit_is_refused() {
    assert_eq!(overrides::apply(&mut small(), b"tasks.retries", 1), Err(What::Override));
}
#[test]
fn a_compound_member_override_replaces_only_that_member() {
    let mut expected = small();
    expected.core.tasks.retries.transient.retries = 4;
    assert_eq!(parse(br#"{"profile":"small","overrides":{"tasks.retries.transient.retries":4}}"#), Ok(expected));
}
fn an_invalid_compound_member_is_refused_naming_it() {
    problem(
        br#"{"profile":"small","overrides":{"documents.versions.minimum":2}}"#,
        &[b"limits", b"overrides", b"documents.versions.minimum"],
        What::Override,
    );
}

pub(super) const SMALL: &[crate::names::Name] = &crate::names::NAMES;

fn overrides_past_their_types_are_refused() {
    table([
        ("an_override_past_its_type_is_refused", an_override_past_its_type_is_refused),
        ("a_duration_override_past_its_type_is_refused", a_duration_override_past_its_type_is_refused),
    ]);
}

type Case = (&'static str, fn());

fn table<const N: usize>(cases: [Case; N]) {
    let mut failures = [None; N];
    let mut passed = true;
    for ((name, case), failure) in cases.into_iter().zip(&mut failures) {
        if std::panic::catch_unwind(case).is_err() {
            *failure = Some(name);
            passed = false;
        }
    }
    assert!(passed, "{failures:?}");
}

#[test]
fn override_document_problems_name_the_invalid_fields() {
    table([
        ("an_override_that_breaks_a_bound_is_refused_naming_it", an_override_that_breaks_a_bound_is_refused_naming_it),
        ("an_unknown_profile_is_refused_naming_it", an_unknown_profile_is_refused_naming_it),
        ("a_missing_profile_is_refused_naming_it", a_missing_profile_is_refused_naming_it),
        ("an_unknown_limits_member_is_refused_naming_it", an_unknown_limits_member_is_refused_naming_it),
        ("an_override_named_twice_is_refused_naming_it", an_override_named_twice_is_refused_naming_it),
        ("a_non_integer_override_is_refused_naming_it", a_non_integer_override_is_refused_naming_it),
        ("an_invalid_compound_member_is_refused_naming_it", an_invalid_compound_member_is_refused_naming_it),
    ]);
}

#[test]
fn sign_ins_at_once_and_the_sign_in_deadline_reach_the_driver() {
    let limits =
        parse(br#"{"profile":"small","overrides":{"accounts.sign_ins":3,"accounts.sign_in_deadline_ms":120000}}"#)
            .unwrap();
    assert_eq!((limits.core.accounts.sign_ins, limits.driver.listeners), (3, 6));
    assert_eq!(limits.driver.client.sign_in_time, skein_lib::Duration::from_secs(120));
}
fn no_sign_ins_at_once_is_refused() {
    refused(br#"{"profile":"small","overrides":{"accounts.sign_ins":0}}"#, b"accounts.sign_ins");
}
fn sign_ins_past_the_drivers_sockets_are_refused() {
    refused(br#"{"profile":"small","overrides":{"accounts.sign_ins":7}}"#, b"accounts.sign_ins");
}
fn a_holders_accounts_past_the_deployments_are_refused() {
    refused(br#"{"profile":"small","overrides":{"accounts.holder_accounts":5}}"#, b"accounts.holder_accounts");
}
fn no_sources_per_account_is_refused() {
    refused(br#"{"profile":"small","overrides":{"accounts.sources":0}}"#, b"accounts.sources");
}
fn a_sign_in_deadline_of_zero_is_refused() {
    refused(br#"{"profile":"small","overrides":{"accounts.sign_in_deadline_ms":0}}"#, b"accounts.sign_in_deadline_ms");
}
/// The small profile's override in `bytes`, of `name`, is refused naming it.
fn refused(bytes: &[u8], name: &[u8]) {
    problem(bytes, &[b"limits", b"overrides", name], What::Override);
}
#[test]
fn accounts_bounds_past_their_limits_are_refused_naming_them() {
    table([
        ("no_sign_ins_at_once_is_refused", no_sign_ins_at_once_is_refused),
        ("sign_ins_past_the_drivers_sockets_are_refused", sign_ins_past_the_drivers_sockets_are_refused),
        ("a_holders_accounts_past_the_deployments_are_refused", a_holders_accounts_past_the_deployments_are_refused),
        ("no_sources_per_account_is_refused", no_sources_per_account_is_refused),
        ("a_sign_in_deadline_of_zero_is_refused", a_sign_in_deadline_of_zero_is_refused),
    ]);
}

#[test]
fn invalid_override_names_and_values_are_refused() {
    table([
        ("an_override_of_an_unknown_name_is_refused", an_override_of_an_unknown_name_is_refused),
        ("a_whole_compound_limit_is_refused", a_whole_compound_limit_is_refused),
        ("overrides_past_their_types_are_refused", overrides_past_their_types_are_refused),
    ]);
}

#[test]
fn document_segments_follow_the_largest_admitted_name_after_overrides() {
    const CASES: [(&[u8], u32); 3] = [
        (br#"{"profile":"small","overrides":{"tasks.hold_segments":6}}"#, 6),
        (br#"{"profile":"small","overrides":{"tasks.authority_segments":7}}"#, 7),
        (br#"{"profile":"small","overrides":{"authority.segments":8}}"#, 8),
    ];
    for (configuration, segments) in CASES {
        let limits = parse(configuration).expect("admitted name bounds");
        assert_eq!(limits.core.documents.segments, segments, "{configuration:?}");
        assert_eq!(limits.core.documents.lines, 4, "short lists stay bounded independently");
    }
}

#[test]
fn document_segments_have_no_operator_override() {
    assert_eq!(overrides::apply(&mut small(), b"documents.segments", 8), Err(What::Override));
    assert_eq!(overrides::apply(&mut small(), b"documents.grants", 8), Err(What::Override));
    assert_eq!(overrides::apply(&mut small(), b"documents.executors", 8), Err(What::Override));
    assert_eq!(overrides::apply(&mut small(), b"documents.implications", 8), Err(What::Override));
}

#[test]
fn the_accounts_move_deadline_is_positive_and_overridden_in_milliseconds() {
    for profile in [Profile::Small, Profile::Standard] {
        assert_eq!(crate::profile(profile).core.accounts.move_deadline, skein_lib::Duration::from_secs(60));
    }
    let mut expected = small();
    expected.core.accounts.move_deadline = skein_lib::Duration::from_millis(12_345);
    assert_eq!(parse(br#"{"profile":"small","overrides":{"accounts.move_deadline_ms":12345}}"#), Ok(expected));
    refused(br#"{"profile":"small","overrides":{"accounts.move_deadline_ms":0}}"#, b"accounts.move_deadline_ms");
}

#[test]
fn the_tail_has_its_own_profile_budget_and_override() {
    let mut expected = small();
    assert_eq!(expected.core.brief_core_budgets.tail, 2048);
    expected.core.brief_core_budgets.tail = 90;
    assert_eq!(parse(br#"{"profile":"small","overrides":{"core.brief_core_budgets.tail":90}}"#), Ok(expected));
    assert!(jig_domain_core::valid(&expected.core));
}

#[test]
fn a_standing_now_has_its_own_profile_budget_and_override() {
    for selected in [Profile::Small, Profile::Standard] {
        assert_eq!(profile(selected).core.brief_core_budgets.now, 4096);
    }
    let mut expected = small();
    expected.core.brief_core_budgets.now = 90;
    assert_eq!(parse(br#"{"profile":"small","overrides":{"core.brief_core_budgets.now":90}}"#), Ok(expected));
    assert!(jig_domain_core::valid(&expected.core));
    refused(
        br#"{"profile":"small","overrides":{"core.brief_core_budgets.now":4294967296}}"#,
        b"core.brief_core_budgets.now",
    );
}

#[test]
fn settings_history_is_positive_and_can_override_the_profile() {
    assert_eq!(small().core.settings.history, 64);
    assert_eq!(
        parse(br#"{"profile":"small","overrides":{"core.settings.history":2}}"#).unwrap().core.settings.history,
        2
    );
    problem(
        br#"{"profile":"small","overrides":{"core.settings.history":0}}"#,
        &[b"limits", b"overrides", b"core.settings.history"],
        What::Override,
    );
}

#[test]
fn a_journal_too_small_for_settings_is_refused_before_start() {
    let mut engine = super::fixtures::engine(super::fixtures::ENGINE_SMALL).unwrap();
    engine.limits.journal.writes = 1;
    let mut problems = Problems::empty();
    crate::domain::whole(&engine, &mut problems);
    let mut found = false;
    for problem in problems.problems.as_slice() {
        found |= problem.path == path(&[b"limits", b"journal.writes"]) && problem.what == What::Bound;
    }
    assert!(found, "the refusal names the journal write limit");
}

#[test]
fn a_policy_override_past_the_codec_envelope_is_refused_at_its_path() {
    let errors = parse(br#"{"profile":"small","overrides":{"core.policy_bytes":65536}}"#).unwrap_err();
    assert_eq!(errors.problems.len(), 1);
    let problem = errors.problems.get(0).unwrap();
    assert_eq!(problem.path, path(&[b"limits", b"overrides", b"core.policy_bytes"]));
    assert_eq!(problem.codec_message().unwrap().as_ref(), b"limits.overrides.core.policy_bytes: 65536 exceeds the people-v13 codec envelope of 43695 bytes (PolicyValue.permissions would need 98304 bytes; ceiling 65536).");
}

#[test]
fn whole_admission_checks_the_limits_handed_to_the_store() {
    let mut engine = super::fixtures::engine(super::fixtures::ENGINE_SMALL).unwrap();
    engine.limits.core.policy_bytes = 65_536;
    let mut problems = Problems::empty();
    crate::domain::whole(&engine, &mut problems);
    let problem = problems.problems.get(0).unwrap();
    assert_eq!(problem.path, path(&[b"limits", b"core", b"policy_bytes"]));
    assert_eq!(problem.codec_message().unwrap().as_ref(), b"limits.core.policy_bytes: 65536 exceeds the people-v13 codec envelope of 43695 bytes (PolicyValue.permissions would need 98304 bytes; ceiling 65536).");
}

#[test]
fn codec_admission_keeps_a_composed_documents_override_prefix() {
    let document = document(br#"{"profile":"small","overrides":{"core.policy_bytes":43696}}"#);
    let prefix = path(&[b"engine", b"limits"]);
    let problems = overrides::parse(&document, 0, &prefix).problems;
    assert_eq!(problems.problems.get(0).unwrap().path, prefix.member(b"overrides").member(b"core.policy_bytes"));
}

#[test]
fn pending_settings_proposals_are_positive_and_keep_the_profile_defaults() {
    assert_eq!(small().core.settings.proposals, 4);
    assert_eq!(profile(Profile::Standard).core.settings.proposals, 8);
    assert_eq!(
        parse(br#"{"profile":"small","overrides":{"core.settings.proposals":2}}"#).unwrap().core.settings.proposals,
        2
    );
    problem(
        br#"{"profile":"small","overrides":{"core.settings.proposals":0}}"#,
        &[b"limits", b"overrides", b"core.settings.proposals"],
        What::Override,
    );
}
#[test]
fn the_journal_prices_each_pending_settings_withdrawal() {
    let mut limits = small().core;
    limits.settings.proposals = 1;
    let one = jig_domain_core::settings::writes_bound(&limits).expect("write room");
    limits.settings.proposals = 2;
    let two = jig_domain_core::settings::writes_bound(&limits).expect("write room");
    // Task withdrawal: set erase, terminal history, two activity rows and up to three live rows.
    assert_eq!(two - one, 7);
}

#[test]
fn settings_revision_limits_are_positive_and_size_the_children() {
    for (profile_name, revisions, versions) in [(Profile::Small, 4, 8), (Profile::Standard, 8, 16)] {
        let limits = profile(profile_name).core;
        assert_eq!(limits.settings.revisions, revisions);
        assert_eq!(limits.settings.versions, versions);
    }
    let changed =
        parse(br#"{"profile":"small","overrides":{"core.settings.revisions":2,"core.settings.versions":3}}"#).unwrap();
    assert_eq!(changed.core.settings.revisions, 2);
    assert_eq!(changed.core.settings.versions, 3);
    for (document, name) in [
        (
            br#"{"profile":"small","overrides":{"core.settings.revisions":0}}"#.as_slice(),
            b"core.settings.revisions".as_slice(),
        ),
        (
            br#"{"profile":"small","overrides":{"core.settings.versions":0}}"#.as_slice(),
            b"core.settings.versions".as_slice(),
        ),
    ] {
        problem(document, &[b"limits", b"overrides", name], What::Override);
    }
    for revisions in [true, false] {
        let mut engine = super::fixtures::engine(super::fixtures::ENGINE_SMALL).unwrap();
        let name = if revisions {
            engine.limits.core.settings.revisions = 0;
            b"settings.revisions".as_slice()
        } else {
            engine.limits.core.settings.versions = 0;
            b"settings.versions".as_slice()
        };
        let mut problems = Problems::empty();
        crate::domain::whole(&engine, &mut problems);
        let mut found = false;
        for problem in problems.problems.as_slice() {
            if problem.path == path(&[b"limits", b"core", name]) && problem.what == What::Bound {
                found = true;
            }
        }
        assert!(found);
    }
}

#[test]
fn placed_hold_overrides_reject_zero_and_exemptions_past_the_live_set() {
    for (bytes, key) in [
        (br#"{"profile":"small","overrides":{"tasks.placed_holds":0}}"#.as_slice(), b"tasks.placed_holds".as_slice()),
        (
            br#"{"profile":"small","overrides":{"tasks.project_placed_holds":0}}"#.as_slice(),
            b"tasks.project_placed_holds".as_slice(),
        ),
        (
            br#"{"profile":"small","overrides":{"tasks.party_placed_holds":0}}"#.as_slice(),
            b"tasks.party_placed_holds".as_slice(),
        ),
        (br#"{"profile":"small","overrides":{"tasks.exemptions":0}}"#.as_slice(), b"tasks.exemptions".as_slice()),
    ] {
        problem(bytes, &[b"limits", b"overrides", key], What::Override);
    }
    problem(
        br#"{"profile":"small","overrides":{"tasks.exemptions":9}}"#,
        &[b"limits", b"overrides", b"tasks.exemptions"],
        What::Capacity { member: b"tasks.exemptions", configured: 9, requires: b"tasks.tasks", required: 9 },
    );
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "each capacity dependency retains its override document and named diagnostic"
)]
fn capacity_dependencies_survive_override_parsing() {
    for (bytes, member, configured, dependency, minimum) in [
        (
            br#"{"profile":"small","overrides":{"documents.lines":3,"tasks.tasks":64}}"#.as_slice(),
            b"tasks.tasks".as_slice(),
            64,
            b"people.inbox_tasks".as_slice(),
            128,
        ),
        (
            br#"{"profile":"small","overrides":{"documents.lines":3,"tasks.delegates":8}}"#,
            b"tasks.delegates",
            8,
            b"people.operations",
            9,
        ),
        (
            br#"{"profile":"small","overrides":{"documents.lines":3,"tasks.delegates":16}}"#,
            b"tasks.delegates",
            16,
            b"people.operations",
            17,
        ),
        (
            br#"{"profile":"small","overrides":{"tasks.project_goals":9}}"#,
            b"tasks.project_goals",
            9,
            b"people.operations",
            9,
        ),
        (
            br#"{"profile":"small","overrides":{"documents.lines":3,"tasks.message_bytes":8192}}"#,
            b"tasks.message_bytes",
            8192,
            b"tasks.inbox_bytes",
            8192,
        ),
        (
            br#"{"profile":"small","overrides":{"documents.lines":3,"authority.grants":6}}"#,
            b"authority.grants",
            6,
            b"notes.scopes",
            9,
        ),
        (
            br#"{"profile":"small","overrides":{"documents.lines":3,"tasks.tasks":4}}"#,
            b"people.operations",
            8,
            b"tasks.tasks",
            8,
        ),
        (
            br#"{"profile":"small","overrides":{"documents.lines":3,"tasks.run_inbox":9}}"#,
            b"tasks.run_inbox",
            9,
            b"tasks.inbox_messages",
            9,
        ),
        (
            br#"{"profile":"small","overrides":{"documents.lines":3,"people.inbox_entries":9}}"#,
            b"people.inbox_entries",
            9,
            b"tasks.inbox_messages",
            9,
        ),
        (
            br#"{"profile":"small","overrides":{"documents.lines":3,"tasks.exemptions":9}}"#,
            b"tasks.exemptions",
            9,
            b"tasks.tasks",
            9,
        ),
        (
            br#"{"profile":"small","overrides":{"documents.lines":3,"fleet.configured_workers":9}}"#,
            b"fleet.configured_workers",
            9,
            b"people.program_hosts",
            10,
        ),
        (
            br#"{"profile":"small","overrides":{"documents.lines":3,"accounts.program_hosts":8}}"#,
            b"fleet.configured_workers",
            8,
            b"accounts.program_hosts",
            9,
        ),
        (
            br#"{"profile":"small","overrides":{"documents.lines":3,"fleet.worker_name_bytes":33}}"#,
            b"fleet.worker_name_bytes",
            33,
            b"people.program_host_name_bytes",
            33,
        ),
        (
            br#"{"profile":"small","overrides":{"documents.lines":3,"accounts.program_names":13}}"#,
            b"accounts.program_names",
            13,
            b"fleet.program_sources",
            13,
        ),
        (
            br#"{"profile":"small","overrides":{"documents.lines":3,"tasks.inbox_messages":13}}"#,
            b"tasks.inbox_messages",
            13,
            b"people.inbox_recipients",
            34,
        ),
        (
            br#"{"profile":"small","overrides":{"documents.lines":3,"accounts.credentials":5}}"#,
            b"accounts.credentials",
            5,
            b"fleet.grants",
            9,
        ),
        (
            br#"{"profile":"small","overrides":{"documents.lines":3,"authority.accounts":3}}"#,
            b"accounts.credentials",
            3,
            b"authority.accounts",
            4,
        ),
        (
            br#"{"profile":"small","overrides":{"documents.lines":3,"accounts.accounts":65}}"#,
            b"accounts.credentials",
            3,
            b"fleet.grants",
            67,
        ),
        (
            br#"{"profile":"small","overrides":{"documents.lines":3,"accounts.accounts":65,"fleet.grants":67,"authority.accounts":63}}"#,
            b"accounts.credentials",
            3,
            b"authority.accounts",
            64,
        ),
        (
            br#"{"profile":"small","overrides":{"documents.lines":3,"accounts.credentials":9,"fleet.grants":13,"authority.accounts":8}}"#,
            b"accounts.credentials",
            9,
            b"authority.accounts",
            9,
        ),
    ] {
        problem(
            bytes,
            &[b"limits", b"overrides", member],
            What::Capacity { member, configured, requires: dependency, required: minimum },
        );
    }
    parse(br#"{"profile":"small","overrides":{"tasks.project_tasks":64}}"#).expect("project capacity is independent");
    parse(br#"{"profile":"small","overrides":{"tasks.tasks":64,"people.inbox_tasks":128}}"#)
        .expect("sufficient inbox capacity");
}

#[test]
fn batch_past_the_schema_keeps_its_binding_count() {
    problem(
        br#"{"profile":"small","overrides":{"documents.lines":3,"tasks.batch":1025}}"#,
        &[b"limits", b"overrides", b"tasks.batch"],
        What::CodecCeiling {
            bound: crate::CodecBound {
                field: b"tasks-v9.TaskInterests.result_followups",
                needed: 1025,
                ceiling: jig_protocol_store::tasks_v11::CEILINGS.task_interests_result_followups,
            },
        },
    );
}

#[test]
fn a_broken_relation_is_the_only_problem() {
    problem(
        br#"{"profile":"small","overrides":{"documents.lines":3,"tasks.charters":8,"authority.grants":6,"tasks.message_bytes":8192}}"#,
        &[b"limits", b"overrides", b"authority.grants"],
        What::Capacity { member: b"authority.grants", configured: 6, requires: b"notes.scopes", required: 9 },
    );
}

#[test]
fn zero_capacity_sides_keep_their_family_bound_problem() {
    for (bytes, member, what) in [
        (
            br#"{"profile":"small","overrides":{"tasks.run_inbox":0}}"#.as_slice(),
            b"tasks.run_inbox".as_slice(),
            What::Override,
        ),
        (br#"{"profile":"small","overrides":{"tasks.inbox_bytes":0}}"#, b"tasks.inbox_bytes", What::Override),
        (br#"{"profile":"small","overrides":{"tasks.inbox_messages":0}}"#, b"tasks.inbox_messages", What::Override),
        (
            br#"{"profile":"small","overrides":{"people.operations":0}}"#,
            b"tasks.delegates",
            What::Capacity { member: b"tasks.delegates", configured: 4, requires: b"people.operations", required: 5 },
        ),
    ] {
        problem(bytes, &[b"limits", b"overrides", member], what);
    }
}

#[test]
fn infrastructure_retry_members_can_each_be_overridden() {
    let json = br#"{"profile":"small","overrides":{"tasks.retries.infrastructure.retries":7,"tasks.retries.infrastructure.base_ms":2000,"tasks.retries.infrastructure.max_ms":9000}}"#;
    let mut expected = small();
    expected.core.tasks.retries.infrastructure = jig_domain_tasks::Retry {
        retries: 7, base: skein_lib::Duration::from_millis(2000), max: skein_lib::Duration::from_millis(9000),
    };
    assert_eq!(parse(json), Ok(expected));
}

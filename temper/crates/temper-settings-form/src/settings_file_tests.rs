//! Charter settings in the file's forms (temper/docs/design/client/tools.md, 5.3).
//! Retained charters follow temper/docs/design/protocol/configuration.md, 13.7.
use crate::{Limits, file, values, view as c};
use alloc::boxed::Box;
use skein_json::writer::{Encoder, Limits as JsonLimits};
use skein_lib::{Duration, Writer};

// C7's coordinator paces news at 30 minutes, in the settings file's nanoseconds.
const COORDINATOR: &[u8] = br#", "wake":{"words":"immediate","notices":"immediate","news":{"paced":1800000000000},"results":"each","questions":true,"answers":true,"timers":true}"#;
const PREFIX: &[u8] = br#"{"items":[{"setting":{"kind":"charter","number":5},"set":{"number":5,"instructions":"coordinate","waiting":1,"resume":true,"turns":2,"time":2,"reserve_turns":1,"reserve_time":1,"model":{"dialect":1,"endpoint":1,"program":null,"name":"model","window":8192,"output":1024,"input_price":1,"cached_price":1,"cache_write_price":null,"output_price":1,"price_unit":1000,"efforts":[],"effort":null},"alternatives":[],"inspect":true,"modify":false,"shell":false,"agents":false,"call_timeout":1,"hosts":"both","spend":100,"documents":[]"#;
const SUFFIX: &[u8] = b"}}]}";
#[test]
#[expect(clippy::disallowed_macros, reason = "decode the retained process fixture without filesystem access")]
fn the_retained_cluster_settings_file_decodes_with_no_charter_documents() {
    // Two charters, the chat template and the deployment landing rule.
    const ITEMS: u32 = 4;
    let bytes = include_bytes!("../../temper/tests/fixtures/cluster/settings.json");
    let items = file::decode(bytes, &Limits::standard()).expect("retained cluster settings file");
    assert_eq!(items.len(), ITEMS);
    for (index, number) in [(0, 1), (1, 2)] {
        let item = items.get(index).expect("cluster charter item");
        assert_eq!(item.setting, c::Setting::Charter(c::SettingNumber { number }));
        let c::SettingEdit::Set(set) = &item.edit else { panic!("whole charter value") };
        let c::SettingValue::Charter(charter) = &set.value else { panic!("charter") };
        assert!(charter.documents.is_empty());
    }
}
fn charter_file(wake: &[u8]) -> Box<[u8]> {
    let length = PREFIX.len().checked_add(wake.len()).expect("fixture wake").checked_add(SUFFIX.len()).expect("suffix");
    let mut out = Writer::new(length);
    out.put(PREFIX).unwrap();
    out.put(wake).unwrap();
    out.put(SUFFIX).unwrap();
    out.finish()
}
fn charter(bytes: &[u8]) -> c::Charter {
    let items = file::decode(bytes, &Limits::standard()).expect("charter file");
    let c::SettingEdit::Set(set) = &items.get(0).unwrap().edit else { panic!("set") };
    let c::SettingValue::Charter(value) = &set.value else { panic!("charter") };
    *value.clone()
}
fn coordinator() -> c::WakePolicy {
    c::WakePolicy {
        words: c::WakeRule::Immediate,
        notices: c::WakeRule::Immediate,
        news: c::WakeRule::Paced { span: Duration::from_secs(30 * 60) },
        results: c::ResultsWake::Each,
        questions: true,
        answers: true,
        timers: true,
    }
}
#[test]
fn a_settings_file_charter_names_its_wake_policy() {
    assert_eq!(charter(&charter_file(COORDINATOR)).wake, coordinator());
}
#[test]
fn a_settings_file_charter_without_wake_takes_the_default() {
    for wake in [b"".as_slice(), br#", "wake":null"#] {
        assert_eq!(charter(&charter_file(wake)).wake, c::WakePolicy::DEFAULT);
    }
}
#[test]
fn an_unknown_wake_rule_is_refused_naming_its_path() {
    let file = charter_file(br#", "wake":{"words":"immediate","notices":"immediate","news":"often","results":"each","questions":true,"answers":true,"timers":true}"#);
    assert_eq!(
        file::decode(&file, &Limits::standard()).unwrap_err().as_ref(),
        b"settings file item 1: charter.wake.news: unknown wake rule"
    );
}
#[test]
fn a_missing_or_unknown_wake_member_is_refused_naming_its_path() {
    for (wake, expected) in [
        (
            br#", "wake":{"words":"immediate","notices":"immediate"}"#.as_slice(),
            b"settings file item 1: charter.wake.news: missing member".as_slice(),
        ),
        (br#", "wake":{"other":true}"#, b"settings file item 1: charter.wake.other: unknown member"),
        (
            br#", "wake":{"words":"immediate","notices":"immediate","news":{"often":1}}"#,
            b"settings file item 1: charter.wake.news.often: unknown member",
        ),
        (
            br#", "wake":{"words":"immediate","notices":"immediate","news":{"paced":1,"batch":{"count":1,"age":1}}}"#,
            b"settings file item 1: charter.wake.news: one wake rule member required",
        ),
        (
            br#", "wake":{"words":"immediate","notices":"immediate","news":{"batch":{"count":1}}}"#,
            b"settings file item 1: charter.wake.news.batch.age: missing member",
        ),
        (
            br#", "wake":{"words":"immediate","notices":"immediate","news":{"batch":{"other":1}}}"#,
            b"settings file item 1: charter.wake.news.batch.other: unknown member",
        ),
    ] {
        assert_eq!(file::decode(&charter_file(wake), &Limits::standard()).unwrap_err().as_ref(), expected);
    }
}
#[test]
fn zero_wake_counts_and_durations_are_sent_for_deployment_validation() {
    let file = charter_file(br#", "wake":{"words":{"batch":{"count":0,"age":0}},"notices":"never","news":{"paced":0},"results":"never","questions":false,"answers":false,"timers":false}"#);
    let value = charter(&file);
    let wire = crate::wire::settings_wire_charter(&value, &Limits::standard().settings).unwrap();
    assert_eq!(
        wire.wake().words(),
        &jig_client_protocol_wire::WakeRule::Batch(
            jig_client_protocol_wire::WakeRuleBatch::new(
                &Limits::standard().settings,
                jig_client_protocol_wire::WakeRuleBatchParts { count: 0, age: Duration::ZERO }
            )
            .unwrap()
        )
    );
    assert_eq!(value.wake.news, c::WakeRule::Paced { span: Duration::ZERO });
}
#[test]
fn a_shown_charter_with_a_wake_policy_applies_unchanged() {
    let value = charter(&charter_file(COORDINATOR));
    let bounds = JsonLimits { depth: 32, length: 8192 };
    let mut measure = Encoder::measure(&bounds);
    encode_file(&value, &mut measure);
    let mut out = Encoder::write(measure.measured().unwrap(), &bounds);
    encode_file(&value, &mut out);
    assert_eq!(charter(&out.finish()), value);
}
fn encode_file(value: &c::Charter, out: &mut Encoder) {
    out.object_start();
    out.key(b"items");
    out.array_start();
    out.object_start();
    out.key(b"setting");
    out.object_start();
    out.key(b"kind");
    out.string(b"charter");
    out.key(b"number");
    out.unsigned(5);
    out.object_end();
    out.key(b"set");
    values::encode_charter(value, out);
    out.object_end();
    out.array_end();
    out.object_end();
}

const LANDING: &[u8] = br#"{"pattern":{"segments":["666f726765"],"last":{"exact":"6d61696e"}},"enforce":true,"ci":false,"up_to_date":true,"gates":[{"number":2,"blocking":true,"freshness":"clean"}],"approvals":[{"role":3,"people":1,"freshness":"exact"}]}"#;
const DEPLOYMENT_RULE: &[u8] = br#"{"kind":"landing-rule","number":1}"#;
const PROJECT_RULE: &[u8] = br#"{"kind":"landing-rule","project":7,"number":2}"#;
const FORGE_CHARTER: &[u8] = br#"{"kind":"forge-charter","number":1}"#;
fn joined(parts: &[&[u8]]) -> Box<[u8]> {
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("fixture length fits");
    }
    let mut out = Writer::new(length);
    for part in parts {
        out.put(part).expect("counted fixture bytes");
    }
    out.finish()
}
fn kind_file(setting: &[u8], value: &[u8]) -> Box<[u8]> {
    joined(&[br#"{"items":[{"setting":"#, setting, br#", "set":"#, value, b"}]}"])
}
fn shown(value: &c::ReadValue) -> Box<[u8]> {
    let bounds = JsonLimits { depth: 32, length: 8192 };
    let mut measure = Encoder::measure(&bounds);
    crate::render::encode(value, &mut measure);
    let mut out = Encoder::write(measure.measured().expect("fixture JSON bound"), &bounds);
    crate::render::encode(value, &mut out);
    out.finish()
}
#[test]
fn temper_kinds_round_trip_from_file_to_shown_json() {
    for (setting, json, expected) in [
        (DEPLOYMENT_RULE, LANDING, c::Setting::Application { kind: 1, scope: 0, number: 1 }),
        (PROJECT_RULE, LANDING, c::Setting::Application { kind: 2, scope: 7, number: 2 }),
        (FORGE_CHARTER, b"23".as_slice(), c::Setting::Application { kind: 3, scope: 0, number: 1 }),
    ] {
        let items = file::decode(&kind_file(setting, json), &Limits::standard()).expect("temper kind file");
        let item = items.get(0).expect("one item");
        assert_eq!(item.setting, expected);
        let c::SettingEdit::Set(set) = &item.edit else { panic!("set value") };
        let document = c::ReadValue::Setting(Box::new(c::SettingDocument {
            setting: item.setting,
            revision: 1,
            value: Some(set.value.clone()),
            application: None,
            proposed: None,
        }));
        assert_eq!(shown(&document), joined(&[br#"{"setting":"#, setting, br#","revision":1,"value":"#, json, b"}"]));
    }
}
#[test]
fn temper_kind_edits_keep_removed_and_reset() {
    for setting in [DEPLOYMENT_RULE, PROJECT_RULE, FORGE_CHARTER] {
        for (word, expected) in [(b"removed".as_slice(), c::SettingEdit::Removed), (b"reset", c::SettingEdit::Reset)] {
            let bytes = joined(&[br#"{"items":[{"setting":"#, setting, br#", "edit":""#, word, br#""}]}"#]);
            let items = file::decode(&bytes, &Limits::standard()).expect("kind edit");
            assert_eq!(items.get(0).expect("one edit").edit, expected);
        }
    }
}
#[test]
fn temper_kind_member_refusals_name_the_file_path() {
    for (setting, value, expected) in [
        (
            br#"{"kind":"landing-rule","project":0,"number":1}"#.as_slice(),
            LANDING,
            b"settings file item 1: landing-rule.project: from 1".as_slice(),
        ),
        (br#"{"kind":"landing-rule","number":0}"#, LANDING, b"settings file item 1: landing-rule.number: from 1"),
        (DEPLOYMENT_RULE, br#"{"other":true}"#, b"settings file item 1: landing-rule.other: unknown member"),
        (DEPLOYMENT_RULE, br#"{"connector":1}"#, b"settings file item 1: landing-rule.connector: unknown member"),
        (DEPLOYMENT_RULE, br#"{"kind":4}"#, b"settings file item 1: landing-rule.kind: unknown member"),
        (FORGE_CHARTER, b"null", b"settings file item 1: forge-charter: expected unsigned integer"),
        (
            DEPLOYMENT_RULE,
            br#"{"pattern":{"segments":[null]}}"#,
            b"settings file item 1: landing-rule.pattern.segments: expected string",
        ),
        (
            DEPLOYMENT_RULE,
            br#"{"pattern":{"segments":[],"last":{"open":"XX"}}}"#,
            b"settings file item 1: landing-rule.pattern.last.open: lower-case hexadecimal required",
        ),
    ] {
        assert_eq!(
            file::decode(&kind_file(setting, value), &Limits::standard()).expect_err("bad member").as_ref(),
            expected
        );
    }
}
#[test]
fn landing_rule_lists_hold_at_their_limits() {
    let mut limits = Limits::standard();
    // LANDING contains one gate, one approval, one base segment and a four-byte last segment.
    limits.kinds.landing_rule_gates = 1;
    limits.kinds.landing_rule_approvals = 1;
    limits.settings.pattern_segments = 1;
    limits.settings.pattern_segments_item = 5;
    limits.settings.pattern_end_exact_value = 4;
    file::decode(&kind_file(DEPLOYMENT_RULE, LANDING), &limits).expect("one member at each limit");
}
#[test]
fn landing_rule_members_past_the_given_limits_name_their_paths() {
    let base = Limits::standard();
    let mut gates = base;
    gates.kinds.landing_rule_gates = 0;
    let mut approvals = base;
    approvals.kinds.landing_rule_approvals = 0;
    let mut segments = base;
    segments.settings.pattern_segments = 0;
    let mut segment_bytes = base;
    segment_bytes.settings.pattern_segments_item = 4;
    let mut last = base;
    last.settings.pattern_end_exact_value = 3;
    let mut kind_segments = base;
    kind_segments.kinds.landing_pattern_segments = 0;
    let mut kind_segment_bytes = base;
    kind_segment_bytes.kinds.landing_pattern_segments_item = 4;
    let mut kind_last = base;
    kind_last.kinds.landing_exact_value = 3;
    for (limits, expected) in [
        (gates, b"settings file item 1: landing-rule.gates: bound 0".as_slice()),
        (approvals, b"settings file item 1: landing-rule.approvals: bound 0"),
        (segments, b"settings file item 1: landing-rule.pattern.segments: bound 0"),
        (segment_bytes, b"settings file item 1: landing-rule.pattern.segments: bound 4"),
        (last, b"settings file item 1: landing-rule.pattern.last.exact: bound 3"),
        (kind_segments, b"settings file item 1: landing-rule.pattern.segments: bound 0"),
        (kind_segment_bytes, b"settings file item 1: landing-rule.pattern.segments: bound 4"),
        (kind_last, b"settings file item 1: landing-rule.pattern.last.exact: bound 3"),
    ] {
        assert_eq!(
            file::decode(&kind_file(DEPLOYMENT_RULE, LANDING), &limits).expect_err("bound refusal").as_ref(),
            expected
        );
    }
}

const PROJECT_POLICY: &[u8] = br#"{"kind":"project-policy","project":7}"#;
const POLICY: &[u8] = br#"{"escalation_role":null,"ceiling":{"models":"all","tools":1,"grants":[],"delegation":{"kinds":[],"tasks":1,"depth":1},"spend":100,"deadline":null,"notes":0,"note_resources":[]},"projections":[],"period_spend":200,"roles":[],"requirements":[],"permissions":[],"templates":[]}"#;
fn encode_policy_file(setting: c::Setting, value: &c::SettingValue, out: &mut Encoder) {
    out.object_start();
    out.key(b"items");
    out.array_start();
    out.object_start();
    out.key(b"setting");
    crate::render::encode_setting(setting, out);
    out.key(b"set");
    crate::render::encode_value(value, out);
    out.object_end();
    out.array_end();
    out.object_end();
}
#[test]
fn a_project_policy_item_decodes_to_its_setting_and_a_policy_value() {
    let limits = Limits::standard();
    let items = file::decode(&kind_file(PROJECT_POLICY, POLICY), &limits).expect("project policy");
    let item = items.get(0).expect("one project policy item");
    assert_eq!(item.setting, c::Setting::ProjectPolicy { project: 7 });
    let c::SettingEdit::Set(set) = &item.edit else { panic!("set policy") };
    let numbered =
        file::decode(&kind_file(br#"{"kind":"policy","number":1}"#, POLICY), &limits).expect("numbered policy");
    assert_eq!(item.edit, numbered.get(0).expect("one numbered policy").edit);
    let bounds = JsonLimits { depth: 32, length: 8192 };
    let mut measure = Encoder::measure(&bounds);
    encode_policy_file(item.setting, &set.value, &mut measure);
    let mut out = Encoder::write(measure.measured().expect("policy file fits"), &bounds);
    encode_policy_file(item.setting, &set.value, &mut out);
    assert_eq!(file::decode(&out.finish(), &limits).expect("encoded policy file"), items);
    let wire = crate::wire::settings_wire_setting(item.setting, &limits.settings).expect("wire project policy");
    assert_eq!(crate::wire::settings_restore_setting(&wire, &limits), item.setting);
}
#[test]
fn a_project_policy_edit_a_second_one_or_a_bad_project_is_refused_naming_the_item() {
    for (bytes, expected) in [
        (
            joined(&[br#"{"items":[{"setting":"#, PROJECT_POLICY, br#", "edit":"removed"}]}"#]),
            b"settings file item 1: edit: a project's policy is set alone".as_slice(),
        ),
        (
            joined(&[br#"{"items":[{"setting":"#, PROJECT_POLICY, br#", "edit":"reset"}]}"#]),
            b"settings file item 1: edit: a project's policy is set alone",
        ),
        (
            joined(&[
                br#"{"items":[{"setting":"#,
                PROJECT_POLICY,
                br#", "set":"#,
                POLICY,
                br#"},{"setting":{"kind":"project-policy","project":8},"set":"#,
                POLICY,
                b"}]}",
            ]),
            b"settings file item 2: setting: one project's policy per file",
        ),
        (kind_file(br#"{"kind":"project-policy","project":0}"#, POLICY), b"settings file item 1: project: from 1"),
        (kind_file(br#"{"kind":"project-policy"}"#, POLICY), b"settings file item 1: project: missing member"),
        (
            kind_file(br#"{"kind":"project-policy","project":7,"number":1}"#, POLICY),
            b"settings file item 1: number: project-policy has no number",
        ),
    ] {
        assert_eq!(file::decode(&bytes, &Limits::standard()).expect_err("refused policy item").as_ref(), expected);
    }
}

#[test]
fn revision_replaced_states_reach_the_host_settings_form_from_the_wire() {
    use jig_client_protocol_wire as w;
    let limits = Limits::standard();
    let set = c::SettingSet { value: c::SettingValue::Charter(Box::new(charter(&charter_file(COORDINATOR)))) };
    let cases = [
        (
            w::RevisionReplaced::Value(crate::wire::settings_wire_setting_set(&set, &limits.settings).unwrap()),
            c::RevisionReplaced::Value(set),
        ),
        (
            w::RevisionReplaced::TooLarge(w::Size::new(&limits.settings, w::SizeParts { bytes: 1_000_000 }).unwrap()),
            c::RevisionReplaced::TooLarge(c::Size { bytes: 1_000_000 }),
        ),
        (
            w::RevisionReplaced::Application(
                w::ApplicationSize::new(&limits.settings, w::ApplicationSizeParts { size: 12 }).unwrap(),
            ),
            c::RevisionReplaced::Application { size: 12 },
        ),
        (w::RevisionReplaced::Default, c::RevisionReplaced::Default),
        (w::RevisionReplaced::Removed, c::RevisionReplaced::Removed),
        (w::RevisionReplaced::NotOffered, c::RevisionReplaced::NotOffered),
    ];
    for (replaced, expected) in cases {
        let item = w::RevisionItem::new(
            &limits.settings,
            w::RevisionItemParts { setting: w::Setting::Rules, edit: w::RevisionEdit::Reset, replaced },
        )
        .unwrap();
        let restored = crate::wire::settings_restore_revision_item(&item, &limits).unwrap();
        assert_eq!(restored.replaced, expected);
    }
}

//! Charter settings in the file's forms (temper/docs/design/client/ctl/tools.md, 3).
use crate::{Limits, file, values, view as c};
use alloc::boxed::Box;
use skein_json::writer::{Encoder, Limits as JsonLimits};
use skein_lib::{Duration, Writer};

// C7's coordinator paces news at 30 minutes, in the settings file's nanoseconds.
const COORDINATOR: &[u8] = br#", "wake":{"words":"immediate","notices":"immediate","news":{"paced":1800000000000},"results":"each","questions":true,"answers":true,"timers":true}"#;
const PREFIX: &[u8] = br#"{"items":[{"setting":{"kind":"charter","number":5},"set":{"number":5,"instructions":"coordinate","waiting":1,"resume":true,"turns":2,"time":2,"reserve_turns":1,"reserve_time":1,"model":{"dialect":1,"endpoint":1,"program":null,"name":"model","window":8192,"output":1024,"input_price":1,"cached_price":1,"cache_write_price":null,"output_price":1,"price_unit":1000,"efforts":[],"effort":null},"alternatives":[],"inspect":true,"modify":false,"shell":false,"agents":false,"call_timeout":1,"hosts":"both","spend":100,"documents":[]"#;
const SUFFIX: &[u8] = b"}}]}";
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
const TEMPLATE_GOAL: &[u8] = br#"{"kind":"template-goal","number":4}"#;
const GOAL: &[u8] = br#"{"design":true,"close_out":false}"#;
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
const CHARTER_WORDS: &[u8] = br#"{"kind":"charter-words","number":4}"#;
const PROJECT_WORDS: &[u8] = br#"{"kind":"project-words","project":7}"#;
const LINE: &[u8] = br#""llllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllll""#;
const WORDS: &[u8] = br#"{"headline":"hhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhh","lede":"llllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllll"}"#;
#[test]
fn temper_kinds_round_trip_from_file_to_shown_json() {
    for (setting, json, expected) in [
        (DEPLOYMENT_RULE, LANDING, c::Setting::Application { kind: 1, scope: 0, number: 1 }),
        (PROJECT_RULE, LANDING, c::Setting::Application { kind: 2, scope: 7, number: 2 }),
        (
            PROJECT_RULE,
            br#"{"pattern":{"segments":["6272616e6368"],"last":{"exact":"6d61696e"}},"enforce":true,"ci":true,"up_to_date":true,"gates":[],"approvals":[]}"#,
            c::Setting::Application { kind: 2, scope: 7, number: 2 },
        ),
        (FORGE_CHARTER, b"23".as_slice(), c::Setting::Application { kind: 3, scope: 0, number: 1 }),
        (CHARTER_WORDS, LINE, c::Setting::Application { kind: 9, scope: 0, number: 4 }),
        (PROJECT_WORDS, WORDS, c::Setting::Application { kind: 10, scope: 7, number: 1 }),
        (TEMPLATE_GOAL, GOAL, c::Setting::Application { kind: 11, scope: 0, number: 4 }),
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
    for setting in [DEPLOYMENT_RULE, PROJECT_RULE, FORGE_CHARTER, TEMPLATE_GOAL] {
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
        (TEMPLATE_GOAL, br#"{"design":true}"#, b"settings file item 1: template-goal.close_out: missing member"),
        (TEMPLATE_GOAL, br#"{"close_out":false}"#, b"settings file item 1: template-goal.design: missing member"),
        (
            TEMPLATE_GOAL,
            br#"{"design":true,"close_out":false,"other":true}"#,
            b"settings file item 1: template-goal.other: unknown member",
        ),
        (
            br#"{"kind":"template-goal","number":4,"project":7}"#,
            GOAL,
            b"settings file item 1: template-goal.project: unknown member",
        ),
        (br#"{"kind":"template-goal"}"#, GOAL, b"settings file item 1: template-goal.number: missing member"),
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
        (br#"{"kind":"charter-words","number":4,"project":7}"#, LINE, b"settings file item 1: charter-words.project: unknown member"),
        (br#"{"kind":"charter-words"}"#, LINE, b"settings file item 1: charter-words.number: missing member"),
        (CHARTER_WORDS, br#""lllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllll""#, b"settings file item 1: charter-words: wire value bound"),
        (PROJECT_WORDS, br#"{"headline":"Our way"}"#, b"settings file item 1: project-words.lede: missing member"),
        (PROJECT_WORDS, br#"{"lede":"We review."}"#, b"settings file item 1: project-words.headline: missing member"),
        (PROJECT_WORDS, br#"{"headline":false,"lede":"We review."}"#, b"settings file item 1: project-words.headline: expected string"),
        (PROJECT_WORDS, br#"{"headline":"hhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhh","lede":"We review."}"#, b"settings file item 1: project-words: wire value bound"),
        (PROJECT_WORDS, br#"{"headline":"Our way","lede":"lllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllll"}"#, b"settings file item 1: project-words: wire value bound"),
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
        (Box::new(gates), b"settings file item 1: landing-rule.gates: bound 0".as_slice()),
        (Box::new(approvals), b"settings file item 1: landing-rule.approvals: bound 0"),
        (Box::new(segments), b"settings file item 1: landing-rule.pattern.segments: bound 0"),
        (Box::new(segment_bytes), b"settings file item 1: landing-rule.pattern.segments: bound 4"),
        (Box::new(last), b"settings file item 1: landing-rule.pattern.last.exact: bound 3"),
        (Box::new(kind_segments), b"settings file item 1: landing-rule.pattern.segments: bound 0"),
        (Box::new(kind_segment_bytes), b"settings file item 1: landing-rule.pattern.segments: bound 4"),
        (Box::new(kind_last), b"settings file item 1: landing-rule.pattern.last.exact: bound 3"),
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

const TEMPLATE_PREFIX: &[u8] = br#"{"items":[{"setting":{"kind":"template","number":4},"set":{"number":4,"name":{"bytes":"draft","cut":false},"executor":{"charter":1},"contract":{"report":{"words":1024}},"shallower":1,"budget":"given","deadline":"none","tracked":1,"flags":{"conversation":false,"listed":true,"retention":"kept","ended_by":"result"},"label":1,"grace":null,"llm":null"#;
fn template_file(resources: &[u8]) -> Box<[u8]> {
    let length = TEMPLATE_PREFIX
        .len()
        .checked_add(resources.len())
        .expect("fixture resources")
        .checked_add(SUFFIX.len())
        .expect("suffix");
    let mut out = Writer::new(length);
    out.put(TEMPLATE_PREFIX).unwrap();
    out.put(resources).unwrap();
    out.put(SUFFIX).unwrap();
    out.finish()
}
fn template(bytes: &[u8], limits: &Limits) -> c::Template {
    let items = file::decode(bytes, limits).expect("template file");
    let c::SettingEdit::Set(set) = &items.get(0).unwrap().edit else { panic!("set") };
    let c::SettingValue::Template(value) = &set.value else { panic!("template") };
    *value.clone()
}
#[test]
fn a_settings_file_template_names_its_resources() {
    let limits = Limits::standard();
    let value = template(&template_file(br#", "resources":[{"connector":1,"symbol":1}]"#), &limits);
    assert_eq!(value.resources.as_slice(), &[c::Symbol { connector: 1, symbol: 1 }]);
    let wire = crate::wire::settings_wire_template(&value, &limits.settings).unwrap();
    assert_eq!(wire.resources().get(0).unwrap().connector(), 1);
    assert_eq!(wire.resources().get(0).unwrap().symbol(), 1);
    assert_eq!(crate::wire::documents_me_template(&wire, &limits).unwrap(), value);
}
#[test]
fn a_settings_file_template_without_resources_names_none() {
    for resources in [b"".as_slice(), br#", "resources":[]"#] {
        assert!(template(&template_file(resources), &Limits::standard()).resources.is_empty());
    }
}
#[test]
fn a_settings_file_template_refuses_a_resource_pair_twice() {
    let bytes = template_file(br#", "resources":[{"connector":1,"symbol":1},{"connector":1,"symbol":1}]"#);
    assert_eq!(
        file::decode(&bytes, &Limits::standard()).unwrap_err().as_ref(),
        b"settings file item 1: template.resources: duplicate pair"
    );
}
#[test]
fn a_settings_file_template_bounds_resources_before_decoding_members() {
    let mut limits = Limits::standard();
    limits.settings.template_resources = 2;
    let bytes = template_file(br#", "resources":[{"connector":1,"symbol":1},{"connector":1,"symbol":2}]"#);
    assert_eq!(template(&bytes, &limits).resources.len(), 2);
    let bytes = template_file(br#", "resources":[{"connector":1,"symbol":1},{"connector":1,"symbol":2},null]"#);
    assert_eq!(
        file::decode(&bytes, &limits).unwrap_err().as_ref(),
        b"settings file item 1: template.resources: bound 2"
    );
}
#[test]
fn a_settings_file_template_refuses_malformed_resources_naming_the_path() {
    for (resources, member) in [
        (br#", "resources":null"#.as_slice(), b"template.resources".as_slice()),
        (br#", "resources":{}"#, b"template.resources"),
        (br#", "resources":[false]"#, b"template.resources"),
        (br#", "resources":[{"connector":"1","symbol":1}]"#, b"template.resources.connector"),
        (br#", "resources":[{"connector":1,"symbol":false}]"#, b"template.resources.symbol"),
        (br#", "resources":[{"connector":65536,"symbol":1}]"#, b"template.resources.connector"),
        (br#", "resources":[{"connector":1,"symbol":65536}]"#, b"template.resources.symbol"),
        (br#", "resources":[{"connector":1}]"#, b"template.resources.symbol"),
        (br#", "resources":[{"connector":1,"symbol":1,"extra":1}]"#, b"template.resources.extra"),
    ] {
        let why = file::decode(&template_file(resources), &Limits::standard()).unwrap_err();
        assert!(skein_lib::bytes::find(&why, member).is_some(), "{why:?}");
    }
}

#[test]
fn a_charters_tools_round_trip() {
    let source = charter(&charter_file(br#", "tools":1536"#));
    assert_eq!(source.tools, 1536);
    let limits = Limits::standard();
    let wire = crate::wire::settings_wire_charter(&source, &limits.settings).unwrap();
    assert_eq!(wire.tools(), 1536);
    let restored = crate::wire::settings_restore_charter(&wire, &limits).unwrap();
    assert_eq!(restored, source);
    let bounds = JsonLimits { depth: 32, length: 8192 };
    let mut measure = Encoder::measure(&bounds);
    encode_file(&restored, &mut measure);
    let mut out = Encoder::write(measure.measured().unwrap(), &bounds);
    encode_file(&restored, &mut out);
    let encoded = out.finish();
    assert!(skein_lib::bytes::find(&encoded, br#""tools":1536"#).is_some());
    assert_eq!(charter(&encoded), source);
}

#[test]
fn a_charter_left_without_tools_names_no_settings() {
    let value = charter(&charter_file(b""));
    assert_eq!(crate::CHARTER_TOOLS_DEFAULT, 1023);
    assert_eq!(value.tools, crate::CHARTER_TOOLS_DEFAULT);
    assert_eq!(value.tools & 1024, 0);
}

#[test]
fn forge_document_forms_use_the_registered_connector_and_round_trip() {
    let prefix = PREFIX.strip_suffix(br#","documents":[]"#).expect("charter documents suffix");
    for host in [b"127.0.0.1:3000".as_slice(), b"127.0.0.1", b"localhost", b"forgejo", b"1"] {
        for (heading, expected) in [
            (b"".as_slice(), None),
            (br#","heading":null"#, None),
            (br#","heading":"Rules""#, Some(b"Rules".as_slice())),
        ] {
            let bytes = joined(&[
                prefix,
                br#","documents":[{"forge":""#,
                host,
                br#"","repository":"playground/widget","branch":"main","path":"README.md""#,
                heading,
                b"}]",
                SUFFIX,
            ]);
            let value = charter(&bytes);
            let document = value.documents.get(0).expect("one document");
            assert_eq!(document.connector, 1);
            assert_eq!(document.name.get(1).expect("host").as_ref(), host);
            assert_eq!(document.heading.as_deref(), expected);
            let bounds = JsonLimits { depth: 32, length: 8192 };
            let mut measure = Encoder::measure(&bounds);
            encode_file(&value, &mut measure);
            let mut out = Encoder::write(measure.measured().expect("charter fits"), &bounds);
            encode_file(&value, &mut out);
            let shown = out.finish();
            assert!(skein_lib::bytes::find(&shown, br#""repository":"playground/widget""#).is_some());
            assert_eq!(charter(&shown), value);
        }
    }
}

const PROJECT_WAYS: &[u8] = br#"{"kind":"project-ways","project":7}"#;
const PROJECT_CLASSES: &[u8] = br#"{"kind":"project-classes","project":7}"#;
const CLASSES: &[u8] = br#"{"items":[{"kind":"implementation","words":"","class":"decide"},{"kind":"tests","words":"Keep coverage","class":"ask"},{"set":1,"words":"","class":"ask-away"}],"sets":[{"number":1,"name":"Core","patterns":["src/**","Cargo.*"]}]}"#;
const WAYS: &[u8] = br#"{"drafts":true,"goals":false,"changes":true,"way":4294967295}"#;
#[test]
fn project_ways_is_read_and_printed_whole() {
    for (setting, kind, value) in [(PROJECT_WAYS, 7, WAYS), (PROJECT_CLASSES, 8, CLASSES)] {
        let items = file::decode(&kind_file(setting, value), &Limits::standard()).expect("whole project file");
        let item = items.get(0).expect("one item");
        assert_eq!(item.setting, c::Setting::Application { kind, scope: 7, number: 1 });
        let c::SettingEdit::Set(set) = &item.edit else { panic!("whole value") };
        let c::SettingValue::Application(bytes) = &set.value else { panic!("project bytes") };
        let bounds = JsonLimits { depth: 32, length: 4096 };
        let mut measure = Encoder::measure(&bounds);
        assert!(crate::kinds::render(kind, bytes, &mut measure));
        let mut out = Encoder::write(measure.measured().expect("project value fits"), &bounds);
        assert!(crate::kinds::render(kind, bytes, &mut out));
        let shown = out.finish();
        assert_eq!(shown.as_ref(), value);
        assert_eq!(file::decode(&kind_file(setting, &shown), &Limits::standard()).expect("rendered file"), items);
        let mut trailing = Writer::new(bytes.len().checked_add(1).expect("one extra byte"));
        trailing.put(bytes).expect("encoding room");
        trailing.put(&[0]).expect("trailing byte room");
        assert!(!crate::kinds::render(kind, &trailing.finish(), &mut Encoder::measure(&bounds)));
        let mut selector = Encoder::measure(&bounds);
        crate::render::encode_setting(item.setting, &mut selector);
        let mut out = Encoder::write(selector.measured().expect("selector fits"), &bounds);
        crate::render::encode_setting(item.setting, &mut out);
        assert_eq!(out.finish().as_ref(), setting);
    }
    let no_words = br#"{"items":[{"kind":"implementation","class":"decide"}],"sets":[]}"#;
    let items = file::decode(&kind_file(PROJECT_CLASSES, no_words), &Limits::standard()).expect("optional words");
    let c::SettingEdit::Set(set) = &items.get(0).expect("one item").edit else { panic!("whole value") };
    let c::SettingValue::Application(bytes) = &set.value else { panic!("classes bytes") };
    let mut reader = skein_lib::Reader::new(bytes);
    let record = temper_client_protocol_wire::v1::ProjectClasses::decode(
        &temper_client_protocol_wire::v1::CEILINGS,
        &mut reader,
    )
    .expect("classes record");
    assert!(reader.is_empty());
    assert_eq!(temper_protocol_reaches::classes_from(&record).items[0].words.as_ref(), b"");
}
#[test]
fn project_settings_require_the_project_and_refuse_a_number() {
    for (kind, number) in
        [(b"project-ways".as_slice(), 7), (b"project-questions", 6), (b"project-classes", 8), (b"project-words", 10)]
    {
        for (suffix, member) in [
            ("", "project"),
            (",\"project\":0", "project"),
            (",\"project\":4294967296", "project"),
            (",\"project\":7,\"number\":1", "number"),
        ] {
            let bytes = joined(&[br#"{"setting":{"kind":""#, kind, b"\"", suffix.as_bytes(), b"}}"]);
            let why = file::read_setting(&bytes, &Limits::standard()).expect_err("bad selector");
            assert!(skein_lib::bytes::find(&why, member.as_bytes()).is_some(), "{why:?}");
        }
        let selector = joined(&[br#"{"setting":{"kind":""#, kind, br#"","project":7}}"#]);
        assert_eq!(
            file::read_setting(&selector, &Limits::standard()).expect("project selector"),
            c::Setting::Application { kind: number, scope: 7, number: 1 }
        );
    }
}
#[test]
fn project_values_refuse_missing_unknown_and_wrong_members() {
    let full = kind_file(PROJECT_CLASSES, CLASSES);
    for (member, field) in
        [(b"items".as_slice(), 0_u8), (b"sets", 1), (b"words", 2), (b"name", 3), (b"patterns", 4), (b"patterns", 5)]
    {
        let mut limits = Limits::standard();
        match field {
            0 => limits.kinds.project_classes_items = 0,
            1 => limits.kinds.project_classes_sets = 0,
            2 => limits.kinds.class_item_words = 0,
            3 => limits.kinds.path_set_name = 0,
            4 => limits.kinds.path_set_patterns = 0,
            5 => limits.kinds.path_set_patterns_item = 0,
            _ => unreachable!("table field"),
        }
        let why = file::decode(&full, &limits).expect_err("receiving bound enforced");
        assert!(skein_lib::bytes::find(&why, member).is_some(), "{why:?}");
        assert!(skein_lib::bytes::find(&why, b"bound 0").is_some(), "{why:?}");
    }
    // A well-shaped wire record can still be outside the table's rules.
    for (kind, class) in [
        (temper_protocol_reaches::Kind::Money, temper_protocol_reaches::Class::Ask),
        (temper_protocol_reaches::Kind::Tests, temper_protocol_reaches::Class::Wait),
    ] {
        let classes = temper_protocol_reaches::Classes {
            items: Box::new([temper_protocol_reaches::Item {
                subject: temper_protocol_reaches::Subject::Kind(kind),
                words: Box::default(),
                class,
            }]),
            sets: Box::default(),
        };
        let record = temper_protocol_reaches::classes_record(&classes).expect("shaped record");
        let mut bytes = Writer::new(usize::try_from(record.measure()).expect("record length"));
        record.encode(&mut bytes).expect("measured record");
        let mut out = Encoder::measure(&JsonLimits { depth: 32, length: 4096 });
        assert!(!crate::kinds::render(8, &bytes.finish(), &mut out));
    }
    for (value, member) in [
        (b"{}".as_slice(), b"items".as_slice()),
        (br#"{"items":[]}"#.as_slice(), b"sets".as_slice()),
        (br#"{"items":[],"sets":[],"extra":0}"#.as_slice(), b"extra".as_slice()),
        (br#"{"items":[{"kind":"money","class":"ask"}],"sets":[]}"#.as_slice(), b"always wait".as_slice()),
        (br#"{"items":[{"kind":"wording","class":"wait"}],"sets":[]}"#.as_slice(), b"class".as_slice()),
        (br#"{"items":[{"kind":"unknown","class":"ask"}],"sets":[]}"#.as_slice(), b"kind".as_slice()),
        (br#"{"items":[{"kind":"tests","set":1,"class":"ask"}],"sets":[]}"#.as_slice(), b"kind/set".as_slice()),
        (br#"{"items":[{"class":"ask"}],"sets":[]}"#.as_slice(), b"kind/set".as_slice()),
        (br#"{"items":[{"kind":"tests"}],"sets":[]}"#.as_slice(), b"class".as_slice()),
        (br#"{"items":[{"set":1,"class":"ask"}],"sets":[]}"#.as_slice(), b"items.set".as_slice()),
        (br#"{"items":[{"kind":"tests","class":"ask","extra":1}],"sets":[]}"#.as_slice(), b"extra".as_slice()),
        (
            br#"{"items":[{"kind":"tests","class":"ask"},{"kind":"tests","class":"decide"}],"sets":[]}"#.as_slice(),
            b"items.words".as_slice(),
        ),
        (
            br#"{"items":[],"sets":[{"number":0,"name":"Core","patterns":["src/**"]}]}"#.as_slice(),
            b"sets.number".as_slice(),
        ),
        (br#"{"items":[],"sets":[{"number":1,"name":"","patterns":["src/**"]}]}"#.as_slice(), b"sets.name".as_slice()),
        (br#"{"items":[],"sets":[{"number":1,"name":"Core","patterns":[]}]}"#.as_slice(), b"sets.patterns".as_slice()),
        (
            br#"{"items":[],"sets":[{"number":1,"name":"Core","patterns":["/src"]}]}"#.as_slice(),
            b"sets.patterns".as_slice(),
        ),
        (
            br#"{"items":[],"sets":[{"number":256,"name":"Core","patterns":["src/**"]}]}"#.as_slice(),
            b"number".as_slice(),
        ),
    ] {
        let why = file::decode(&kind_file(PROJECT_CLASSES, value), &Limits::standard()).expect_err("malformed classes");
        assert!(skein_lib::bytes::find(&why, member).is_some(), "{why:?}");
    }
    for (value, member) in [
        (br#"{"drafts":true,"goals":true,"changes":true}"#.as_slice(), b"way".as_slice()),
        (br#"{"drafts":true,"goals":true,"way":0}"#, b"changes"),
        (br#"{"drafts":true,"changes":true,"way":0}"#, b"goals"),
        (br#"{"goals":true,"changes":true,"way":0}"#, b"drafts"),
        (br#"{"drafts":true,"goals":true,"changes":true,"way":4294967296}"#, b"way"),
        (br#"{"drafts":true,"goals":true,"changes":true,"way":-1}"#, b"way"),
        (br#"{"drafts":true,"goals":true,"changes":true,"way":false}"#, b"way"),
        (br#"{"drafts":1,"goals":true,"changes":true,"way":0}"#, b"drafts"),
        (br#"{"drafts":true,"goals":true,"changes":true,"way":0,"extra":true}"#, b"extra"),
        (br#"{"drafts":true,"goals":true,"changes":true,"away":true,"way":0}"#, b"away"),
    ] {
        let why = file::decode(&kind_file(PROJECT_WAYS, value), &Limits::standard()).expect_err("malformed ways");
        assert!(skein_lib::bytes::find(&why, member).is_some(), "{why:?}");
    }
    for route in [b"\"none\"".as_slice(), b"\"foreman\"", b"\"up\""] {
        file::decode(&kind_file(br#"{"kind":"project-questions","project":7}"#, route), &Limits::standard())
            .expect("valid route");
    }
    for value in [b"\"other\"".as_slice(), b"true", b"{}"] {
        file::decode(&kind_file(br#"{"kind":"project-questions","project":7}"#, value), &Limits::standard())
            .expect_err("malformed route");
    }
}

#[test]
fn the_settings_form_round_trips_a_model() {
    const SETTING: &[u8] = br#"{"kind":"model","number":4}"#;
    const MODEL: &[u8] = br#"{"number":4,"model":{"provider":2,"dialect":1,"endpoint":3,"program":null,"name":"gpt-6.1-terra","window":131072,"output":8192,"input_price":3,"cached_price":1,"cache_write_price":null,"output_price":12,"price_unit":1000000,"efforts":["high"],"effort":"high"},"shown":"GPT-6.1 Terra","description":"For careful work","picker":false,"withdrawn":false,"source":"stock","estimated":false}"#;
    let bytes = kind_file(SETTING, MODEL);
    let items = file::decode(&bytes, &Limits::standard()).expect("whole model file");
    let item = items.get(0).expect("model item");
    assert_eq!(item.setting, c::Setting::Model(c::SettingNumber { number: 4 }));
    let c::SettingEdit::Set(set) = &item.edit else { panic!("model set") };
    let c::SettingValue::Model(model) = &set.value else { panic!("model value") };
    assert_eq!(model.shown.as_ref().expect("shown name").bytes(), b"GPT-6.1 Terra");
    assert_eq!(model.model.input_price, 3);
    assert!(!model.picker);
    let document = c::ReadValue::Setting(Box::new(c::SettingDocument {
        setting: item.setting,
        revision: 2,
        value: Some(set.value.clone()),
        application: None,
        proposed: None,
    }));
    assert_eq!(shown(&document), joined(&[br#"{"setting":"#, SETTING, br#","revision":2,"value":"#, MODEL, b"}"]));
    assert_eq!(
        file::proposal_items(
            &joined(&[
                br#"{"against":1,"reason":"New prices", "items":[{"setting":"#,
                SETTING,
                br#","set":"#,
                MODEL,
                b"}]}"
            ]),
            &Limits::standard()
        )
        .expect("inline proposal"),
        items
    );
}

#[test]
fn inline_proposals_resolve_only_the_projects_repository_identities() {
    let repositories = [
        file::Repository { host: b"forge.example:3847", owner: b"ORG", name: b"Repo" },
        file::Repository { host: b"forge.example:3847", owner: b"ORG", name: b"Docs" },
    ];
    let prefix = PREFIX.strip_suffix(br#","documents":[]"#).unwrap();
    for (forge, repository, identities, expected) in [
        ("", "org/repo", Some(repositories.as_slice()), Ok(("forge.example:3847", "ORG", "Repo"))),
        (
            "",
            "org/other",
            Some(repositories.as_slice()),
            Err(
                "settings file item 1: charter.documents[0].repository: no repository org/other in this project; its repositories: ORG/Repo · ORG/Docs",
            ),
        ),
        (
            "",
            "org/other",
            Some([].as_slice()),
            Err(
                "settings file item 1: charter.documents[0].repository: no repository org/other in this project; it has none",
            ),
        ),
        (r#""forge":"given:1234","#, "org/repo", Some(repositories.as_slice()), Ok(("given:1234", "org", "repo"))),
        ("", "org/repo", None, Err("settings file item 1: charter.documents[0].forge: missing member")),
    ] {
        let bytes = joined(&[
            prefix,
            br#","documents":[{"#,
            forge.as_bytes(),
            br#""repository":""#,
            repository.as_bytes(),
            br#"","branch":"main","path":"README.md"}]"#,
            SUFFIX,
        ]);
        let result = file::proposal_items_in(&bytes, &Limits::standard(), identities);
        match expected {
            Ok((host, owner, name)) => {
                let items = result.expect("repository resolved");
                let c::SettingEdit::Set(set) = &items.get(0).unwrap().edit else { panic!("set") };
                let c::SettingValue::Charter(charter) = &set.value else { panic!("charter") };
                let document = charter.documents.get(0).unwrap();
                assert_eq!(document.name.get(1).unwrap().as_ref(), host.as_bytes());
                assert_eq!(document.name.get(2).unwrap().as_ref(), owner.as_bytes());
                assert_eq!(document.name.get(3).unwrap().as_ref(), name.as_bytes());
            }
            Err(why) => assert_eq!(result.unwrap_err().as_ref(), why.as_bytes()),
        }
    }
}

#[test]
fn a_templates_closer_reads_and_writes() {
    let original = joined(&[
        br#"{"items":[{"setting":{"kind":"template","number":4},"set":{"resources":[],"#,
        TEMPLATE_PREFIX.strip_prefix(br#"{"items":[{"setting":{"kind":"template","number":4},"set":{"#).unwrap(),
        SUFFIX,
    ]);
    let at = skein_lib::bytes::find(&original, br#""ended_by":"result""#).unwrap();
    let end = at.checked_add(br#""ended_by":"result""#.len()).unwrap();
    let bytes = joined(&[
        original.get(..at).unwrap(),
        br#""ended_by":{"closer":{"connector":1,"code":6}}"#,
        original.get(end..).unwrap(),
    ]);
    let limits = Limits::standard();
    let value = template(&bytes, &limits);
    assert_eq!(value.flags.ended_by, c::EndedBy::Closer(c::Closer { connector: 1, code: 6 }));
    let setting = c::Setting::Template(c::SettingNumber { number: 4 });
    let value = c::SettingValue::Template(Box::new(value));
    let bounds = JsonLimits { depth: 32, length: 8192 };
    let mut measure = Encoder::measure(&bounds);
    encode_policy_file(setting, &value, &mut measure);
    let mut out = Encoder::write(measure.measured().unwrap(), &bounds);
    encode_policy_file(setting, &value, &mut out);
    assert_eq!(out.finish(), bytes);
}

#[test]
fn follows_use_the_binding_json_form_and_default_optional_members() {
    use temper_client_protocol_wire::v1 as w;
    let selector = br#"{"kind":"follow","project":2,"number":1}"#;
    let explicit = br#"{"source":{"forge":1,"repository":12,"branch":"main","paths":["src","include"]},"target":{"forge":1,"repository":13,"branch":""},"owner":{"person":4},"start":"owner","producer":"6;3.1;","check":{"workflow":".forgejo/workflows/vsr-tests.yml","artifact":"results","format":"junit"}}"#;
    let defaults = br#"{"source":{"forge":1,"repository":12},"target":{"forge":1,"repository":13},"owner":{"person":4},"check":{"workflow":"tests.yml","artifact":"results","format":"lines"}}"#;
    for (json, optional) in [(explicit.as_slice(), false), (defaults.as_slice(), true)] {
        let items = file::decode(&kind_file(selector, json), &Limits::standard()).unwrap();
        let item = items.get(0).unwrap();
        assert_eq!(item.setting, c::Setting::Application { kind: 12, scope: 2, number: 1 });
        let c::SettingEdit::Set(set) = &item.edit else { panic!("set") };
        let c::SettingValue::Application(bytes) = &set.value else { panic!("follow bytes") };
        let record = w::Follow::decode(&w::CEILINGS, &mut skein_lib::Reader::new(bytes)).unwrap();
        assert_eq!(record.source().repository().forge(), 1);
        assert_eq!(record.source().repository().repository(), 12);
        assert_eq!(record.target().repository().repository(), 13);
        assert_eq!(record.start(), &w::FollowStart::Owner);
        assert_eq!(record.source().paths().is_empty(), optional);
        assert_eq!(record.source().branch().is_empty(), optional);
        assert_eq!(record.producer().is_none(), optional);
        assert!(record.target().branch().is_empty());
        let document = c::ReadValue::Setting(Box::new(c::SettingDocument {
            setting: item.setting,
            revision: 1,
            value: Some(set.value.clone()),
            application: None,
            proposed: None,
        }));
        let text = shown(&document);
        if !optional {
            assert_eq!(text, joined(&[br#"{"setting":"#, selector, br#","revision":1,"value":"#, json, b"}"]));
        }
        let mut out = Encoder::measure(&JsonLimits { depth: 32, length: 8192 });
        crate::render::encode_setting(item.setting, &mut out);
        out.measured().unwrap();
    }
    for selector in [br#"{"kind":"follow","number":1}"#.as_slice(), br#"{"kind":"follow","project":2}"#] {
        file::decode(&kind_file(selector, explicit), &Limits::standard()).unwrap_err();
    }
}

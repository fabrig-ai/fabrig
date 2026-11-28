//! Charter settings in the file's forms (temper/docs/design/client/ctl/tools-acts.md, 1).
use crate::{Limits, file, values, view as c};
use alloc::boxed::Box;
use skein_json::writer::{Encoder, Limits as JsonLimits};
use skein_lib::{Duration, Writer};
use temper_client_protocol_wire::v1::Record;

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
    let wire = crate::wire::settings_wire_charter(&value);
    assert_eq!(
        (&wire.wake.words),
        &jig_client_protocol_wire::WakeRule::Batch(jig_client_protocol_wire::WakeRuleBatch {
            count: 0,
            age: Duration::ZERO
        })
    );
    assert_eq!(value.wake.news, c::WakeRule::Paced { span: Duration::ZERO });
}
#[test]
fn a_shown_charter_with_a_wake_policy_applies_unchanged() {
    for name in [None, Some(b"coordinator".as_slice())] {
        let members = match name {
            Some(_) => joined(&[COORDINATOR, br#", "name":"coordinator""#]),
            None => COORDINATOR.into(),
        };
        let value = charter(&charter_file(&members));
        assert_eq!(value.name.as_deref(), name);
        let bounds = JsonLimits { depth: 32, length: 8192 };
        let mut measure = Encoder::measure(&bounds);
        encode_file(&value, &mut measure);
        let mut out = Encoder::write(measure.measured().unwrap(), &bounds);
        encode_file(&value, &mut out);
        let printed = out.finish();
        assert_eq!(skein_lib::bytes::find(&printed, br#""name":"coordinator""#).is_some(), name.is_some());
        assert_eq!(charter(&printed), value);
    }
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
fn charter_owners_keep_names_and_numbers_in_the_settings_form() {
    use temper_client_protocol_wire::v1 as w;
    use w::Record;
    for (json, expected) in [
        (
            br#""triager""#.as_slice(),
            w::AreaOwner::CharterNamed(w::AreaCharterNamed { name: b"triager".as_slice().into() }),
        ),
        (b"14".as_slice(), w::AreaOwner::Charter(w::AreaCharter { number: 14 })),
    ] {
        for (kind, setting, value) in [
            (
                14,
                br#"{"kind":"project-areas","project":3}"#.as_slice(),
                joined(&[br#"{"areas":[{"name":"tests","words":"C tests","owner":{"charter":"#, json, b"}}]}"]),
            ),
            (
                15,
                br#"{"kind":"project-triager","project":3}"#.as_slice(),
                joined(&[br#"{"owner":{"charter":"#, json, b"}}"]),
            ),
        ] {
            let items =
                file::decode(&kind_file(setting, &value), &Limits::standard()).expect("owner in a settings file");
            let item = items.get(0).expect("one item");
            assert_eq!(item.setting, c::Setting::Application { kind, scope: 3, number: 1 });
            let c::SettingEdit::Set(set) = &item.edit else { panic!("set value") };
            let c::SettingValue::Application(bytes) = &set.value else { panic!("owner bytes") };
            let mut reader = skein_lib::Reader::new(bytes);
            if kind == 14 {
                let areas = w::ProjectAreas::decode(&w::CEILINGS, &mut reader).unwrap();
                assert_eq!(areas.areas.get(0).unwrap().owner, expected);
            } else {
                let triager = w::ProjectTriager::decode(&w::CEILINGS, &mut reader).unwrap();
                assert_eq!(triager.owner, expected);
            }
            assert!(reader.is_empty());
            let document = c::ReadValue::Setting(Box::new(c::SettingDocument {
                overlay: None,
                setting: item.setting,
                revision: 1,
                value: Some(set.value.clone()),
                application: None,
                proposed: None,
            }));
            assert_eq!(
                shown(&document),
                joined(&[br#"{"setting":"#, setting, br#","revision":1,"value":"#, &value, b"}"])
            );
        }
    }
}

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
        (br#"{"kind":"project-areas","project":3}"#, br#"{"areas":[{"name":"tests","words":"vsr's C tests","owner":{"role":0}}]}"#, c::Setting::Application { kind: 14, scope: 3, number: 1 }),
        (br#"{"kind":"project-triager","project":3}"#, br#"{"owner":{"charter":14}}"#, c::Setting::Application { kind: 15, scope: 3, number: 1 }),
        (br#"{"kind":"project-landing","project":1}"#, b"{}", c::Setting::Application { kind: 4, scope: 1, number: 1 }),
        (br#"{"kind":"project-landing","project":1}"#, br#"{"wait":"landed"}"#, c::Setting::Application { kind: 4, scope: 1, number: 1 }),
        (br#"{"kind":"project-landing","project":1}"#, br#"{"by":"turn","batch":4}"#, c::Setting::Application { kind: 4, scope: 1, number: 1 }),
        (br#"{"kind":"project-gates","project":1}"#, br#"{"repositories":[{"forge":1,"repository":7,"gates":[{"name":"gate","workflow":"gate.yml","blocking":true,"report":{"artifact":"report","format":"lines","fuzzy":"fuzzy_"}}],"tiers":[{"name":"slow","workflow":"slow.yml","artifact":"report","format":"junit","branch":"goal"}]}]}"#, c::Setting::Application { kind: 5, scope: 1, number: 1 }),
        (TEMPLATE_GOAL, GOAL, c::Setting::Application { kind: 11, scope: 0, number: 4 }),
    ] {
        let items = file::decode(&kind_file(setting, json), &Limits::standard()).expect("temper kind file");
        let item = items.get(0).expect("one item");
        assert_eq!(item.setting, expected);
        let c::SettingEdit::Set(set) = &item.edit else { panic!("set value") };
        if item.setting == (c::Setting::Application { kind: 4, scope: 1, number: 1 }) {
            use temper_client_protocol_wire::v1 as w;
            let c::SettingValue::Application(bytes) = &set.value else { panic!("landing bytes") };
            let landing = w::ProjectLanding::decode(&w::CEILINGS, &mut skein_lib::Reader::new(bytes)).unwrap();
            assert_eq!(landing.wait, if json == br#"{"wait":"landed"}"# {
                w::SettingsLandingWait::Landed
            } else {
                w::SettingsLandingWait::Cleared
            });
        }
        let document = c::ReadValue::Setting(Box::new(c::SettingDocument {
            overlay: None,
            setting: item.setting,
            revision: 1,
            value: Some(set.value.clone()),
            application: None,
            proposed: None,
        }));
        assert_eq!(shown(&document), joined(&[br#"{"setting":"#, setting, br#","revision":1,"value":"#, json, b"}"]));
    }
}
const LANDING_SETTING: &[u8] = br#"{"kind":"project-landing","project":7}"#;
fn landing_round_trip(json: &[u8]) -> temper_client_protocol_wire::v1::ProjectLanding {
    use temper_client_protocol_wire::v1 as w;
    let items = file::decode(&kind_file(LANDING_SETTING, json), &Limits::standard()).expect("landing setting file");
    let item = items.get(0).unwrap();
    let c::SettingEdit::Set(set) = &item.edit else { panic!("set") };
    let c::SettingValue::Application(bytes) = &set.value else { panic!("kind 4 bytes") };
    let landing = w::ProjectLanding::decode(&w::CEILINGS, &mut skein_lib::Reader::new(bytes)).unwrap();
    let mut measure = Encoder::measure(&JsonLimits { depth: 32, length: 8192 });
    assert!(crate::kinds::render(4, bytes, &mut measure));
    let bounds = JsonLimits { depth: 32, length: 8192 };
    let mut out = Encoder::write(measure.measured().unwrap(), &bounds);
    assert!(crate::kinds::render(4, bytes, &mut out));
    assert_eq!(out.finish().as_ref(), json);
    landing
}
#[test]
fn landing_hook_ceiling_round_trips_and_defaults() {
    for (json, expected) in [(b"{}".as_slice(), 3_600), (br#"{"hook_timeout_max_seconds":1800}"#, 1_800)] {
        assert_eq!(landing_round_trip(json).hook_timeout_max_seconds, expected);
    }
}
#[test]
fn landing_flakes_round_trip_unset_never_and_rerun() {
    use temper_client_protocol_wire::v1::{SettingsFlakeRerun, SettingsFlakes};
    for (json, expected) in [
        (b"{}".as_slice(), None),
        (br#"{"flakes":"none"}"#, Some(SettingsFlakes::Never)),
        (br#"{"flakes":{"rerun":3}}"#, Some(SettingsFlakes::Rerun(SettingsFlakeRerun { processes: 3 }))),
    ] {
        assert_eq!(landing_round_trip(json).flakes, expected);
    }
}
#[test]
fn malformed_landing_flakes_are_refused_by_path() {
    for (json, why) in [
        (br#"{"flakes":"sometimes"}"#.as_slice(), b"flakes: none or an object with rerun required".as_slice()),
        (br#"{"flakes":{"rerun":-1}}"#, b"flakes.rerun: expected unsigned integer"),
        (br#"{"flakes":{"rerun":4294967296}}"#, b"flakes.rerun: u32 bound"),
        (br#"{"flakes":{"rerun":1,"other":0}}"#, b"flakes.other: unknown member"),
        (br#"{"flakes":{}}"#, b"flakes.rerun: missing member"),
    ] {
        let error = file::decode(&kind_file(LANDING_SETTING, json), &Limits::standard()).unwrap_err();
        assert_eq!(error.as_ref(), joined(&[b"settings file item 1: ", why]).as_ref());
    }
}
#[test]
fn repair_settings_default_counts_and_deadline_are_omitted_when_shown() {
    let landing = landing_round_trip(
        br#"{"wait":"landed","repairs":{"goal":{"charter":5,"llm":null,"priority":0,"budget":2000000}}}"#,
    );
    let repairs = landing.repairs.as_ref().unwrap();
    assert_eq!((repairs.goals, repairs.attempts, repairs.deadline_ms), (2, 2, 86_400_000));
    assert!(repairs.goal.llm.is_none());
}
#[test]
fn repair_settings_round_trip_all_members_and_the_templates_named_llm() {
    use jig_client_protocol_wire as jig;
    use temper_client_protocol_wire::v1 as w;
    let landing = landing_round_trip(br#"{"wait":"landed","repairs":{"repair_goals":16,"repair_attempts":8,"repair_deadline_ms":604800000,"goal":{"charter":5,"llm":{"account":3,"main":{"model":"main","effort":"high","speed":"fast"},"subagents":[{"model":"helper","effort":null}],"later":{"main":{"model":"later","effort":"low"},"subagents":[]}},"priority":42,"budget":2000000}}}"#);
    let repairs = landing.repairs.as_ref().unwrap();
    assert_eq!((repairs.goals, repairs.attempts, repairs.deadline_ms), (16, 8, 604_800_000));
    let named = repairs.goal.llm.as_ref().unwrap();
    assert_eq!(named.main.model.as_ref(), b"main");
    assert_eq!(named.main.speed.as_deref(), Some(b"fast".as_slice()));
    assert_eq!(named.subagents.get(0).unwrap().speed, None);

    // The named repair choice and jig wire choice share a layout, even for a populated speed.
    let bytes = named.encode(&w::CEILINGS).unwrap();
    let choice = jig::LlmChoice::decode(&jig::CEILINGS, &mut skein_lib::Reader::new(&bytes)).unwrap();
    assert_eq!(choice.main.speed.as_deref(), Some(b"fast".as_slice()));
    assert_eq!(choice.encode(&jig::CEILINGS).unwrap(), bytes);
    let bytes = landing.encode(&w::CEILINGS).unwrap();
    assert_eq!(w::ProjectLanding::decode(&w::CEILINGS, &mut skein_lib::Reader::new(&bytes)), Ok(landing));
}
#[test]
fn a_templates_model_speeds_round_trip_through_jigs_client_wire() {
    use jig_client_protocol_wire as jig;
    for (llm, speeds) in [
        (
            br#"{"account":null,"main":{"model":"main","effort":"high","speed":"fast"},"subagents":[{"model":"helper","effort":null,"speed":"standard"}],"later":{"main":{"model":"later","effort":"low","speed":"ultrafast"},"subagents":[]}}"#.as_slice(),
            [Some(b"fast".as_slice()), Some(b"standard".as_slice()), Some(b"ultrafast".as_slice())],
        ),
        (
            br#"{"account":null,"main":{"model":"main","effort":"high"},"subagents":[{"model":"helper","effort":null}],"later":{"main":{"model":"later","effort":"low"},"subagents":[]}}"#,
            [None, None, None],
        ),
    ] {
        let limits = Limits::standard();
        let value = template(&joined(&[TEMPLATE_PREFIX, llm, SUFFIX]), &limits);
        let choice = value.llm.as_ref().unwrap();
        for (model, speed) in [&choice.main, choice.subagents.get(0).unwrap(), &choice.later.as_ref().unwrap().main]
            .into_iter()
            .zip(speeds)
        {
            let named = match &model.speed {
                Some(speed) => Some(speed.bytes()),
                None => None,
            };
            assert_eq!(named, speed);
        }
        let wire = crate::wire::settings_wire_template(&value, &limits.settings).unwrap();
        let bytes = wire.encode(&limits.settings).unwrap();
        let wire = jig::Template::decode(&limits.settings, &mut skein_lib::Reader::new(&bytes)).unwrap();
        let restored = crate::wire::documents_me_template(&wire, &limits).unwrap();
        assert_eq!(restored, value);

        let bounds = JsonLimits { depth: 32, length: 8192 };
        let mut measure = Encoder::measure(&bounds);
        let choice = restored.llm.as_ref().unwrap();
        values::encode_llm_choice(choice, &mut measure);
        let mut out = Encoder::write(measure.measured().unwrap(), &bounds);
        values::encode_llm_choice(choice, &mut out);
        assert_eq!(out.finish().as_ref(), llm);
    }
}
#[test]
fn a_template_speed_is_text_bounded_at_the_wire_limit() {
    let prefix = br#"{"account":null,"main":{"model":"main","effort":null,"speed":"#;
    let suffix = br#"},"subagents":[]}"#;
    let limits = Limits::standard();
    let at_bound = br#""0123456789abcdef0123456789abcdef""#;
    file::decode(&joined(&[TEMPLATE_PREFIX, prefix, at_bound, suffix, SUFFIX]), &limits).expect("a 32-byte speed fits");
    for (speed, reason) in
        [(br#""0123456789abcdef0123456789abcdefg""#.as_slice(), b"bound 32".as_slice()), (b"null", b"expected string")]
    {
        let file = joined(&[TEMPLATE_PREFIX, prefix, speed, suffix, SUFFIX]);
        assert_eq!(
            file::decode(&file, &limits).unwrap_err().as_ref(),
            joined(&[b"settings file item 1: llm.main.speed: ", reason]).as_ref(),
        );
    }
}
#[test]
fn repair_settings_refuse_unknown_and_missing_members_by_path() {
    for (json, why) in [
        (
            br#"{"repairs":{"goal":{"charter":5,"priority":0,"budget":2},"other":0}}"#.as_slice(),
            b"repairs.other: unknown member".as_slice(),
        ),
        (
            br#"{"repairs":{"goal":{"charter":5,"priority":0,"budget":2,"other":0}}}"#,
            b"repairs.goal.other: unknown member",
        ),
        (br#"{"repairs":{}}"#, b"repairs.goal: missing member"),
        (br#"{"repairs":{"goal":{"priority":0,"budget":2}}}"#, b"repairs.goal.charter: missing member"),
        (br#"{"repairs":{"goal":{"charter":5,"budget":2}}}"#, b"repairs.goal.priority: missing member"),
        (br#"{"repairs":{"goal":{"charter":5,"priority":0}}}"#, b"repairs.goal.budget: missing member"),
    ] {
        let error = file::decode(&kind_file(LANDING_SETTING, json), &Limits::standard()).unwrap_err();
        assert_eq!(error.as_ref(), joined(&[b"settings file item 1: ", why]).as_ref());
    }
    // llm alone is optional, as for a template's default choice.
    file::decode(
        &kind_file(LANDING_SETTING, br#"{"wait":"landed","repairs":{"goal":{"charter":5,"priority":0,"budget":2}}}"#),
        &Limits::standard(),
    )
    .expect("llm may be absent");
}

#[test]
fn landing_wait_refuses_another_word_at_wait() {
    let bytes = kind_file(br#"{"kind":"project-landing","project":1}"#, br#"{"wait":"soon"}"#);
    assert_eq!(
        file::decode(&bytes, &Limits::standard()).expect_err("unknown wait").as_ref(),
        b"settings file item 1: wait: cleared or landed required"
    );
}

#[test]
fn welcome_is_an_unnumbered_deployment_setting_with_a_boolean_value_and_reset() {
    use temper_client_protocol_wire::v1 as w;
    let setting = br#"{"kind":"welcome"}"#;
    let items = file::decode(&kind_file(setting, br#"{"shown":true}"#), &Limits::standard()).unwrap();
    let item = items.get(0).unwrap();
    assert_eq!(item.setting, c::Setting::Application { kind: 20, scope: 0, number: 1 });
    let c::SettingEdit::Set(set) = &item.edit else { panic!("set welcome") };
    let c::SettingValue::Application(bytes) = &set.value else { panic!("welcome bytes") };
    assert!(w::Welcome::decode(&w::CEILINGS, &mut skein_lib::Reader::new(bytes)).unwrap().shown);
    let document = c::ReadValue::Setting(Box::new(c::SettingDocument {
        overlay: None,
        setting: item.setting,
        revision: 1,
        value: Some(set.value.clone()),
        application: None,
        proposed: None,
    }));
    assert_eq!(shown(&document).as_ref(), br#"{"setting":{"kind":"welcome"},"revision":1,"value":{"shown":true}}"#);
    let reset =
        file::decode(br#"{"items":[{"setting":{"kind":"welcome"},"edit":"reset"}]}"#, &Limits::standard()).unwrap();
    assert_eq!(reset.get(0).unwrap().setting, item.setting);
    assert_eq!(reset.get(0).unwrap().edit, c::SettingEdit::Reset);
    for bad in [
        br#"{"items":[{"setting":{"kind":"welcome","project":1},"set":{"shown":true}}]}"#.as_slice(),
        br#"{"items":[{"setting":{"kind":"welcome","number":1},"set":{"shown":true}}]}"#,
        br#"{"items":[{"setting":{"kind":"welcome"},"set":{"shown":"true"}}]}"#,
    ] {
        file::decode(bad, &Limits::standard()).expect_err("invalid welcome");
    }
}

fn landing_sets(member: &[u8]) -> Box<[u8]> {
    joined(&[LANDING.strip_suffix(b"}").expect("rule object"), member, b"}"])
}
#[test]
fn landing_sets_default_empty_and_round_trip_in_number_order_given() {
    use temper_client_protocol_wire::v1 as w;
    for (member, expected, printed) in [
        (b"".as_slice(), b"".as_slice(), b"".as_slice()),
        (br#","sets":[]"#, b"", b""),
        (br#","sets":[1,3]"#, &[1, 3], br#","sets":[1,3]"#),
        (br#","sets":[6,5,4,3,2,1]"#, &[6, 5, 4, 3, 2, 1], br#","sets":[6,5,4,3,2,1]"#),
    ] {
        let items =
            file::decode(&kind_file(PROJECT_RULE, &landing_sets(member)), &Limits::standard()).expect("sets file");
        let item = items.get(0).expect("one rule");
        let c::SettingEdit::Set(set) = &item.edit else { panic!("set rule") };
        let c::SettingValue::Application(bytes) = &set.value else { panic!("rule bytes") };
        let rule = w::LandingRule::decode(&w::CEILINGS, &mut skein_lib::Reader::new(bytes)).expect("rule record");
        assert_eq!(rule.sets.as_slice(), expected);
        let document = c::ReadValue::Setting(Box::new(c::SettingDocument {
            overlay: None,
            setting: item.setting,
            revision: 1,
            value: Some(set.value.clone()),
            application: None,
            proposed: None,
        }));
        assert_eq!(
            shown(&document),
            joined(&[br#"{"setting":"#, PROJECT_RULE, br#","revision":1,"value":"#, &landing_sets(printed), b"}"])
        );
    }
}
#[test]
fn landing_sets_refuse_wrong_types_numbers_and_counts_by_path() {
    for (member, expected) in [
        (br#","sets":false"#.as_slice(), b"settings file item 1: landing-rule.sets: expected array".as_slice()),
        (br#","sets":[true]"#, b"settings file item 1: landing-rule.sets: expected unsigned integer"),
        (br#","sets":[-1]"#, b"settings file item 1: landing-rule.sets: expected unsigned integer"),
        (br#","sets":[256]"#, b"settings file item 1: landing-rule.sets: u8 bound"),
        (br#","sets":[1,2,3,4,5,6,1]"#, b"settings file item 1: landing-rule.sets: bound 6"),
    ] {
        assert_eq!(
            file::decode(&kind_file(PROJECT_RULE, &landing_sets(member)), &Limits::standard())
                .expect_err("bad sets")
                .as_ref(),
            expected
        );
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
    let wire = crate::wire::settings_wire_setting(item.setting);
    assert_eq!(crate::wire::settings_restore_setting(&wire), item.setting);
}
#[test]
fn numbered_policies_read_and_print_optional_names() {
    let limits = Limits::standard();
    let setting = br#"{"kind":"policy","number":33}"#;
    let named = joined(&[br#"{"name":"lean-policy","#, &POLICY[1..]]);
    for (body, name) in [(POLICY, None), (named.as_ref(), Some(b"lean-policy".as_slice()))] {
        let items = file::decode(&kind_file(setting, body), &limits).unwrap();
        let item = items.get(0).unwrap();
        let c::SettingEdit::Set(set) = &item.edit else { panic!("set policy") };
        let c::SettingValue::Policy(value) = &set.value else { panic!("policy") };
        assert_eq!(value.name.as_deref(), name);
        let bounds = JsonLimits { depth: 32, length: 8192 };
        let mut measure = Encoder::measure(&bounds);
        encode_policy_file(item.setting, &set.value, &mut measure);
        let mut out = Encoder::write(measure.measured().unwrap(), &bounds);
        encode_policy_file(item.setting, &set.value, &mut out);
        let printed = out.finish();
        assert_eq!(skein_lib::bytes::find(&printed, br#""name":"lean-policy""#).is_some(), name.is_some());
        assert_eq!(file::decode(&printed, &limits).unwrap(), items);
    }
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
            w::RevisionReplaced::TooLarge(w::Size { bytes: 1_000_000 }),
            c::RevisionReplaced::TooLarge(c::Size { bytes: 1_000_000 }),
        ),
        (
            w::RevisionReplaced::Application(w::ApplicationSize { size: 12 }),
            c::RevisionReplaced::Application { size: 12 },
        ),
        (w::RevisionReplaced::Default, c::RevisionReplaced::Default),
        (w::RevisionReplaced::Removed, c::RevisionReplaced::Removed),
        (w::RevisionReplaced::NotOffered, c::RevisionReplaced::NotOffered),
    ];
    for (replaced, expected) in cases {
        let item = w::RevisionItem { setting: w::Setting::Rules, edit: w::RevisionEdit::Reset, replaced };
        let restored = crate::wire::settings_restore_revision_item(&item, &limits).unwrap();
        assert_eq!(restored.replaced, expected);
    }
}

const TEMPLATE_PREFIX: &[u8] = br#"{"items":[{"setting":{"kind":"template","number":4},"set":{"number":4,"shown":"draft","executor":{"charter":1},"contract":{"report":{"words":1024}},"shallower":1,"budget":"given","deadline":"none","tracked":1,"flags":{"conversation":false,"listed":true,"retention":"kept","ended_by":"result"},"label":1,"grace":null,"llm":"#;
fn template_file(resources: &[u8]) -> Box<[u8]> {
    joined(&[TEMPLATE_PREFIX, b"null", resources, SUFFIX])
}
fn template(bytes: &[u8], limits: &Limits) -> c::Template {
    let items = file::decode(bytes, limits).expect("template file");
    let c::SettingEdit::Set(set) = &items.get(0).unwrap().edit else { panic!("set") };
    let c::SettingValue::Template(value) = &set.value else { panic!("template") };
    *value.clone()
}
const TEMPLATE_SETTING: &[u8] = br#"{"kind":"template","number":33}"#;
const TEMPLATE_VALUE: &[u8] = br#"{"number":33,"executor":{"charter":1},"contract":{"report":{"words":1024}},"shallower":1,"budget":"given","deadline":"none","tracked":1,"flags":{"conversation":false,"listed":true,"retention":"kept","ended_by":"result"},"label":1,"grace":null,"llm":null,"#;

#[test]
fn templates_read_both_display_forms_and_print_shown() {
    let limits = Limits::standard();
    let bounds = JsonLimits { depth: 32, length: 8192 };
    for member in [
        br#""shown":"goal for a lean design"}"#.as_slice(),
        br#""name":"goal for a lean design"}"#,
        br#""name":{"bytes":"goal for a lean design","cut":false}}"#,
    ] {
        let file = kind_file(TEMPLATE_SETTING, &joined(&[TEMPLATE_VALUE, member]));
        let value = template(&file, &limits);
        assert_eq!(value.shown.bytes(), b"goal for a lean design");
        assert_eq!(value.name, None);
        let mut measure = Encoder::measure(&bounds);
        values::encode_template(&value, &mut measure);
        let mut out = Encoder::write(measure.measured().unwrap(), &bounds);
        values::encode_template(&value, &mut out);
        let printed = out.finish();
        assert!(skein_lib::bytes::find(&printed, br#""shown":"goal for a lean design""#).is_some());
        assert!(skein_lib::bytes::find(&printed, br#""name""#).is_none());
        assert_eq!(template(&kind_file(TEMPLATE_SETTING, &printed), &limits), value);
    }
}

#[test]
fn a_template_with_shown_reads_and_prints_its_separate_name() {
    let limits = Limits::standard();
    let named = br#""name":"lean-goal","shown":"goal for a lean design"}"#;
    let file = kind_file(TEMPLATE_SETTING, &joined(&[TEMPLATE_VALUE, named]));
    let value = template(&file, &limits);
    assert_eq!(value.name.as_deref(), Some(b"lean-goal".as_slice()));
    assert_eq!(value.shown.bytes(), b"goal for a lean design");
    let bounds = JsonLimits { depth: 32, length: 8192 };
    let mut measure = Encoder::measure(&bounds);
    values::encode_template(&value, &mut measure);
    let mut out = Encoder::write(measure.measured().unwrap(), &bounds);
    values::encode_template(&value, &mut out);
    let printed = out.finish();
    assert!(skein_lib::bytes::find(&printed, br#""name":"lean-goal","shown":"goal for a lean design""#).is_some());
    assert_eq!(template(&kind_file(TEMPLATE_SETTING, &printed), &limits), value);
}

#[test]
fn a_charter_overlay_refuses_a_name_member() {
    let file = br#"{"items":[{"setting":{"kind":"charter","number":1},"overlay":{"name":"a-name"}}]}"#;
    assert_eq!(
        file::decode(file, &Limits::standard()).unwrap_err().as_ref(),
        b"settings file item 1: name: unknown member"
    );
}

#[test]
fn a_settings_file_template_names_its_resources() {
    let limits = Limits::standard();
    let value = template(&template_file(br#", "resources":[{"connector":1,"symbol":1}]"#), &limits);
    assert_eq!(value.resources.as_slice(), &[c::Symbol { connector: 1, symbol: 1 }]);
    let wire = crate::wire::settings_wire_template(&value, &limits.settings).unwrap();
    assert_eq!(wire.resources.get(0).unwrap().connector, 1);
    assert_eq!(wire.resources.get(0).unwrap().symbol, 1);
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
    let wire = crate::wire::settings_wire_charter(&source);
    assert_eq!(wire.tools, 1536);
    let restored = crate::wire::settings_restore_charter(&wire).unwrap();
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
const CLASSES: &[u8] = br#"{"items":[{"kind":"findings","words":"","class":"decide"},{"kind":"tests","words":"Keep coverage","class":"ask"},{"set":1,"words":"","class":"ask-away"}],"sets":[{"number":1,"name":"Core","patterns":["src/**","Cargo.*"]}]}"#;
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
        let mut bytes = Writer::new(
            usize::try_from(record.measure(&temper_client_protocol_wire::v1::CEILINGS).expect("within ceilings"))
                .expect("record length"),
        );
        record.write(&temper_client_protocol_wire::v1::CEILINGS, &mut bytes).expect("measured record");
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
    for route in [b"\"none\"".as_slice(), b"\"up\""] {
        file::decode(&kind_file(br#"{"kind":"project-questions","project":7}"#, route), &Limits::standard())
            .expect("valid route");
    }
    for value in [b"\"foreman\"".as_slice(), b"\"other\"", b"true", b"{}"] {
        let why = file::decode(&kind_file(br#"{"kind":"project-questions","project":7}"#, value), &Limits::standard())
            .expect_err("malformed route");
        assert!(skein_lib::bytes::find(&why, b"project-questions").is_some());
    }
}

const MODEL_SETTING: &[u8] = br#"{"kind":"model","number":4}"#;
const MODEL_VALUE: &[u8] = br#"{"number":4,"model":{"provider":2,"dialect":1,"endpoint":3,"program":null,"name":"gpt-6.1-terra","window":131072,"output":8192,"input_price":3,"cached_price":1,"cache_write_price":null,"output_price":12,"price_unit":1000000,"efforts":["high"],"effort":"high"},"shown":"GPT-6.1 Terra","description":"For careful work","picker":false,"withdrawn":false,"source":"stock","estimated":false}"#;

#[test]
fn the_settings_form_round_trips_a_model() {
    let bytes = kind_file(MODEL_SETTING, MODEL_VALUE);
    let items = file::decode(&bytes, &Limits::standard()).expect("whole model file");
    let item = items.get(0).expect("model item");
    assert_eq!(item.setting, c::Setting::Model(c::SettingNumber { number: 4 }));
    let c::SettingEdit::Set(set) = &item.edit else { panic!("model set") };
    let c::SettingValue::Model(model) = &set.value else { panic!("model value") };
    assert_eq!(model.shown.as_ref().expect("shown name").bytes(), b"GPT-6.1 Terra");
    assert_eq!(model.model.input_price, 3);
    assert!(model.model.speeds.is_empty());
    assert_eq!(model.model.speed, None);
    assert!(!model.picker);
    let document = c::ReadValue::Setting(Box::new(c::SettingDocument {
        overlay: None,
        setting: item.setting,
        revision: 2,
        value: Some(set.value.clone()),
        application: None,
        proposed: None,
    }));
    assert_eq!(
        shown(&document),
        joined(&[br#"{"setting":"#, MODEL_SETTING, br#","revision":2,"value":"#, MODEL_VALUE, b"}"])
    );
    assert_eq!(
        file::proposal_items(
            &joined(&[
                br#"{"against":1,"reason":"New prices", "items":[{"setting":"#,
                MODEL_SETTING,
                br#","set":"#,
                MODEL_VALUE,
                b"}]}"
            ]),
            &Limits::standard()
        )
        .expect("inline proposal"),
        items
    );
}

#[test]
fn the_settings_form_round_trips_declared_speeds_and_the_default() {
    const SPEEDS: &[u8] = br#""speeds":[{"name":"fast","also":["priority"],"input_price":4000,"cached_price":200,"cache_write_price":5000,"output_price":20000},{"name":"ultrafast","also":null,"input_price":12000,"cached_price":600,"cache_write_price":15000,"output_price":60000},{"name":"turbo","input_price":20000,"cached_price":900,"output_price":90000}],"speed":"ultrafast""#;
    let after_model = br#"},"shown":"GPT-6.1 Terra","description":"For careful work","picker":false,"withdrawn":false,"source":"stock","estimated":false}"#;
    let prefix = MODEL_VALUE.strip_suffix(after_model).unwrap();
    let value = joined(&[prefix, b",", SPEEDS, after_model]);
    let limits = Limits::standard();
    let items = file::decode(&kind_file(MODEL_SETTING, &value), &limits).unwrap();
    let item = items.get(0).unwrap();
    let c::SettingEdit::Set(set) = &item.edit else { panic!("model set") };
    let c::SettingValue::Model(model) = &set.value else { panic!("model value") };
    let [fast, ultrafast, turbo] = model.model.speeds.as_slice() else { panic!("three speeds") };
    assert_eq!(fast.name.bytes(), b"fast");
    assert_eq!(fast.also.get(0).unwrap().bytes(), b"priority");
    assert_eq!(
        (fast.input_price, fast.cached_price, fast.cache_write_price, fast.output_price),
        (4000, 200, Some(5000), 20_000)
    );
    assert_eq!(ultrafast.name.bytes(), b"ultrafast");
    assert!(ultrafast.also.is_empty());
    assert_eq!(
        (ultrafast.input_price, ultrafast.cached_price, ultrafast.cache_write_price, ultrafast.output_price),
        (12_000, 600, Some(15_000), 60_000)
    );
    assert_eq!(turbo.name.bytes(), b"turbo");
    assert!(turbo.also.is_empty());
    assert_eq!(turbo.cache_write_price, None);
    assert_eq!(model.model.speed.as_ref().unwrap().bytes(), b"ultrafast");
    let document = c::ReadValue::Setting(Box::new(c::SettingDocument {
        overlay: None,
        setting: item.setting,
        revision: 2,
        value: Some(set.value.clone()),
        application: None,
        proposed: None,
    }));
    let printed = shown(&document);
    let prefix = joined(&[br#"{"setting":"#, MODEL_SETTING, br#","revision":2,"value":"#]);
    let value = printed.strip_prefix(prefix.as_ref()).unwrap().strip_suffix(b"}").unwrap();
    assert_eq!(file::decode(&kind_file(MODEL_SETTING, value), &limits).unwrap(), items);
    let proposal = joined(&[
        br#"{"against":1,"reason":"New speeds","items":[{"setting":"#,
        MODEL_SETTING,
        br#","set":"#,
        value,
        b"}]}",
    ]);
    assert_eq!(file::proposal_items(&proposal, &limits).unwrap(), items);
    let wire = crate::wire::settings_wire_setting_model(&model.model);
    assert_eq!(crate::wire::settings_restore_setting_model(&wire).unwrap(), model.model);
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
        b"null",
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
        assert_eq!(record.source.repository.forge, 1);
        assert_eq!(record.source.repository.repository, 12);
        assert_eq!(record.target.repository.repository, 13);
        assert_eq!(&record.start, &w::FollowStart::Owner);
        assert_eq!(record.source.paths.is_empty(), optional);
        assert_eq!(record.source.branch.is_empty(), optional);
        assert_eq!(record.producer.is_none(), optional);
        assert_eq!(record.target.branch.len(), 0);
        let document = c::ReadValue::Setting(Box::new(c::SettingDocument {
            overlay: None,
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

#[test]
fn follow_owners_accept_one_person_or_task_and_render_the_same_variant() {
    use temper_client_protocol_wire::v1 as w;
    let selector = br#"{"kind":"follow","project":2,"number":1}"#;
    let prefix = br#"{"source":{"forge":1,"repository":12},"target":{"forge":1,"repository":13},"owner":"#;
    let suffix = br#", "check":{"workflow":"tests.yml","artifact":"results","format":"lines"}}"#;
    for (owner, expected) in [
        (br#"{"task":52}"#.as_slice(), w::FollowOwner::Task(w::FollowOwnerTask { task: 52 })),
        (br#"{"person":4}"#.as_slice(), w::FollowOwner::Person(w::FollowPerson { person: 4 })),
    ] {
        let items = file::decode(&kind_file(selector, &joined(&[prefix, owner, suffix])), &Limits::standard()).unwrap();
        let item = items.get(0).unwrap();
        let c::SettingEdit::Set(set) = &item.edit else { panic!("set") };
        let c::SettingValue::Application(bytes) = &set.value else { panic!("follow bytes") };
        let record = w::Follow::decode(&w::CEILINGS, &mut skein_lib::Reader::new(bytes)).unwrap();
        assert_eq!(&record.owner, &expected);
        let text = shown(&c::ReadValue::Setting(Box::new(c::SettingDocument {
            overlay: None,
            setting: item.setting,
            revision: 1,
            value: Some(set.value.clone()),
            application: None,
            proposed: None,
        })));
        assert!(skein_lib::bytes::find(&text, &joined(&[br#""owner":"#, owner])).is_some());
    }
    for owner in [br#"{"person":4,"task":52}"#.as_slice(), b"{}", br#"{"role":3}"#] {
        file::decode(&kind_file(selector, &joined(&[prefix, owner, suffix])), &Limits::standard()).unwrap_err();
    }
}

#[test]
fn settings_files_refuse_the_retired_questions_route_by_name() {
    let why =
        file::decode(&kind_file(br#"{"kind":"project-questions","project":2}"#, br#""foreman""#), &Limits::standard())
            .expect_err("retired route");
    assert_eq!(why.as_ref(), b"settings file item 1: project-questions: none or up required");
}

#[test]
fn dependencies_round_trip_and_require_a_numeric_project() {
    let selector = br#"{"kind":"dependency","project":3,"number":1}"#;
    let json = br#"{"project":7}"#;
    let items = file::decode(&kind_file(selector, json), &Limits::standard()).unwrap();
    let item = items.get(0).unwrap();
    assert_eq!(item.setting, c::Setting::Application { kind: 13, scope: 3, number: 1 });
    let c::SettingEdit::Set(set) = &item.edit else { panic!("set") };
    let document = c::ReadValue::Setting(Box::new(c::SettingDocument {
        overlay: None,
        setting: item.setting,
        revision: 1,
        value: Some(set.value.clone()),
        application: None,
        proposed: None,
    }));
    assert_eq!(shown(&document), joined(&[br#"{"setting":"#, selector, br#","revision":1,"value":"#, json, b"}"]));
    for (json, reason) in
        [(b"{}".as_slice(), b"missing member".as_slice()), (br#"{"project":"7"}"#, b"expected unsigned integer")]
    {
        assert_eq!(
            file::decode(&kind_file(selector, json), &Limits::standard()).unwrap_err(),
            joined(&[b"settings file item 1: dependency.project: ", reason])
        );
    }
}

#[test]
fn neutral_limits_keep_the_wire_value_and_render_one_line_per_override() {
    let limits = Limits::standard();
    let value = c::SettingValue::Limits(Box::new([
        c::Override { name: b"tasks.depth".as_slice().into(), value: 0, reason: b"a reason".as_slice().into() },
        c::Override { name: b"tasks.tasks".as_slice().into(), value: u64::MAX, reason: "理由".as_bytes().into() },
    ]));
    let wire = crate::wire::settings_wire_setting_value(&value, &limits.settings).unwrap();
    let restored = crate::wire::settings_restore_setting_value(&wire, &limits).unwrap();
    assert_eq!(restored, value);
    let mut measured = crate::Text::measure();
    crate::render::value_inline(&restored, &limits, &mut measured);
    let mut text = crate::Text::writing_exact(measured);
    crate::render::value_inline(&restored, &limits, &mut text);
    assert_eq!(
        text.finish().as_ref(),
        "tasks.depth = 0  a reason\ntasks.tasks = 18446744073709551615  理由".as_bytes()
    );
    let bounds = JsonLimits { depth: 8, length: 1024 };
    let mut measured = Encoder::measure(&bounds);
    crate::render::encode_value(&restored, &mut measured);
    let mut out = Encoder::write(measured.measured().unwrap(), &bounds);
    crate::render::encode_value(&restored, &mut out);
    assert_eq!(
        out.finish().as_ref(),
        "[{\"name\":\"tasks.depth\",\"value\":0,\"reason\":\"a reason\"},{\"name\":\"tasks.tasks\",\"value\":18446744073709551615,\"reason\":\"理由\"}]".as_bytes()
    );
}

#[test]
#[expect(clippy::disallowed_methods, reason = "read fixture JSON and rendered error text in the form test")]
fn agents_form_round_trips_named_maps_and_reports_duplicates() {
    let setting = br#"{"kind":"agents"}"#;
    let json = br#"{"profile":{"policy.max_turns":900,"declared.memory":18446744073709551615},"endpoints":{"openai":{"window":1048576,"output":8192,"reasoning_item":65536,"head_ms":60000,"idle_ms":300000,"oversized":"fail"}},"fallback":{"window":1,"output":1,"reasoning_item":1,"head_ms":1,"idle_ms":1,"oversized":"drop"}}"#;
    let items = file::decode(&kind_file(setting, json), &Limits::standard()).unwrap();
    let item = items.get(0).unwrap();
    assert_eq!(item.setting, c::Setting::Application { kind: 21, scope: 0, number: 1 });
    let c::SettingEdit::Set(set) = &item.edit else { panic!("set") };
    let document = c::ReadValue::Setting(Box::new(c::SettingDocument {
        overlay: None,
        setting: item.setting,
        revision: 1,
        value: Some(set.value.clone()),
        application: None,
        proposed: None,
    }));
    assert_eq!(shown(&document), joined(&[br#"{"setting":"#, setting, br#","revision":1,"value":"#, json, b"}"]));
    for (from, to, name) in [
        ("\"policy.max_turns\":900", "\"policy.max_turns\":900,\"policy.max_turns\":901", "policy.max_turns"),
        ("\"openai\":{", "\"openai\":{},\"openai\":{", "openai"),
        ("\"output\":8192", "\"output\":4294967296", "output"),
    ] {
        let bad = core::str::from_utf8(json).unwrap().replace(from, to);
        let why = file::decode(&kind_file(setting, bad.as_bytes()), &Limits::standard()).unwrap_err();
        assert!(core::str::from_utf8(&why).unwrap().contains(name));
    }
    let page = crate::fields::page(&Limits::standard());
    assert!(
        core::str::from_utf8(&page)
            .unwrap()
            .contains("Agent profile and endpoint limits, taken at each worker's next start.")
    );
}

#[test]
fn both_limit_kinds_use_a_whole_list_of_named_values_and_reasons() {
    let limits = Limits::standard();
    let list = br#"[{"name":"forge.entries","value":16,"reason":"dev's outbox"}]"#;
    for (kind, setting) in [
        (br#"{"kind":"limits"}"#.as_slice(), c::Setting::Limits),
        (br#"{"kind":"temper-limits"}"#, c::Setting::Application { kind: 22, scope: 0, number: 1 }),
    ] {
        let bytes = kind_file(kind, list);
        let items = file::decode(&bytes, &limits).expect("whole override list");
        let item = items.as_slice().first().unwrap();
        assert_eq!(item.setting, setting);
        let c::SettingEdit::Set(set) = &item.edit else { panic!("set list") };
        assert_eq!(
            crate::render::limit_values(setting, &set.value).unwrap().as_ref(),
            &[c::Override {
                name: b"forge.entries".as_slice().into(),
                value: 16,
                reason: b"dev's outbox".as_slice().into(),
            }]
        );
        let shown = shown(&c::ReadValue::Setting(Box::new(c::SettingDocument {
            setting,
            revision: 1,
            value: Some(set.value.clone()),
            overlay: None,
            application: None,
            proposed: None,
        })));
        assert!(skein_lib::bytes::find(&shown, list).is_some(), "{shown:?}");
        assert!(file::decode(&kind_file(kind, b"{}"), &limits).is_err(), "requires list");
    }
    for kind in [br#"{"kind":"limits","number":1}"#.as_slice(), br#"{"kind":"temper-limits","project":1}"#] {
        assert!(file::decode(&kind_file(kind, list), &limits).is_err(), "unnumbered deployment kind");
    }
}

#[test]
fn provider_files_proposals_and_reads_keep_the_whole_value_and_number() {
    const SETTING: &[u8] = br#"{"kind":"provider","number":3}"#;
    const VALUE: &[u8] = br#"{"number":3,"name":"local","endpoint":"local","signed_in_plan":false,"sources":{},"endpoints":[{"name":"local","number":5,"dialect":3,"address":{"socket":"127.0.0.1:8080"},"transport":"plaintext","headers":[],"connect_ms":10000,"handshake_ms":10000}]}"#;
    let limits = Limits::standard();
    let items = file::decode(&kind_file(SETTING, VALUE), &limits).unwrap();
    let item = items.get(0).unwrap();
    assert_eq!(item.setting, c::Setting::Provider(c::SettingNumber { number: 3 }));
    let c::SettingEdit::Set(set) = &item.edit else { panic!("set") };
    let c::SettingValue::Provider(value) = &set.value else { panic!("provider") };
    assert_eq!(value.provider.number, 3);
    assert_eq!(value.provider.name.as_ref(), b"local");
    let endpoint = value.endpoints.first().unwrap();
    assert_eq!(endpoint.name.as_ref(), b"local");
    assert_eq!(endpoint.number, 5);
    assert_eq!(endpoint.provider.as_ref(), b"local");
    assert_eq!(endpoint.connect.as_millis(), 10000);
    let mut measured = Encoder::measure(&skein_json::writer::Limits { depth: 32, length: 32768 });
    crate::render::encode_value(&set.value, &mut measured);
    let mut out =
        Encoder::write(measured.measured().unwrap(), &skein_json::writer::Limits { depth: 32, length: 32768 });
    crate::render::encode_value(&set.value, &mut out);
    assert_eq!(file::decode(&kind_file(SETTING, &out.finish()), &limits).unwrap(), items);
    let shown = shown(&c::ReadValue::Setting(Box::new(c::SettingDocument {
        setting: item.setting,
        revision: 2,
        value: Some(set.value.clone()),
        overlay: None,
        application: None,
        proposed: None,
    })));
    assert!(skein_lib::bytes::find(&shown, VALUE).is_some());
    let proposal = joined(&[
        br#"{"against":1,"reason":"Add provider","items":[{"setting":"#,
        SETTING,
        br#", "set":"#,
        VALUE,
        b"}]}",
    ]);
    assert_eq!(file::proposal_items(&proposal, &limits).unwrap(), items);
    let mismatch = joined(&[br#"{"number":4,"name":"local","sources":{},"endpoints":[]}"#]);
    let why = file::decode(&kind_file(SETTING, &mismatch), &limits).unwrap_err();
    assert!(skein_lib::bytes::find(&why, b"number").is_some(), "{why:?}");
    for (edit, expected) in [
        (br#""remove":true"#.as_slice(), c::SettingEdit::Removed),
        (br#""reset":true"#, c::SettingEdit::Reset),
        (br#""edit":"removed""#, c::SettingEdit::Removed),
        (br#""edit":"reset""#, c::SettingEdit::Reset),
    ] {
        let bytes = joined(&[br#"{"items":[{"setting":"#, SETTING, b",", edit, b"}]}"]);
        let items = file::decode(&bytes, &limits).unwrap();
        assert_eq!(items.get(0).unwrap(), &c::SettingItem { setting: item.setting, edit: expected });
    }
    let read = joined(&[br#"{"setting":"#, SETTING, b"}"]);
    assert_eq!(file::read_setting(&read, &limits).unwrap(), item.setting);
    let page = crate::fields::page(&limits);
    assert!(skein_lib::bytes::find(&page, b"LLM provider/endpoints; next engine start").is_some());
}

#[test]
fn worker_files_proposals_and_reads_keep_the_digest_as_lowercase_hex() {
    const SETTING: &[u8] = br#"{"kind":"worker","number":2}"#;
    const DIGEST: &[u8] = b"abababababababababababababababababababababababababababababababab";
    for (value, digest) in [
        (
            br#"{"name":"w1","places":4,"digest":"abababababababababababababababababababababababababababababababab"}"#
                .as_slice(),
            Some([0xab; 32]),
        ),
        (br#"{"name":"w1","places":4}"#.as_slice(), None),
    ] {
        let items = file::decode(&kind_file(SETTING, value), &Limits::standard()).expect("worker file");
        let item = items.get(0).unwrap();
        assert_eq!(item.setting, c::Setting::Worker(c::SettingNumber { number: 2 }));
        let c::SettingEdit::Set(set) = &item.edit else { panic!("set") };
        assert_eq!(
            set.value,
            c::SettingValue::Worker(Box::new(c::WorkerSetting {
                number: 2,
                name: b"w1".as_slice().into(),
                places: 4,
                digest,
                withdrawn: false
            }))
        );
        let shown = shown(&c::ReadValue::Setting(Box::new(c::SettingDocument {
            overlay: None,
            setting: item.setting,
            revision: 2,
            value: Some(set.value.clone()),
            application: None,
            proposed: None,
        })));
        assert!(skein_lib::bytes::find(&shown, SETTING).is_some());
        assert_eq!(skein_lib::bytes::find(&shown, DIGEST).is_some(), digest.is_some());
        let proposal = joined(&[
            br#"{"against":1,"reason":"Add worker","items":[{"setting":"#,
            SETTING,
            br#", "set":"#,
            value,
            b"}]}",
        ]);
        assert_eq!(file::proposal_items(&proposal, &Limits::standard()).unwrap(), items);
    }
    for bad in
        [br#""ab""#.as_slice(), br#""ABABABABABABABABABABABABABABABABABABABABABABABABABABABABABABABAB""#, br#""zz""#]
    {
        file::decode(
            &kind_file(SETTING, &joined(&[br#"{"name":"w1","places":4,"digest":"#, bad, b"}"])),
            &Limits::standard(),
        )
        .expect_err("malformed digest");
    }
}

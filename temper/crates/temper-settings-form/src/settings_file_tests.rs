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
    let bytes = include_bytes!("../../temper/tests/fixtures/cluster/settings.json");
    let items = file::decode(bytes, &Limits::standard()).expect("retained cluster settings file");
    assert_eq!(items.len(), 3);
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

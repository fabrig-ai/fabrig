//! Settings keep byte-summed verdict rules at the deployment's scale (protocol/store/records.md, 1).
use super::fixtures;
use crate::{codec::settings, settings_v2};
use jig_domain_authority as authority;
use jig_domain_core::{self as core, settings as domain};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::Duration;

const RETRY: tasks::Retry = tasks::Retry { retries: 3, base: Duration::from_secs(1), max: Duration::from_secs(30) };
const RETRIES: tasks::Retries =
    tasks::Retries { transient: RETRY, permanent: RETRY, run: RETRY, agent: RETRY, lost: RETRY, invalid: RETRY, infrastructure: RETRY };
const SMALL_TASKS: tasks::Limits = tasks::Limits {
    charter_parts: 8,
    placed_holds: 4,
    project_placed_holds: 4,
    party_placed_holds: 2,
    exemptions: 8,

    tasks: 8,
    funders: 8,
    project_tasks: 8,
    project_goals: 8,
    tree_tasks: 32,
    depth: 8,
    delegates: 4,
    references: 8,
    subscriptions: 4,
    batch: 2,
    // A batch's bytes: two of small's largest tasks, one of them recurring with a template of two,
    // hold under 90 KiB; 128 KiB.
    batch_bytes: 131_072,
    circle: 8,
    dependencies: 8,
    holdings: 4,
    hold_kinds: 8,
    resource_reports: 128,
    pools: 8,
    hold_segments: 4,
    hold_bytes: 1_024,
    hold_waiters: 8,
    grace: Duration::from_secs(600),
    hold_wait: Duration::from_secs(120),
    inputs: 8,
    spec_bytes: 4_096,
    parameters: 4,
    result_bytes: 512,
    run_inbox: 8,
    inbox_messages: 8,
    inbox_bytes: 4_096,
    message_bytes: 4_096,
    person_stall: Duration::from_secs(3600),
    proposal_stall: Duration::from_secs(3600),
    escalation_stall: Duration::from_secs(3600),
    saved_resources: 4,
    contract_choices: 1,
    charters: 4,
    charter_documents: 2,
    authority_grants: 4,
    authority_segments: 4,
    authority_bytes: 2_048,
    executor_kinds: 4,
    retries: RETRIES,
    facts: 32,
    room_refusals: 8,
};

// Small's inputs used by settings' limits and revision pricing; unrelated children stay tiny.
fn scale_core() -> core::Limits {
    let mut core = fixtures::CORE_LIMITS;
    core.run_bytes = 16_384;
    core.policy_bytes = 16_384;
    core.documents.text_bytes = 4096;
    core.authority = authority::Limits {
        projects: 2,
        roles: 4,
        requirements: 4,
        facts: 16,
        grants: 16,
        executors: 16,
        segments: 8,
        segment_bytes: 256,
        implications: 8,
        batch: 2,
        accounts: 4,
        writes: 4,
    };
    core.tasks = tasks::Limits {
        tasks: 128,
        project_tasks: 128,
        project_goals: 80,
        tree_tasks: 384,
        depth: 32,
        delegates: 80,
        batch: 160,
        batch_bytes: 4_194_304,
        inbox_messages: 82,
        run_inbox: 82,
        inbox_bytes: 21_495_808,
        spec_bytes: 131_072,
        result_bytes: 262_144,
        message_bytes: 262_144,
        contract_choices: 4,
        charters: 12,
        authority_grants: 16,
        authority_segments: 8,
        hold_segments: 8,
        executor_kinds: 16,
        ..SMALL_TASKS
    };
    core
}
fn limits() -> settings_v2::Limits {
    settings::limits(&scale_core()).expect("K2 settings fit the schema")
}
fn row(fields: Box<[tasks::FieldRule]>) -> domain::Record {
    let template = core::Template {
        number: 1,
        name: Box::from(b"template".as_slice()),
        executor: people::TemplateExecutor::Chosen,
        contract: tasks::Contract::Verdict {
            choices: Box::new([tasks::Verdict { code: 1, words: 0, followups: 0, fields }]),
        },
        shallower: 0,
        budget: people::TemplateBudget::Given,
        deadline: people::TemplateDeadline::None,
        tracked: None,
        flags: tasks::Flags::PLAIN,
        label: 0,
        grace: None,
        llm: None,
        resources: Box::new([]),
    };
    domain::Record::Default {
        setting: domain::Setting::Template(1),
        default: domain::Default { value: domain::Value::Template(Box::new(template)), revision: 1 },
    }
}
fn boundary_row(long: usize) -> domain::Record {
    row(Box::new([
        tasks::FieldRule { name: skein_lib::bytes::zeroed(long), max: 0 },
        tasks::FieldRule { name: Box::new([b'b'; 2]), max: 0 },
        tasks::FieldRule { name: Box::new([b'c'; 2]), max: 0 },
    ]))
}
#[test]
fn settings_hold_fabrig_scale() {
    let limits = limits();
    assert_eq!(limits.field_rule_name, 262_144);
    assert_eq!(limits.approval_state, 262_144);
    assert_eq!(limits.verdict_fields, tasks::VERDICT_FIELDS * 12 + 262_144);
}
#[test]
fn a_verdict_round_trips_as_pieces() {
    let limits = limits();
    let row = boundary_row(262_140);
    let bytes = settings::encode(&row, &limits, &[]).expect("names sum to exactly result_bytes");
    assert_eq!(settings::decode(&row.key(), &bytes, &limits, &[]), Ok(row));
    let past = boundary_row(262_141);
    assert_eq!(settings::encode(&past, &limits, &[]).unwrap_err().reason, skein_codec::Reason::Bound);
    let mut core = scale_core();
    core.tasks.result_bytes += 1;
    let wider = settings::limits(&core).unwrap();
    let bytes = settings::encode(&past, &wider, &[]).unwrap();
    assert_eq!(settings::decode(&past.key(), &bytes, &limits, &[]).unwrap_err().reason, skein_codec::Reason::Bound);
}
#[test]
fn settings_refuse_a_result_past_their_ceiling() {
    let mut core = scale_core();
    core.tasks.result_bytes = 524_288;
    assert!(settings::limits(&core).is_some());
    core.tasks.result_bytes = 524_289;
    assert!(settings::limits(&core).is_none());
}

// Generated types price each piece; the typed codec also checks the domain's count and sum.
fn wire_row(fields: &[tasks::FieldRule], limits: &settings_v2::Limits) -> Box<[u8]> {
    use skein_lib::{List, Reader, Writer};
    let empty = row(Box::new([]));
    let bytes = settings::encode(&empty, limits, &[]).unwrap();
    let mut default = settings_v2::Default::decode(limits, &mut Reader::new(&bytes)).unwrap().into_parts();
    let settings_v2::Value::Template(template) = default.value else { unreachable!("template fixture") };
    let mut template = template.into_parts();
    let mut length = 0_usize;
    for field in fields {
        length = length.checked_add(field.name.len()).unwrap().checked_add(12).unwrap();
    }
    let mut pieces = Writer::new(length);
    for field in fields {
        let field = settings_v2::FieldRule::new(
            limits,
            settings_v2::FieldRuleParts { name: field.name.clone(), max: field.max },
        )
        .unwrap();
        let mut item = Writer::new(usize::try_from(field.measure()).unwrap());
        field.encode(&mut item).unwrap();
        skein_codec::piece::write(&mut pieces, &item.finish()).unwrap();
    }
    let verdict = settings_v2::Verdict::new(
        limits,
        settings_v2::VerdictParts { code: 1, words: 0, followups: 0, fields: pieces.finish() },
    )
    .unwrap();
    let mut choices = List::with_capacity(1);
    choices.push(verdict).unwrap();
    template.contract = settings_v2::Contract::Verdict(
        settings_v2::VerdictChoices::new(limits, settings_v2::VerdictChoicesParts { choices }).unwrap(),
    );
    default.value = settings_v2::Value::Template(Box::new(settings_v2::Template::new(limits, template).unwrap()));
    let default = settings_v2::Default::new(limits, default).unwrap();
    let mut bytes = Writer::new(usize::try_from(default.measure()).unwrap());
    default.encode(&mut bytes).unwrap();
    bytes.finish()
}

#[test]
fn verdict_pieces_enforce_the_rule_count_nonempty_names_and_max_sum() {
    let limits = limits();
    for (count, name_bytes, max, passes) in [
        (0_u32, 0, 0, true),
        (512, 1, 0, true),
        (513, 1, 0, false),
        (1, 0, 0, false),
        (1, 1, 262_143, true),
        (1, 1, 262_144, false),
    ] {
        let mut fields = skein_lib::List::with_capacity(count);
        for _ in 0..count {
            fields.push(tasks::FieldRule { name: skein_lib::bytes::zeroed(name_bytes), max }).unwrap();
        }
        let fields = fields.into_boxed();
        let forged = wire_row(&fields, &limits);
        let row = row(fields);
        let encoded = settings::encode(&row, &limits, &[]);
        let decoded = settings::decode(&row.key(), &forged, &limits, &[]);
        if passes {
            assert_eq!(encoded.unwrap(), forged);
            assert_eq!(decoded, Ok(row));
        } else {
            assert_eq!(encoded.unwrap_err().reason, skein_codec::Reason::Bound);
            assert_eq!(decoded.unwrap_err().reason, skein_codec::Reason::Bound);
        }
    }
}

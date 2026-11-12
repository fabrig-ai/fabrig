//! Byte-summed pieces preserve the domain's admissions (skein/docs/design/data/codec/README.md, 3).
use super::fixtures;
use crate::codec::{people, tasks};
use crate::{people_v14, tasks_v10};
use jig_domain_people as person;
use jig_domain_tasks as task;
use skein_lib::{List, Reader, Writer};

fn people_limits() -> people_v14::Limits {
    let mut core = fixtures::CORE_LIMITS;
    core.people.amendment_bytes = 8192;
    core.policy_bytes = 8192;
    people::limits(&core).expect("test-sized byte sums")
}
fn pattern(segments: usize) -> person::Pattern {
    let mut values: List<Box<[u8]>> = List::with_capacity(u32::try_from(segments).expect("bounded count"));
    for _ in 0..segments {
        values.push(Box::new([])).expect("same count");
    }
    person::Pattern { segments: values.into_boxed(), last: person::Last::Exact(Box::new([])) }
}
fn amendment() -> person::Amendment {
    let mut value = fixtures::people_amendment();
    value.reason = Box::new([]);
    value.instructions = None;
    value.spec = None;
    value.dependencies = None;
    let authority = value.authority.as_mut().expect("fixture authority");
    authority.grants = Box::new([person::Grant { connector: 1, kind: 2, pattern: pattern(509) }]);
    authority.delegation.kinds = Box::new([]);
    authority.note_resources = Box::new([]);
    value
}

#[test]
fn the_pattern_piece_admits_512_segments_and_refuses_513() {
    let limits = people_limits();
    for (label, segments, passes) in [("at_sum", 512, true), ("past_sum", 513, false)] {
        let value = pattern(segments);
        let encoded = people::policy::encode_pattern(&value, &limits);
        if passes {
            let encoded = encoded.expect(label);
            let mut out = Writer::new(skein_lib::length::usize_of(encoded.measure()));
            encoded.encode(&mut out).expect("measured pattern");
            let bytes = out.finish();
            let decoded = people_v14::Pattern::decode(&limits, &mut Reader::new(&bytes)).expect("encoded pattern");
            assert_eq!(people::policy::decode_pattern(decoded, &limits), Ok(value), "{label}");
        } else {
            assert_eq!(encoded, Err(people_problem()), "{label}");
        }
    }
}

#[test]
fn a_whole_amendment_admits_its_full_sum_and_refuses_one_byte_past_it() {
    let limits = people_limits();
    let value = amendment();
    assert_eq!(person::amendment_bytes(&value), Some(8192));
    let encoded = people::policy::encode_amendment(&value, &limits).expect("whole sum fits");
    let mut out = Writer::new(skein_lib::length::usize_of(encoded.measure()));
    encoded.encode(&mut out).expect("measured amendment");
    let bytes = out.finish();
    let decoded = people_v14::Amendment::decode(&limits, &mut Reader::new(&bytes)).expect("whole amendment");
    assert_eq!(people::policy::decode_amendment(decoded, &limits), Ok(value.clone()));
    let mut past = value;
    past.reason = Box::new([1]);
    assert_eq!(person::amendment_bytes(&past), Some(8193));
    assert_eq!(people::policy::encode_amendment(&past, &limits), Err(people_problem()), "encoded past_sum");
    let mut parts = encoded.into_parts();
    parts.reason = Box::new([1]);
    let forged = people_v14::Amendment::new(&limits, parts).expect("each scalar still fits");
    assert_eq!(people::policy::decode_amendment(forged, &limits), Err(people_problem()), "decoded past_sum");
}

fn task_limits() -> tasks_v10::Limits {
    let mut domain = fixtures::TASKS_LIMITS;
    domain.result_bytes = 512;
    tasks::limits(&domain).expect("test-sized byte sum")
}
fn fields(count: usize) -> Box<[task::ResultField]> {
    let mut values = List::with_capacity(u32::try_from(count).expect("bounded count"));
    for _ in 0..count {
        values.push(task::ResultField { name: Box::new([1]), value: Box::new([]) }).expect("same count");
    }
    values.into_boxed()
}
fn saved_fields(
    fields: &[task::ResultField],
    limits: &tasks_v10::Limits,
) -> Result<Box<[task::ResultField]>, tasks_v10::Problem> {
    let value = task::TaskResult::Verdict { code: 7, fields: fields.into(), words: Box::new([]) };
    let encoded = tasks::phase::result(&value, limits)?;
    let mut out = Writer::new(skein_lib::length::usize_of(encoded.measure()));
    encoded.encode(&mut out).expect("measured verdict");
    let bytes = out.finish();
    let decoded = tasks_v10::TaskResult::decode(limits, &mut Reader::new(&bytes))?;
    match tasks::phase::read_result(decoded, limits)? {
        task::TaskResult::Verdict { fields, .. } => Ok(fields),
        task::TaskResult::Job { .. } => {
            Err(tasks_v10::Problem { path: tasks_v10::Path::TaskResultTag, reason: skein_codec::Reason::Malformed })
        }
        task::TaskResult::Approved { .. }
        | task::TaskResult::Report { .. }
        | task::TaskResult::Change { .. }
        | task::TaskResult::NoChange { .. }
        | task::TaskResult::Failure { .. } => unreachable!("verdict fixture"),
    }
}

#[test]
fn verdict_fields_round_trip_at_the_domain_sum_and_refuse_one_past_it() {
    let limits = task_limits();
    for (label, count, passes) in [("at_sum", 512, true), ("past_sum", 513, false)] {
        let fields = fields(count);
        let saved = saved_fields(&fields, &limits);
        if passes {
            assert_eq!(saved, Ok(fields), "{label}");
        } else {
            assert_eq!(
                saved,
                Err(tasks_v10::Problem { path: tasks_v10::Path::TaskResultTag, reason: skein_codec::Reason::Bound }),
                "{label}"
            );
        }
    }
}

#[test]
fn the_encoded_piece_bound_holds_a_verdict_at_the_domain_sum_exactly() {
    let limits = task_limits();
    let value = task::TaskResult::Verdict { code: 7, fields: fields(512), words: Box::new([]) };
    let encoded = tasks::phase::result(&value, &limits).expect("domain sum fits");
    let tasks_v10::TaskResult::Verdict(encoded) = encoded else { unreachable!("verdict fixture") };
    assert_eq!(encoded.fields().len(), 6656);
    assert_eq!(limits.verdict_result_fields, 6656);
    assert_eq!(limits.verdict_fields, 6656);
    assert_eq!(limits.verdict_message_fields, 6656);
}

fn people_problem() -> people_v14::Problem {
    people_v14::Problem { path: people_v14::Path::RowValue, reason: skein_codec::Reason::Bound }
}

#[test]
fn verdict_fields_are_framed_for_at_most_verdict_fields_items() {
    let mut domain = fixtures::TASKS_LIMITS;
    domain.result_bytes = 4096;
    let limits = tasks::limits(&domain).expect("within ceilings");
    // Each fixed field has eight bytes, and each piece adds four bytes.
    assert_eq!(tasks_v10::FieldRule::encoded_bound(&tasks::EMPTY), Some(8));
    assert_eq!(tasks_v10::ResultField::encoded_bound(&tasks::EMPTY), Some(8));
    assert_eq!(limits.verdict_fields, 512 * (8 + 4) + 4096);
    assert_eq!(limits.verdict_result_fields, 512 * (8 + 4) + 4096);
    assert_eq!(limits.verdict_message_fields, 512 * (8 + 4) + 4096);
}

#[test]
fn a_creation_without_payload_has_only_its_fixed_part() {
    let member = tasks_v10::Member::encoded_bound(&tasks::EMPTY).expect("fixed member");
    let creation = tasks_v10::Creation::encoded_bound(&tasks::EMPTY).expect("fixed creation");
    assert!(creation < 1024, "New={} Member(EMPTY)={member} Creation(EMPTY)={creation}", size_of::<task::New>());
}

fn wire_round_trip<T>(
    value: T,
    limits: &tasks_v10::Limits,
    measure: fn(&T) -> u32,
    encode: fn(&T, &mut Writer) -> Result<(), skein_lib::Overflow>,
    decode: fn(&tasks_v10::Limits, &mut Reader<'_>) -> Result<T, tasks_v10::Problem>,
) -> T {
    let mut out = Writer::new(skein_lib::length::usize_of(measure(&value)));
    encode(&value, &mut out).expect("measured item");
    decode(limits, &mut Reader::new(&out.finish())).expect("encoded item")
}

fn task_segments(count: u32) -> Box<[Box<[u8]>]> {
    let mut values = List::with_capacity(count);
    for _ in 0..count {
        values.push(Box::from([b'a'].as_slice())).expect("segment count");
    }
    values.into_boxed()
}
fn task_pattern(depth: u32, last: usize) -> task::Pattern {
    task::Pattern { segments: task_segments(depth), last: task::Last::Exact(skein_lib::bytes::zeroed(last)) }
}
fn task_grants(count: u32, depth: u32, last: usize) -> Box<[task::Grant]> {
    let mut values = List::with_capacity(count);
    for _ in 0..count {
        values.push(task::Grant { connector: 1, kind: 2, pattern: task_pattern(depth, last) }).expect("grant count");
    }
    values.into_boxed()
}

#[test]
fn an_authority_is_one_piece_by_its_bytes() {
    let domain = fixtures::TASKS_LIMITS;
    let limits = tasks::limits(&domain).expect("fixture limits");
    let mut value = fixtures::task_authority();
    // Two grants and two scopes, each with two segment bytes and six last bytes: 32 in all.
    value.grants = task_grants(domain.authority_grants, domain.authority_segments, 6);
    let mut scopes = List::with_capacity(domain.authority_grants);
    for _ in 0..domain.authority_grants {
        scopes
            .push(task::ResourceScope { connector: 1, pattern: task_pattern(domain.authority_segments, 6) })
            .expect("scope count");
    }
    value.note_resources = scopes.into_boxed();
    let encoded = tasks::value::authority(&value, &limits, None).expect("shared byte sum fits");
    let encoded = wire_round_trip(
        encoded,
        &limits,
        tasks_v10::Authority::measure,
        tasks_v10::Authority::encode,
        tasks_v10::Authority::decode,
    );
    assert_eq!(tasks::value::read_authority(encoded, &limits), Ok(value.clone()));
    value.grants[0].pattern.last = task::Last::Exact(Box::new([b'b'; 7]));
    assert_eq!(tasks::value::authority(&value, &limits, None), Err(task_problem(tasks_v10::Path::AuthorityGrants)));
}

#[test]
fn a_name_is_one_piece_by_the_hold_bytes() {
    let limits = task_limits();
    // Two segments share the fixture's 16 bytes.
    let mut value = task::Name { connector: 1, path: Box::new([Box::from([b'a'].as_slice()), Box::new([b'b'; 15])]) };
    let mut writer = fixtures::task_writer();
    writer.resource = value.clone();
    let row = task::Stored::Writer(writer.clone());
    let encoded = tasks::encode(&row, &limits, &[]).expect("hold byte sum fits");
    assert_eq!(tasks::decode(&encoded, &limits, &[]), Ok(row));
    value.path[0] = Box::new([b'a'; 2]);
    writer.resource = value;
    assert_eq!(tasks::encode(&task::Stored::Writer(writer), &limits, &[]), Err(task_problem(tasks_v10::Path::NamePath)));
}

#[test]
fn a_spec_parameters_piece_holds_the_spec_bytes() {
    let limits = task_limits();
    let mut value = fixtures::task_spec();
    // Words and the byte parameter share all 32 spec bytes.
    value.words = Box::new([b'a'; 16]);
    value.parameters[1] = task::Parameter::Bytes { name: 2, value: Box::new([b'b'; 16]) };
    let encoded = tasks::value::spec(&value, &limits).expect("spec byte sum fits");
    let encoded =
        wire_round_trip(encoded, &limits, tasks_v10::Spec::measure, tasks_v10::Spec::encode, tasks_v10::Spec::decode);
    assert_eq!(tasks::value::read_spec(encoded, &limits), Ok(value.clone()));
    value.words = Box::new([b'a'; 17]);
    assert_eq!(tasks::value::spec(&value, &limits), Err(task_problem(tasks_v10::Path::SpecWords)));
}

fn task_problem(path: tasks_v10::Path) -> tasks_v10::Problem {
    tasks_v10::Problem { path, reason: skein_codec::Reason::Bound }
}

fn batch_limits() -> task::Limits {
    task::Limits {
        batch: 160,
        batch_bytes: 131_072,
        spec_bytes: 65_536,
        parameters: 8192,
        result_bytes: 4096,
        contract_choices: 4,
        authority_grants: 64,
        authority_segments: 64,
        authority_bytes: 4096,
        holdings: 64,
        hold_segments: 64,
        hold_bytes: 4096,
        ..fixtures::TASKS_LIMITS
    }
}

fn smallest_creation() -> task::New {
    let mut value = fixtures::task_creation(1);
    value.llm = None;
    value.spec.words = Box::new([]);
    value.spec.parameters = Box::new([]);
    value.spec.inputs = Box::new([]);
    value.contract = task::Contract::Report { words: 0 };
    value.authority.grants = Box::new([]);
    value.authority.note_resources = Box::new([]);
    value.authority.delegation.kinds = Box::new([]);
    value.dependencies = Box::new([]);
    value.holdings = Box::new([]);
    value
}

// Fill with the named shape until either the count or complete owned-byte sum binds.
fn filled_batch(value: &task::New, limits: &task::Limits) -> Box<[task::New]> {
    let bytes = task::batch_bytes(core::slice::from_ref(value)).expect("one shape's bytes");
    let count =
        u64::from(limits.batch).min(u64::from(limits.batch_bytes).checked_div(bytes).expect("nonempty creation"));
    let count = u32::try_from(count).expect("batch count");
    let mut values = List::with_capacity(count);
    for _ in 0..count {
        values.push(value.clone()).expect("batch count");
    }
    values.into_boxed()
}

fn assert_batch_round_trips(label: &str, batch: &[task::New], domain: &task::Limits, recurring: bool) {
    assert!(task::bounded_batch(domain, batch), "{label}: domain admission");
    let owned = task::batch_bytes(batch).expect("batch owned bytes");
    assert!(owned <= u64::from(domain.batch_bytes), "{label}: owned sum");
    let limits = tasks::limits(domain).expect("wide counts within schema");
    let mut proposal = fixtures::task_proposal();
    proposal.action = task::ProposalAction::Batch(batch.into());
    let encoded = tasks::proposals::proposal(&proposal, &limits, None).expect(label);
    let encoded = wire_round_trip(
        encoded,
        &limits,
        tasks_v10::Proposal::measure,
        tasks_v10::Proposal::encode,
        tasks_v10::Proposal::decode,
    );
    assert_eq!(tasks::proposals::read_proposal(encoded, &limits), Ok(proposal), "{label}: proposal");
    let end = task::End::FinishedWithFollowups {
        result: task::TaskResult::Verdict { code: 7, fields: Box::new([]), words: Box::new([]) },
        followups: task::ResultFollowups::Delegates(batch.into()),
    };
    let encoded = tasks::carried::end(&end, &limits).expect(label);
    assert_eq!(tasks::carried::read_end(&encoded, &limits), Ok(end), "{label}: followups");
    if recurring {
        let template = task::RecurringTemplate { key: 1, batch: batch.into(), overlap: task::RecurringOverlap::Wait };
        let encoded = tasks::phase::template(&template, &limits, None).expect(label);
        let encoded = wire_round_trip(
            encoded,
            &limits,
            tasks_v10::RecurringTemplate::measure,
            tasks_v10::RecurringTemplate::encode,
            tasks_v10::RecurringTemplate::decode,
        );
        assert_eq!(tasks::phase::read_template(encoded, &limits), Ok(template), "{label}: template");
    }
}

#[test]
fn a_batch_at_its_bytes_encodes_in_every_shape() {
    let domain = batch_limits();
    let mut words = smallest_creation();
    words.spec.words = skein_lib::bytes::zeroed(usize::try_from(domain.spec_bytes).unwrap());
    let mut parameters = smallest_creation();
    // Each byte parameter owns its inline enum and one byte.
    let parameter_count =
        (usize::try_from(domain.batch_bytes).unwrap() - size_of::<task::New>()) / (size_of::<task::Parameter>() + 1);
    let parameter_count = parameter_count.min(usize::try_from(domain.parameters).unwrap());
    let mut values = List::with_capacity(u32::try_from(parameter_count).unwrap());
    for name in 0..parameter_count {
        values
            .push(task::Parameter::Bytes { name: u32::try_from(name).unwrap(), value: Box::new([b'p']) })
            .expect("parameter count");
    }
    parameters.spec.parameters = values.into_boxed();
    // The remaining bytes, fewer than one parameter, fill this batch to its exact byte bound.
    let used = task::batch_bytes(core::slice::from_ref(&parameters)).expect("parameter shape bytes");
    parameters.spec.words = skein_lib::bytes::zeroed(
        usize::try_from(u64::from(domain.batch_bytes).checked_sub(used).expect("parameter shape fits")).unwrap(),
    );
    assert_eq!(task::batch_bytes(core::slice::from_ref(&parameters)), Some(131_072));
    let mut contract = smallest_creation();
    // A byte has 256 distinct names; four choices retain 1,024 one-byte field names.
    let mut choices = List::with_capacity(domain.contract_choices);
    for code in 0..domain.contract_choices {
        let mut rules = List::with_capacity(256);
        for name in 0..=u8::MAX {
            rules.push(task::FieldRule { name: Box::new([name]), max: 0 }).expect("one rule per byte");
        }
        choices.push(task::Verdict { code, words: 0, followups: 0, fields: rules.into_boxed() }).expect("choice count");
    }
    contract.contract = task::Contract::Verdict { choices: choices.into_boxed() };
    let mut grants = smallest_creation();
    grants.authority.grants = task_grants(domain.authority_grants, domain.authority_segments, 0);
    let mut holdings = smallest_creation();
    let mut values = List::with_capacity(domain.holdings);
    for _ in 0..domain.holdings {
        values
            .push(task::Holding::Write {
                resource: task::Name { connector: 1, path: task_segments(domain.hold_segments) },
                kind: 2,
            })
            .expect("holding count");
    }
    holdings.holdings = values.into_boxed();
    for (label, value) in [
        ("words", words),
        ("parameters", parameters),
        ("contract", contract),
        ("grants", grants),
        ("holdings", holdings),
        ("smallest", smallest_creation()),
    ] {
        let batch = filled_batch(&value, &domain);
        assert_batch_round_trips(label, &batch, &domain, true);
    }
    let mut recurring = smallest_creation();
    recurring.recurring = Some(Box::new(task::RecurringTemplate {
        key: 1,
        batch: filled_batch(&smallest_creation(), &domain),
        overlap: task::RecurringOverlap::Wait,
    }));
    assert_batch_round_trips("recurring", &[recurring], &domain, false);
}

#[test]
fn a_creations_piece_past_its_bound_is_refused() {
    let limits = task_limits();
    let bound = limits.proposed_batch_members;
    // The generated record reads the outer piece length before any item.
    let mut out = Writer::new(4 + usize::try_from(bound).unwrap() + 1);
    out.put(&(bound + 1).to_be_bytes()).expect("length room");
    out.put(&skein_lib::bytes::zeroed(usize::try_from(bound).unwrap() + 1)).expect("piece room");
    assert_eq!(
        tasks_v10::ProposedBatch::decode(&limits, &mut Reader::new(&out.finish())),
        Err(task_problem(tasks_v10::Path::ProposedBatchMembers)),
    );
}

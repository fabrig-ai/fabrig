//! Shipped profiles fit the byte-summed families (protocol/store/records.md, 1).
use jig_protocol_config::{Profile, profile};
use jig_protocol_store::codec;

// Standard's development figures (protocol/limits.md), with small's other limits.
fn fabrig_scale() -> jig_domain_core::Limits {
    let mut core = profile(Profile::Small).core;
    core.tasks = jig_domain_tasks::Limits {
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
        authority_grants: 16,
        authority_segments: 8,
        hold_segments: 8,
        executor_kinds: 16,
        ..core.tasks
    };
    core
}

#[test]
fn the_codecs_hold_fabrig_scale() {
    let core = fabrig_scale();
    let limits = codec::limits(&core).expect("development figures within codec ceilings");
    println!(
        "fabrig-scale tasks_v10::Row::encoded_bound={}",
        jig_protocol_store::tasks_v10::Row::encoded_bound(&limits.tasks).expect("task row bound"),
    );
}

#[test]
fn the_codecs_refuse_figures_past_the_ceilings() {
    let core = fabrig_scale();
    // One past each binding schema ceiling (protocol/store/records.md, 1).
    let cases = [
        ("delegates", jig_domain_tasks::Limits { delegates: 257, ..core.tasks }),
        ("inbox_messages", jig_domain_tasks::Limits { inbox_messages: 129, ..core.tasks }),
        ("spec_bytes", jig_domain_tasks::Limits { spec_bytes: 262_145, ..core.tasks }),
        ("result_bytes", jig_domain_tasks::Limits { result_bytes: 524_289, ..core.tasks }),
    ];
    for (name, tasks) in cases {
        let domain = jig_domain_core::Limits { tasks, ..core };
        assert!(codec::limits(&domain).is_none(), "{name}");
    }
}

#[test]
fn both_shipped_profiles_admit_the_tasks_and_people_families() {
    for profile_name in [Profile::Small, Profile::Standard] {
        let limits = profile(profile_name);
        assert!(codec::tasks::limits(&limits.core.tasks).is_some(), "{profile_name:?} Tasks");
        assert!(codec::people::limits(&limits.core).is_some(), "{profile_name:?} People");
    }
}

#[test]
fn both_shipped_profiles_round_trip_the_settled_call_sum() {
    use jig_domain_core as core;
    for (profile_name, call_bytes) in [(Profile::Small, 69_632_u32), (Profile::Standard, 135_168)] {
        let domain = profile(profile_name);
        let limits = codec::limits(&domain.core).expect("shipped profile within codec ceilings");
        assert_eq!(domain.core.call_answer_bytes, call_bytes, "{profile_name:?}");
        // The opaque tool name and answer split the byte-summed carrier equally.
        let half = usize::try_from(call_bytes / 2).expect("half a call fits usize");
        let answers = [
            ("host", core::SettledAnswer::Host { error: false, body: vec![1; half].into_boxed_slice() }),
            (
                "delivery",
                core::SettledAnswer::Delivery {
                    outcome: core::DeliveryOutcome::Delivered,
                    evidence: vec![2; half].into_boxed_slice(),
                },
            ),
        ];
        for (label, answer) in &answers {
            let row = core::CoreRecord::Call(core::CallRecord {
                key: core::CallKey { task: 1, attempt: 2, completion: 3, position: 4 },
                part: core::CallPart::Unavailable,
                settled: Some(core::SettledCall {
                    serial: 1,
                    tool: vec![3; half].into_boxed_slice(),
                    answer: answer.clone(),
                }),
            });
            let bytes = codec::core::encode(&row, &limits).expect("sum at its limit");
            assert_eq!(codec::core::decode(&bytes, &limits), Ok(row), "{profile_name:?} {label}");
        }
    }
}

const STORE: skein_kv::Limits = skein_kv::Limits {
    key: 2_079,
    value: 268_435_456,
    ops: 1_000_000,
    commit: 1_073_741_824,
    queued: 3,
    queued_bytes: 3_221_225_472,
    budget: 268_435_456,
    segment: 1_073_741_824,
    snapshot_after: 268_435_456,
    chunk: 268_437_555,
    page: skein_kv::Page { rows: 2, bytes: 268_437_535 },
    deadline: skein_lib::Duration::from_nanos(1_000_000_000),
};

#[test]
fn both_shipped_profiles_are_admitted_by_the_store() {
    let application = jig_protocol_store::sizes::Application { version: 1, key: 2_078, value: 4_304 };
    assert!(STORE.is_usable());
    for profile_name in [Profile::Small, Profile::Standard] {
        let limits = profile(profile_name);
        assert!(codec::limits(&limits.core).is_some(), "{profile_name:?}");
        let sizes = jig_protocol_store::sizes::check_commits(
            &limits.core,
            &application,
            &jig_protocol_store::sizes::Commits { in_flight: limits.journal.commits, writes: limits.journal.writes },
            &STORE,
            1,
        )
        .expect("shipped profile fits the map-budget commit envelope");
        println!(
            "{profile_name:?}: largest key={}, value={}, commit={}, ops={}",
            sizes.key, sizes.value, sizes.commit, sizes.ops
        );
    }
}

#[test]
fn both_shipped_profiles_round_trip_a_policy_whose_roles_name_models() {
    use jig_domain_people as people;
    // As many models as a role's bits name, each name as long as configuration admits.
    let models = u8::try_from(u64::BITS).expect("one bit per model");
    let name_bytes = skein_lib::length::usize_of(jig_domain_core::MODEL_NAME_BYTES);
    let names: Box<[jig_domain_core::settings::ModelName]> = (0..models)
        .map(|model| jig_domain_core::settings::ModelName {
            name: vec![model; name_bytes].into_boxed_slice(),
            efforts: Box::new([]),
        })
        .collect();
    let role = |number, models| people::PolicyRole {
        number,
        authority: people::Authority {
            tools: 1,
            models,
            grants: Box::new([]),
            delegation: people::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 },
            spend: 0,
            deadline: None,
            notes: 0,
            note_resources: Box::new([]),
        },
        period_spend: 0,
        requests: 0,
        decides: 0,
    };
    let row = people::Stored::Policy {
        name: Box::from("Tally"),
        project: 1,
        value: people::PolicyValue {
            ceiling: role(0, Some(people::Models::ALL)).authority,
            escalation_role: Some(0),
            period_spend: 0,
            projections: Box::new([]),
            roles: Box::new([
                role(0, Some(people::Models::ALL)),
                role(1, Some(people::Models::Named((1 << 63) | 0b101))),
            ]),
            requirements: Box::new([]),
            permissions: Box::new([]),
            templates: Box::new([]),
        },
    };
    for profile_name in [Profile::Small, Profile::Standard] {
        let limits = codec::people::limits(&profile(profile_name).core).expect("shipped profile within ceilings");
        let bytes = codec::people::encode(&row, &limits, &names).expect("every name within people's bounds");
        assert_eq!(codec::people::decode(&bytes, &limits, &names), Ok(row.clone()), "{profile_name:?}");
    }
}

/// Measure format carriers and check each shipped store envelope without changing it.
#[test]
fn question_formats_fit_the_shipped_envelopes() {
    use jig_domain_tasks as tasks;
    use jig_protocol_store::{sizes, tasks_v10};
    println!(
        "resident credit={} task={} line={} activity={} request={} core={} store={}",
        size_of::<tasks::QuestionCredit>(),
        size_of::<tasks::TaskRecord>(),
        size_of::<tasks::Line>(),
        size_of::<tasks::ActivityLine>(),
        size_of::<tasks::Request>(),
        size_of::<jig_domain_core::Core>(),
        size_of::<jig_protocol_store::Store>(),
    );
    println!(
        "encoded credit={} kept={} interests={}",
        tasks_v10::QuestionCredit::encoded_bound(&tasks_v10::CEILINGS).expect("credit bound"),
        tasks_v10::KeptLine::encoded_bound(&tasks_v10::CEILINGS).expect("kept bound"),
        tasks_v10::TaskInterests::encoded_bound(&tasks_v10::CEILINGS).expect("interests bound"),
    );
    for name in [Profile::Small, Profile::Standard] {
        let limits = profile(name);
        let commits = sizes::Commits { in_flight: limits.journal.commits, writes: limits.journal.writes };
        let sizes = sizes::check_commits(
            &limits.core,
            &sizes::Application { version: 1, key: 2_078, value: 4_304 },
            &commits,
            &limits.store.store,
            limits.store.orphan_erases,
        )
        .expect("question formats fit current store envelopes");
        let codec = codec::tasks::limits(&limits.core.tasks).expect("task codec limits");
        println!(
            "{name:?}: task_row={} value={}/{} page={}/{} commit={}/{} ops={}/{} queued_bytes={}/{} journal_writes={} journal_held={}",
            tasks_v10::TaskRecord::encoded_bound(&codec).expect("task row bound"),
            sizes.value,
            limits.store.store.value,
            sizes.key + sizes.value,
            limits.store.store.page.bytes,
            sizes.commit,
            limits.store.store.commit,
            sizes.ops,
            limits.store.store.ops,
            u64::from(sizes.commit) * u64::from(commits.in_flight),
            limits.store.store.queued_bytes,
            commits.writes,
            u64::from(commits.writes) * u64::from(commits.in_flight),
        );
    }
}

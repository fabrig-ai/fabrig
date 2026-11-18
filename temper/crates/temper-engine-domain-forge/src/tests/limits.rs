use super::*;

#[test]
fn a_raised_outbox_retains_the_ninth_entry_without_reconstruction() {
    let mut env = env();
    env.limits.entries = 8;
    env.limits.client.entries = 16;
    let mut domain = domain_with_limits(&env.limits);
    let mut out = Queue::uncapped();
    domain.restore_begin(env.wall);
    step(&mut domain, &env, Event::Restore { record: Stored::Repository(adopted()) }, &mut out);
    out.clear();
    for number in 1..=8 {
        step(&mut domain, &env, Event::Enqueue { entry: status(number, 7) }, &mut out);
    }
    assert_eq!(domain.limit_usage(&LimitSet::Entries), Some(8));
    out.clear();
    step(&mut domain, &env, Event::Enqueue { entry: status(9, 7) }, &mut out);
    assert_eq!(domain.limit_usage(&LimitSet::Entries), Some(8));
    assert_eq!(
        out.pop(),
        Some(Request::ResourceRefused {
            task: 7,
            operation: b"enqueue effect",
            resource: None,
            cause: RefusalCause::Limit { member: "forge.entries", configured: 8, needed: 9 },
        })
    );
    env.limits.entries = 16;
    step(&mut domain, &env, Event::LimitsRaised, &mut out);
    step(&mut domain, &env, Event::Enqueue { entry: status(9, 7) }, &mut out);
    assert_eq!(domain.limit_usage(&LimitSet::Entries), Some(9));
}

#[test]
fn entrance_limits_reject_incoherent_names_rows_and_windows() {
    assert!(valid(&LIMITS));
    for limits in [
        Limits { repositories: 0, ..LIMITS },
        Limits { tasks: 0, ..LIMITS },
        Limits { holds: 0, ..LIMITS },
        Limits { subscriptions: 0, ..LIMITS },
        Limits { entries: 0, ..LIMITS },
        Limits { resources_per_task: 0, ..LIMITS },
        Limits { name_bytes: 0, ..LIMITS },
        Limits { name_bytes: PANEL_NAME_BYTES + 1, ..LIMITS },
        Limits {
            change_policy: temper_engine_domain_forge_change::Limits {
                reruns: 1,
                gates: PANEL_GATES + 1,
                questions: 4,
                ..LIMITS.change_policy
            },
            ..LIMITS
        },
        Limits { queue_window: Duration::ZERO, ..LIMITS },
        Limits { adoptions: 0, ..LIMITS },
        Limits { applications: 0, ..LIMITS },
        Limits { collaborators: 0, ..LIMITS },
        Limits { landings: 0, ..LIMITS },
        Limits { landing_rules: 0, ..LIMITS },
        Limits { brief_sections: 0, ..LIMITS },
        Limits { brief_bytes: 0, ..LIMITS },
        Limits { client: skein_forge_client::Limits { calls: 0, ..LIMITS.client }, ..LIMITS },
        Limits { repositories: LIMITS.client.repositories + 1, ..LIMITS },
        Limits { entries: LIMITS.client.entries + 1, ..LIMITS },
        Limits { holds: LIMITS.client.resources + 1, ..LIMITS },
    ] {
        assert!(!valid(&limits), "{limits:?}");
    }
}

#[test]
fn retained_row_counts_and_landing_indices_must_be_representable() {
    for limits in [
        Limits { tasks: u32::MAX, ..LIMITS },
        Limits { named_calls: u32::MAX, ..LIMITS },
        Limits { landings: u32::MAX, ..LIMITS },
        Limits { landing_members: u32::from(u16::MAX) + 1, ..LIMITS },
        Limits { gates: u32::from(u16::MAX) + 1, ..LIMITS },
        Limits { receipts_per_gate: 1, ..LIMITS },
        Limits { receipts_per_gate: u32::MAX, ..LIMITS },
        Limits { trees_in_flight: 0, ..LIMITS },
        Limits { key_entries: 0, ..LIMITS },
        Limits { key_path_bytes: 0, ..LIMITS },
    ] {
        assert!(!valid(&limits), "{limits:?}");
    }
    assert!(valid(&Limits { name_bytes: PANEL_NAME_BYTES, receipts_per_gate: 2, ..LIMITS }));
}

#[test]
fn internal_output_envelopes_do_not_constrain_admission() {
    for output in [0, 1, u32::MAX] {
        let limits = Limits { output, ..LIMITS };
        assert!(valid(&limits));
        let mut domain = Domain::new(
            &limits,
            4,
            Config { push: None, namespace: Box::from(&b"test"[..]), writers: Box::new([]) },
            &[],
            &[],
            false,
        )
        .expect("internal output does not bound admission");
        domain.restore_begin(Wall::EPOCH);
        let environment = Env { limits, ..env() };
        let mut out = Queue::uncapped();
        step(&mut domain, &environment, Event::Restore { record: Stored::Repository(adopted()) }, &mut out);
        step(
            &mut domain,
            &environment,
            Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic },
            &mut out,
        );
    }
}

#[test]
fn the_questions_bound_leaves_room_for_one_waiting_question() {
    use temper_engine_domain_forge_change::QUESTIONS_OPEN;
    let open = u32::try_from(QUESTIONS_OPEN).expect("question capacity fits u32");
    for (questions, accepted) in [(open - 1, true), (open, false)] {
        let limits = Limits {
            change_policy: temper_engine_domain_forge_change::Limits { questions, ..LIMITS.change_policy },
            ..LIMITS
        };
        assert_eq!(valid(&limits), accepted, "questions: {questions}");
    }
}

#[test]
fn a_changes_delegate_must_have_a_positive_task_bound() {
    for (tasks, accepted) in [(0, false), (1, true), (u32::MAX, true)] {
        let limits = Limits {
            change_policy: temper_engine_domain_forge_change::Limits { tasks, ..LIMITS.change_policy },
            ..LIMITS
        };
        assert_eq!(valid(&limits), accepted, "tasks: {tasks}");
    }
}

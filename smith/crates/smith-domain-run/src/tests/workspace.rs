use super::*;

#[test]
fn convention_path_payloads_fill_the_exact_charter_cap_and_cancel_discovery_settles() {
    let mut selected = charter();
    selected.conventions = Some(crate::Conventions {
        guide: Box::new([b'g'; crate::Conventions::PATH_CAPACITY]),
        checks: Box::new([b'c'; crate::Conventions::PATH_CAPACITY]),
    });
    let exact = crate::charter::cost(&selected).expect("bounded maximum paths")
        + crate::workspace::cost(Some(&workspace())).unwrap();
    let baseline = crate::charter::cost(&Charter { conventions: None, ..charter() }).unwrap()
        + crate::workspace::cost(Some(&workspace())).unwrap();
    // The fixture contributes explicit path payloads; remove them from
    // this independent cap comparison rather than reducing any fixture maximum.
    assert_eq!(exact, baseline + u64::try_from(crate::Conventions::PATH_CAPACITY * 2).unwrap());
    let mut harness = Harness::new(Limits { run_bytes: exact, ..LIMITS });
    let requests = harness.start(77, selected);
    let [Request::Admitted { run, .. }, Request::Read { owner, at, .. }] = requests.as_ref() else {
        panic!("exact aggregate convention storage is admitted: {requests:?}");
    };
    assert_eq!(at.path.len(), crate::Conventions::PATH_CAPACITY);
    assert_eq!(*owner, *run);
    let run = *run;
    assert!(harness.step(Event::Cancel { run }).is_empty(), "cancel cannot manufacture the outstanding Read terminal");
    assert_eq!(
        answered(harness.step(Event::Read { owner: run, read: Read::Missing })),
        (77, failed(Failure::Cancelled, Spend::ZERO))
    );
    harness.domain.reclaim();
    assert_eq!((harness.domain.runs(), harness.domain.conversations(), harness.domain.calls()), (0, 0, 0));

    let oversized = Charter {
        conventions: Some(crate::Conventions {
            guide: Box::new([b'g'; crate::Conventions::PATH_CAPACITY]),
            checks: Box::new([b'c'; crate::Conventions::PATH_CAPACITY]),
        }),
        instructions: {
            let brief = charter().instructions;
            let mut writer = skein_lib::Writer::new(brief.len().checked_add(1).unwrap());
            writer.put(&brief).expect("room for the original brief");
            writer.put(b"x").expect("one exact extra byte");
            writer.finish()
        },
        ..charter()
    };
    assert_eq!(answered(harness.start(78, oversized)), (78, Answer::Refused(Refusal::Invalid(Invalid::TooLarge))));
    assert_eq!((harness.domain.runs(), harness.domain.conversations(), harness.domain.calls()), (0, 0, 0));
}

#[test]
fn accepted_mixed_workspace_drives_discovery_opening_and_child_metadata() {
    let mut harness = Harness::new(LIMITS);
    let mounted = mixed_workspace();
    let mut policy = charter();
    policy.reserve.time = LIMITS.check_timeout;
    policy.grants.deliver = Some(ChangeSpec { checks_must_pass: true, fields: Box::new([]) });
    let requests = harness.start_workspace(51, policy, Some(mounted.clone()));
    let [Request::Admitted { run, .. }, Request::Read { at, .. }] = requests.as_ref() else {
        panic!("mixed workspace starts discovery: {requests:?}");
    };
    let run = *run;
    assert_eq!(at.root, Token::new(41));
    let requests = harness.step(Event::Read { owner: run, read: Read::Missing });
    let [Request::Probe { at, .. }] = requests.as_ref() else { panic!("writable plain directory probes checks") };
    assert_eq!(at.root, Token::new(41));
    let requests = harness.step(Event::Probed { owner: run, executable: false });
    let [Request::Read { at, .. }] = requests.as_ref() else { panic!("read-only git guide") };
    assert_eq!(at.root, Token::new(42));
    let requests = harness.step(Event::Read { owner: run, read: Read::Missing });
    let [Request::Open { opening, .. }] = requests.as_ref() else { panic!("read-only git has no check probe") };
    assert_eq!(opening.workspace, Some(mounted.clone()));
    for fragment in [b"a plain directory".as_slice(), b"a git working tree", b"initial merge conflicts: `src/merge.rs`"]
    {
        assert!(skein_lib::bytes::find(&opening.system, fragment).is_some());
    }
    let child = crate::prompt::child(
        &charter(),
        Some(&mounted),
        &Found::with_capacity(0),
        b"Inspect",
        opening.families,
        crate::Granted {
            turns: 3,
            time: Duration::from_secs(4),
            spend: 100,
            turns_set: crate::Set::Pool,
            time_set: crate::Set::Pool,
        },
        LIMITS.answer_bytes,
    );
    assert!(skein_lib::bytes::find(&child, b"initial merge conflicts: `src/merge.rs`").is_some());
    assert!(skein_lib::bytes::find(&child, b"`reference`, which you may only read").is_some());
}

#[test]
fn absent_workspace_suppresses_workspace_families_and_preserves_main_services() {
    let mut harness = Harness::new(LIMITS);
    let mut policy = charter();
    policy.grants.tools = Tools { inspect: true, modify: true, shell: true };
    policy.grants.agents = true;
    policy.grants.host_tools = Box::new([HostTool {
        name: bytes(b"host_action"),
        description: bytes(b"Host action"),
        schema: bytes(b"{}"),
        effect: crate::HostEffect::Write,
        timeout: Duration::from_secs(5),
    }]);
    let requests = harness.start_workspace(52, policy, None);
    let [Request::Admitted { .. }, Request::Open { opening, .. }] = requests.as_ref() else {
        panic!("workspace-free start has no discovery effects: {requests:?}");
    };
    assert_eq!(opening.workspace, None);
    assert_eq!(opening.tools, Tools { inspect: false, modify: false, shell: false });
    assert_eq!(opening.families.tools, opening.tools);
    assert!(opening.families.agents && opening.finish && opening.wait);
    assert!(!opening.deliver);
    assert_eq!(opening.host_tools.len(), 1);
    assert_eq!(opening.host_tools[0].name.as_ref(), b"host_action");
    assert!(skein_lib::bytes::find(&opening.system, b"There is no checkout.").is_some());
}

#[test]
fn a_run_with_only_finish_does_not_offer_or_accept_wait() {
    let mut harness = Harness::new(LIMITS);
    let mut policy = charter();
    policy.grants.wait = false;
    policy.grants.tools = Tools { inspect: false, modify: false, shell: false };
    policy.grants.agents = false;
    policy.grants.host_tools = Box::new([]);
    let requests = harness.start_workspace(53, policy, None);
    let [Request::Admitted { .. }, Request::Open { conversation, opening }] = requests.as_ref() else {
        panic!("workspace-free start opens main: {requests:?}");
    };
    assert!(!opening.wait);
    assert!(opening.finish);
    assert!(!opening.deliver);
    assert_eq!(opening.tools, Tools { inspect: false, modify: false, shell: false });
    assert!(opening.host_tools.is_empty());
    let conversation = *conversation;
    assert!(harness.step(Event::Started { conversation, peer: Token::new(54) }).is_empty());
    let call = Token::new(55);
    assert_eq!(
        harness
            .step(Event::Delegated {
                conversation,
                call,
                name: crate::CallName { activation: 1, completion: 1, position: 0 },
                ask: Ask::Wait,
                deadline: Time::from_nanos(u64::MAX),
            })
            .as_ref(),
        [Request::Return { spent: 0, call, result: Returned::Refused { refusal: AskRefusal::NotGranted } }]
    );
}

#[test]
fn workspace_names_roots_and_presence_are_admitted_independently() {
    refuse_workspace(Some(Workspace { directories: Box::new([]) }), LIMITS, Invalid::Workspace);
    for name in [b"".as_slice(), b".", b"..", b"bad/name", b"bad\0name"] {
        refuse_workspace(
            Some(Workspace { directories: Box::new([workspace_directory(name, 1, false, false, &[])]) }),
            LIMITS,
            Invalid::Workspace,
        );
    }
    for directories in [
        Box::new([workspace_directory(b"one", 1, false, false, &[]), workspace_directory(b"one", 2, true, true, &[])]),
        Box::new([workspace_directory(b"one", 1, false, false, &[]), workspace_directory(b"two", 1, true, true, &[])]),
    ] {
        refuse_workspace(Some(Workspace { directories }), LIMITS, Invalid::Workspace);
    }
    refuse_workspace(Some(mixed_workspace()), Limits { directories: 1, ..LIMITS }, Invalid::Workspace);
    refuse_workspace(
        Some(Workspace { directories: Box::new([workspace_directory(b"name", 1, false, false, &[])]) }),
        Limits { directory_name_bytes: 3, ..LIMITS },
        Invalid::Workspace,
    );
}

#[test]
fn initial_conflicts_require_git_unique_relative_paths_and_independent_caps() {
    for path in [b"".as_slice(), b"/root", b"a//b", b"a/./b", b"a/../b", b"a/", b"a\0b"] {
        refuse_workspace(
            Some(Workspace { directories: Box::new([workspace_directory(b"git", 1, false, true, &[path])]) }),
            LIMITS,
            Invalid::Workspace,
        );
    }
    for (directory, limits) in [
        (workspace_directory(b"plain", 1, true, false, &[b"a"]), LIMITS),
        (workspace_directory(b"git", 1, false, true, &[b"a", b"a"]), LIMITS),
        (workspace_directory(b"git", 1, false, true, &[b"a", b"b"]), Limits { conflicts: 1, ..LIMITS }),
        (workspace_directory(b"git", 1, false, true, &[b"abcd"]), Limits { conflict_path_bytes: 3, ..LIMITS }),
    ] {
        refuse_workspace(Some(Workspace { directories: Box::new([directory]) }), limits, Invalid::Workspace);
    }
    let limits = Limits { run_bytes: 65_536, ..LIMITS };
    let path = Box::new([b'p'; crate::Marker::CAPACITY]);
    let mounted =
        Workspace { directories: Box::new([workspace_directory(b"git", 1, false, true, &[path.as_slice()])]) };
    assert_eq!(crate::charter::check(&charter(), Some(&mounted), &limits), Ok(limits.guide_bytes));
    let over = Box::new([b'p'; crate::Marker::CAPACITY + 1]);
    refuse_workspace(
        Some(Workspace { directories: Box::new([workspace_directory(b"git", 1, false, true, &[over.as_slice()])]) }),
        limits,
        Invalid::Workspace,
    );
    let invalid_limits = Limits { conflict_path_bytes: 4097, ..limits };
    assert!(!valid(&invalid_limits));
    refuse_workspace(None, invalid_limits, Invalid::Workspace);
}

#[test]
fn exact_workspace_limits_and_aggregate_price_include_every_owner() {
    let limits = Limits { directories: 2, directory_name_bytes: 9, conflicts: 1, conflict_path_bytes: 12, ..LIMITS };
    let mounted = mixed_workspace();
    let directory_cells = 2 * u64::try_from(size_of::<Directory>()).unwrap();
    let conflict_cell = u64::try_from(size_of::<Box<[u8]>>()).unwrap();
    let owned = directory_cells + 4 + 9 + 10 + conflict_cell + 12;
    assert_eq!(crate::workspace::cost(Some(&mounted)), Some(owned));
    assert_eq!(crate::workspace::cost(None), Some(0));
    let policy = charter();
    let exact = crate::charter::cost(&policy).unwrap() + owned;
    let exact_limits = Limits { run_bytes: exact, ..limits };
    assert_eq!(crate::charter::check(&policy, Some(&mounted), &exact_limits), Ok(exact_limits.guide_bytes));
    refuse_workspace(Some(mounted.clone()), Limits { run_bytes: exact - 1, ..limits }, Invalid::TooLarge);
    // Aggregate ownership refuses before quadratic semantic duplicate scans.
    let mut duplicate = mounted;
    duplicate.directories[1].root = duplicate.directories[0].root;
    refuse_workspace(Some(duplicate), Limits { run_bytes: exact - 1, ..limits }, Invalid::TooLarge);
    assert!(valid(&exact_limits));
}

#[test]
fn contracts_requiring_writes_without_writable_directories_refuse_before_effects() {
    for mounted in [None, Some(workspace())] {
        let mut harness = Harness::new(LIMITS);
        let change = Charter {
            reserve: crate::charter::Reserve { turns: 0, time: LIMITS.check_timeout },
            outcome: OutcomeSpec {
                change: Some(ChangeSpec { checks_must_pass: true, fields: Box::new([]) }),
                ..charter().outcome
            },
            ..charter()
        };
        assert_eq!(
            answered(harness.start_workspace(54, change, mounted.clone())),
            (54, Answer::Refused(Refusal::Invalid(Invalid::Outcome)))
        );
        let mut deliver = charter();
        deliver.reserve.time = LIMITS.check_timeout;
        deliver.grants.deliver = Some(ChangeSpec { checks_must_pass: true, fields: Box::new([]) });
        assert_eq!(
            answered(harness.start_workspace(55, deliver, mounted)),
            (55, Answer::Refused(Refusal::Invalid(Invalid::Grants)))
        );
        assert_eq!(harness.domain.runs(), 0);
    }
}

#[test]
fn brief_section_count_refuses_before_discovery_even_when_aggregate_would_fit() {
    let limits = Limits { brief_sections: 2, ..LIMITS };
    let mut harness = Harness::new(limits);
    let selected = Charter {
        brief: crate::Brief {
            sections: Box::new([
                crate::Section { title: Box::new([]), text: Box::new([]) },
                crate::Section { title: Box::new([]), text: Box::new([]) },
                crate::Section { title: Box::new([]), text: Box::new([]) },
            ]),
        },
        ..charter()
    };
    assert!(crate::charter::cost(&selected).unwrap() < limits.run_bytes);
    assert_eq!(answered(harness.start(91, selected)), (91, Answer::Refused(Refusal::Invalid(Invalid::TooLarge))));
    assert_eq!((harness.domain.runs(), harness.domain.conversations(), harness.domain.calls()), (0, 0, 0));
}

#[test]
fn instructions_and_ordered_section_cells_titles_and_text_attain_the_exact_aggregate() {
    let selected = Charter {
        instructions: bytes(b"Literal role"),
        brief: crate::Brief {
            sections: Box::new([
                crate::Section { title: bytes(b"Repeated"), text: bytes(b"first body") },
                crate::Section { title: Box::new([]), text: bytes(b"empty title") },
                crate::Section { title: bytes(b"Repeated"), text: Box::new([]) },
            ]),
        },
        ..charter()
    };
    let empty = Charter { instructions: Box::new([]), brief: crate::Brief { sections: Box::new([]) }, ..charter() };
    let expected_context = u64::try_from(
        b"Literal role".len()
            + 3 * size_of::<crate::Section>()
            + 2 * b"Repeated".len()
            + b"first body".len()
            + b"empty title".len(),
    )
    .unwrap();
    let expected =
        crate::charter::cost(&empty).unwrap() + expected_context + crate::workspace::cost(Some(&workspace())).unwrap();
    assert_eq!(
        crate::charter::cost(&selected).unwrap() + crate::workspace::cost(Some(&workspace())).unwrap(),
        expected
    );
    let limits = Limits { run_bytes: expected, brief_sections: 3, ..LIMITS };
    assert_eq!(crate::charter::check(&selected, Some(&workspace()), &limits), Ok(limits.guide_bytes));
    assert_eq!(
        crate::charter::check(&selected, Some(&workspace()), &Limits { run_bytes: expected - 1, ..limits }),
        Err(Invalid::TooLarge)
    );
    assert_eq!(
        crate::charter::check(&selected, Some(&workspace()), &Limits { brief_sections: 2, ..limits }),
        Err(Invalid::TooLarge)
    );
    let mut harness = Harness::new(limits);
    let requests = harness.start(92, selected);
    match requests.as_ref() {
        [Request::Admitted { .. }, Request::Read { .. }] => {}
        _ => panic!("exact count/byte cap admits before discovery: {requests:?}"),
    }
}

#[test]
fn empty_main_instructions_and_brief_are_valid_even_with_zero_section_limit() {
    let selected = Charter { instructions: Box::new([]), brief: crate::Brief { sections: Box::new([]) }, ..charter() };
    assert_eq!(
        crate::charter::check(&selected, Some(&workspace()), &Limits { brief_sections: 0, ..LIMITS }),
        Ok(LIMITS.guide_bytes)
    );
}

#[test]
fn workspace_host_paths_fit_their_receiving_bound_and_normalized_absolute_shape() {
    let mut mounted = workspace();
    let exact = u32::try_from(mounted.directories[0].host.len()).unwrap();
    assert_eq!(crate::workspace::check(Some(&mounted), &Limits { host_path_bytes: exact, ..LIMITS }), Ok(()));
    assert_eq!(
        crate::workspace::check(Some(&mounted), &Limits { host_path_bytes: exact - 1, ..LIMITS }),
        Err(Invalid::Workspace)
    );
    for host in
        [b"relative".as_slice(), b"/a/../b".as_slice(), b"/a//b".as_slice(), b"/a/".as_slice(), b"/a\0b".as_slice()]
    {
        mounted.directories[0].host = bytes(host);
        assert_eq!(crate::workspace::check(Some(&mounted), &LIMITS), Err(Invalid::Workspace));
    }
    mounted.directories[0].host = bytes(b"/");
    assert_eq!(crate::workspace::check(Some(&mounted), &LIMITS), Ok(()));
}

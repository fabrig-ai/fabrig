use super::*;

#[test]
fn starts_beyond_the_run_or_conversation_slots_are_refused_as_busy() {
    let mut harness = Harness::new(Limits { runs: 1, ..LIMITS });
    let _: (Token, Token) = harness.admit(1);
    assert_eq!(answered(harness.start(2, charter())), (2, Answer::Refused(Refusal::Busy)));

    let mut harness = Harness::new(Limits { conversations: 1, ..LIMITS });
    let _: (Token, Token) = harness.admit(1);
    assert_eq!(answered(harness.start(2, charter())), (2, Answer::Refused(Refusal::Busy)));
    assert_eq!(harness.domain.runs(), 1);

    // A charter that can never fit is invalid, room or not.
    let never = Charter { budget: Budget { turns: 0, ..BUDGET }, ..charter() };
    assert_eq!(
        answered(harness.start(3, never)),
        (3, Answer::Refused(Refusal::Invalid(Invalid::Budget(crate::BudgetRefusal::Empty(crate::Part::Turns)))))
    );
}

#[test]
fn drawn_budget_ceilings_admit_the_boundary_and_refuse_each_part_before_effects() {
    let mut random = skein_lib::Rng::new(701);
    for _ in 0_u32..128 {
        let ceiling = Budget {
            turns: u32::try_from(random.between(1, 4096)).expect("bounded turns"),
            spend: random.between(1, 1_000_000_000),
            time: Duration::from_secs(random.between(1, 86_400)),
        };
        let limits = Limits { budget: ceiling, ..LIMITS };
        let policy = Charter { budget: ceiling, ..charter() };
        let requests = Harness::new(limits).start_workspace(7, policy, None);
        let [Request::Admitted { .. }, Request::Open { .. }] = &*requests else {
            panic!("at ceiling is admitted: {requests:?}")
        };
        for (budget, reason) in [
            (
                Budget { turns: ceiling.turns.checked_add(1).expect("bounded"), ..ceiling },
                crate::Ceiling::Turns(ceiling.turns),
            ),
            (
                Budget { spend: ceiling.spend.checked_add(1).expect("bounded"), ..ceiling },
                crate::Ceiling::Spend(ceiling.spend),
            ),
            (
                Budget { time: ceiling.time.checked_add(Duration::from_secs(1)).expect("bounded"), ..ceiling },
                crate::Ceiling::Time(ceiling.time),
            ),
        ] {
            let mut harness = Harness::new(limits);
            assert_eq!(
                answered(harness.start(7, Charter { budget, ..charter() })),
                (7, Answer::Refused(Refusal::Invalid(Invalid::Budget(crate::BudgetRefusal::Ceiling(reason)))))
            );
            assert_eq!(harness.domain.runs(), 0);
        }
    }
    for (budget, part) in [
        (Budget { turns: 0, ..BUDGET }, crate::Part::Turns),
        (Budget { spend: 0, ..BUDGET }, crate::Part::Spend),
        (Budget { time: Duration::ZERO, ..BUDGET }, crate::Part::Time),
    ] {
        assert_eq!(
            answered(Harness::new(LIMITS).start(7, Charter { budget, ..charter() })),
            (7, Answer::Refused(Refusal::Invalid(Invalid::Budget(crate::BudgetRefusal::Empty(part)))))
        );
    }
}

#[test]
fn every_main_and_child_model_rate_is_positive_before_any_effect() {
    let positive = crate::Prices { input: 1, cached: 1000, cache_write: Some(3), output: 1, unit: 1_000_000 };
    for (prices, rate) in [
        (crate::Prices { input: 0, ..positive }, crate::Rate::Input),
        (crate::Prices { cached: 0, ..positive }, crate::Rate::Cached),
        (crate::Prices { cache_write: Some(0), ..positive }, crate::Rate::CacheWrite),
        (crate::Prices { output: 0, ..positive }, crate::Rate::Output),
    ] {
        for main in [true, false] {
            let mut policy = agents();
            if main {
                policy.llm.prices = prices;
            } else {
                policy.models[0].prices = prices;
            }
            let mut harness = Harness::new(LIMITS);
            assert_eq!(
                answered(harness.start(7, policy)),
                (7, Answer::Refused(Refusal::Invalid(Invalid::Price(rate))))
            );
            assert_eq!(harness.domain.runs(), 0);
        }
    }
    let mut policy = agents();
    policy.llm.prices = positive;
    policy.models[0].prices = positive;
    let requests = Harness::new(LIMITS).start_workspace(7, policy, None);
    let [Request::Admitted { .. }, Request::Open { .. }] = &*requests else {
        panic!("positive model rates are admitted: {requests:?}")
    };
}

#[test]
fn charters_beyond_the_limits_are_refused_as_invalid() {
    let three: Box<[Directory]> = Box::new([repository(b"a"), repository(b"b"), repository(b"c")]);
    let twins: Box<[Directory]> = Box::new([repository(b"a"), repository(b"a")]);
    let host_tools = Box::new([
        HostTool {
            name: bytes(b"reply"),
            description: b"Host action".as_slice().into(),
            schema: b"{}".as_slice().into(),
            effect: crate::HostEffect::Read,
            timeout: Duration::from_secs(5),
        },
        HostTool {
            name: bytes(b"reply"),
            description: b"Host action".as_slice().into(),
            schema: b"{}".as_slice().into(),
            effect: crate::HostEffect::Read,
            timeout: Duration::from_secs(5),
        },
    ]);
    let llm = Llm {
        effort: None,
        prices: crate::Prices { input: 1, cached: 1, cache_write: None, output: 1, unit: 1_000_000 },
        account: Some(0),
        endpoint: Endpoint(2),
        model: bytes(b"model-b"),
        window: 8192,
        output: 512,
        dialect: 1,
    };
    let models = Box::new([llm.clone(), Llm { endpoint: Endpoint(3), ..llm }]);
    for directories in [three, twins] {
        let mut harness = Harness::new(LIMITS);
        assert_eq!(
            answered(harness.start_workspace(9, charter(), Some(Workspace { directories }))),
            (9, Answer::Refused(Refusal::Invalid(Invalid::Workspace)))
        );
    }
    let cases = [
        (Charter { llm: Llm { window: 8192, output: 0, ..charter().llm }, ..charter() }, Invalid::Llm),
        (
            Charter {
                llm: Llm { prices: crate::Prices { unit: 0, ..charter().llm.prices }, ..charter().llm },
                ..charter()
            },
            Invalid::Llm,
        ),
        (Charter { llm: Llm { window: 0, output: 512, ..charter().llm }, ..charter() }, Invalid::Llm),
        (Charter { models, ..charter() }, Invalid::Llm),
        (Charter { instructions: Box::from([b'x'; 4096].as_slice()), ..charter() }, Invalid::TooLarge),
        (Charter { grants: Grants { host_tools, ..charter().grants }, ..charter() }, Invalid::Grants),
        (Charter { outcome: spec(Box::new([])), ..charter() }, Invalid::Outcome),
        (Charter { outcome: spec(Box::new([rule(b"a", 0, 0), rule(b"a", 1, 1)])), ..charter() }, Invalid::Outcome),
        (
            Charter { outcome: spec(Box::new([rule(b"a", 0, 0), rule(b"b", 0, 0), rule(b"c", 0, 0)])), ..charter() },
            Invalid::Outcome,
        ),
        (Charter { outcome: spec(Box::new([rule(b"a", 3, 2)])), ..charter() }, Invalid::Outcome),
        // ItemSpec allowed, with no kind for them to be.
        (
            Charter {
                outcome: spec(Box::new([VerdictRule {
                    items: crate::outcome::ItemSpec { kinds: Box::new([]), ..(rule(b"a", 0, 1)).items },
                    ..rule(b"a", 0, 1)
                }])),
                ..charter()
            },
            Invalid::Outcome,
        ),
    ];
    for (call, (charter, invalid)) in (0_u64..).zip(cases) {
        let mut harness = Harness::new(LIMITS);
        assert_eq!(answered(harness.start(call, charter)), (call, Answer::Refused(Refusal::Invalid(invalid))));
        assert_eq!(harness.domain.runs(), 0);
    }
    // A change alone is an outcome.
    let mut harness = Harness::new(LIMITS);
    drop(harness.start(
        1,
        Charter {
            reserve: crate::charter::Reserve { turns: 0, time: LIMITS.check_timeout },
            outcome: OutcomeSpec {
                change: Some(ChangeSpec {
                    checks_must_pass: true,
                    fields: Box::new([
                        FieldRule { name: b"title".as_slice().into(), max: 1024 },
                        FieldRule { name: b"body".as_slice().into(), max: 1024 },
                    ]),
                }),
                verdicts: Box::new([]),
                report: None,
                failure: None,
            },
            ..charter()
        },
    ));
    assert_eq!(harness.domain.runs(), 1);
}

#[test]
fn zero_activation_has_its_own_refusal_before_admission() {
    let mut harness = Harness::new(LIMITS);
    let requests = harness.step(Event::Start {
        messages: Box::default(),
        window: crate::Window { turns: u32::MAX, bytes: u64::MAX, largest_turn: 1 },
        reply_to: ReplyTo::new(Token::new(81)),
        host_run: Token::new(81),
        identity: None,
        activation: 0,
        charter: charter(),
        workspace: Some(workspace()),
        transcript: None,
        resumed: false,
    });
    assert_eq!(answered(requests), (81, Answer::Refused(Refusal::Invalid(Invalid::Activation))));
    assert_eq!((harness.domain.runs(), harness.domain.conversations()), (0, 0));
}

#[test]
fn invalid_limits_are_refused() {
    assert!(valid(&LIMITS));
    // Heap-total overflow alone is not constructor invalidity.
    assert!(valid(&Limits { run_bytes: u64::MAX, ..LIMITS }));
    // Internal facts need no message-derived capacity.
    assert!(valid(&Limits { messages: u32::MAX, ..LIMITS }));
    // Every yield can offer a message.
    assert!(!valid(&Limits { offer_messages: 0, ..LIMITS }));
    // The offer holds at least one maximum message.
    assert!(!valid(&Limits { offer_bytes: LIMITS.message_bytes - 1, ..LIMITS }));
    // Conflict paths fit the sealed marker vocabulary.
    assert!(!valid(&Limits { conflict_path_bytes: 4097, ..LIMITS }));
    // Run alarms must be representable.
    assert!(!valid(&Limits { runs: u32::MAX / 2 + 1, ..LIMITS }));
    // Delegated-call alarms must be representable.
    assert!(!valid(&Limits { calls: u32::MAX / 2 + 1, ..LIMITS }));
    // Their joint alarm table must be representable even when each product fits.
    assert!(!valid(&Limits { runs: u32::MAX / 2, calls: 1, ..LIMITS }));
}

#[test]
fn the_summary_instruction_and_its_preamble_fit_smiths_own_part() {
    let fixed = crate::prompt::fixed_prompt();
    assert!(fixed.ends_with(b"Summarise the work so far. The work is not finished: a new context window will continue it, opening with the task's own words. Answer with text only; call no tool. Write what the next window needs that the task does not say: what was done and found, with exact values, names and paths as they were read; what remains; and the decisions taken. Do not restate the task or its required answer. Keep within 1,000 words.What follows is your own summary of an earlier context window. The work continues in this window:"));
    assert_eq!(crate::prompt::summary_bytes(), 415);
    assert!(crate::prompt::fixed_prompt_bytes().expect("bounded own part") <= 8_192);
}

#[test]
fn a_charter_model_with_a_negative_spare_is_refused() {
    let mut selected = charter();
    selected.llm.window = 10_000;
    selected.llm.output = 1019;
    let mut harness = Harness::new(Limits { compaction_material: 982, ..LIMITS });
    let requests = harness.start(1, selected);
    let [Request::Answer { answer, .. }] = requests.as_ref() else { panic!("refused: {requests:?}") };
    assert_eq!(*answer, Answer::Refused(Refusal::Invalid(Invalid::Llm)));
}

#[test]
fn a_charter_model_with_a_spare_of_zero_is_admitted() {
    let mut selected = charter();
    selected.llm.window = 10_000;
    selected.llm.output = 1018;
    let mut harness = Harness::new(Limits { compaction_material: 982, ..LIMITS });
    let requests = harness.start(1, selected);
    let [Request::Admitted { host_run, .. }, Request::Read { .. }] = requests.as_ref() else {
        panic!("admitted: {requests:?}")
    };
    assert_eq!(*host_run, Token::new(1));
}

#[test]
fn a_sub_agent_model_with_a_negative_spare_is_refused() {
    let mut selected = agents();
    selected.models[0].window = 10_000;
    selected.models[0].output = 1019;
    let mut harness = Harness::new(Limits { compaction_material: 982, ..LIMITS });
    let requests = harness.start(1, selected);
    let [Request::Answer { answer, .. }] = requests.as_ref() else { panic!("refused: {requests:?}") };
    assert_eq!(*answer, Answer::Refused(Refusal::Invalid(Invalid::Llm)));
}

#[test]
fn each_conversation_opens_with_its_line() {
    let mut selected = agents();
    selected.llm.window = 272_000;
    selected.llm.output = 32_000;
    selected.models[0].window = 272_000;
    selected.models[0].output = 32_000;
    let mut harness = Harness::new(Limits { compaction_material: 982, ..LIMITS });
    let requests = harness.start(1, selected);
    let [Request::Admitted { run, .. }, Request::Read { .. }] = requests.as_ref() else {
        panic!("admitted: {requests:?}")
    };
    let requests = harness.step(Event::Read { owner: *run, read: Read::Missing });
    let [Request::Open { conversation: main, opening }] = requests.as_ref() else {
        panic!("main opening: {requests:?}")
    };
    assert_eq!(opening.line, 239_018);
    let main = *main;
    drop(harness.step(Event::Started { conversation: main, peer: PEER }));
    let requests = harness.step(ask(main, 7, families(true, false, false), Some(bytes(b"model-b")), None));
    let [Request::Open { opening, .. }] = requests.as_ref() else { panic!("child opening: {requests:?}") };
    assert_eq!(opening.line, 239_018);
}

#[test]
fn main_effort_names_obey_the_receiving_bound() {
    let mut selected = charter();
    selected.llm.effort = Some(bytes(b"deep"));
    let requests = Harness::new(Limits { effort_bytes: 4, ..LIMITS }).start(1, selected);
    let [Request::Admitted { .. }, ..] = requests.as_ref() else { panic!("exact effort bound admits") };
    let mut selected = charter();
    selected.llm.effort = Some(bytes(b"deep"));
    assert_eq!(
        answered(Harness::new(Limits { effort_bytes: 3, ..LIMITS }).start(1, selected)),
        (1, Answer::Refused(Refusal::Invalid(Invalid::Llm)))
    );
}

#[test]
fn child_effort_names_obey_the_receiving_bound() {
    let mut selected = agents();
    selected.models[0].effort = Some(bytes(b"deep"));
    let requests = Harness::new(Limits { effort_bytes: 4, ..LIMITS }).start(1, selected);
    let [Request::Admitted { .. }, ..] = requests.as_ref() else { panic!("exact effort bound admits") };
    let mut selected = agents();
    selected.models[0].effort = Some(bytes(b"deep"));
    assert_eq!(
        answered(Harness::new(Limits { effort_bytes: 3, ..LIMITS }).start(1, selected)),
        (1, Answer::Refused(Refusal::Invalid(Invalid::Llm)))
    );
}

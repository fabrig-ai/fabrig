use crate::worker_route;
use jig_domain_host as host;
use smith_host_domain as agent;

#[test]
fn a_call_name_is_encoded_as_smiths_channel_record() {
    let config = fixtures::agent();
    let name = agent::CallName { activation: 7, completion: 2, position: 3 };
    let encoded = worker_route::encode(temper_worker_domain::Encodable::CallName(name), &config);
    let temper_worker_domain::Encoded::CallName(bytes) = encoded else { panic!("call name encoded") };
    let record = smith_channel::CallName::decode(&config.limits.bodies, &mut Reader::new(&bytes)).unwrap();
    assert_eq!((record.activation(), record.completion(), record.position()), (7, 2, 3));
}

#[test]
fn a_process_exit_keeps_its_owner_and_status() {
    let owner = Token::new(9);
    assert_eq!(
        worker_route::process(ProcessEvent::Exited { agent: owner, exit: Some(skein_io::kernel::Exit::Code(3)) }),
        Some(temper_worker_domain::Event::Process(Box::new(agent::process::Event::Exited {
            owner,
            exit: Some(agent::Exit::Code(3)),
        }))),
    );
}

use smith_host_protocol::{ProcessEvent, Received};

const AGENT: Token = Token::new(71);
const RUN: Token = Token::new(43);
const TURN: u32 = 7;
const RETRY: u32 = 3;
// An offset wider than the core keeps must still cross this hand-out whole.
const OFFSET: u64 = 1_u64 << 32;

fn call() -> smith_channel::LiveCall {
    smith_channel::LiveCall::new(
        &smith_channel::CEILINGS,
        smith_channel::LiveCallParts { id: b"provider-call".as_slice().into(), tool: b"tool".as_slice().into() },
    )
    .expect("bounded call")
}

fn live(kind: smith_channel::LiveKind, call: Option<smith_channel::LiveCall>) -> ProcessEvent {
    ProcessEvent::Channel {
        agent: AGENT,
        event: Box::new(Received::Live {
            turn: TURN,
            retry: RETRY,
            kind,
            offset: OFFSET,
            bytes: b"piece".as_slice().into(),
            call,
        }),
    }
}

#[test]
fn each_live_kind_keeps_every_field() {
    for (kind, call, expected) in [
        (smith_channel::LiveKind::Text, None, host::LiveKind::Text),
        (smith_channel::LiveKind::Reasoning, None, host::LiveKind::Reasoning),
        (
            smith_channel::LiveKind::Input,
            Some(call()),
            host::LiveKind::Input { call: b"provider-call".as_slice().into(), tool: b"tool".as_slice().into() },
        ),
    ] {
        assert_eq!(
            worker_route::process(live(kind, call)),
            Some(temper_worker_domain::Event::Live {
                agent: AGENT,
                piece: host::Piece {
                    turn: TURN,
                    retry: RETRY,
                    kind: expected,
                    offset: OFFSET,
                    bytes: b"piece".as_slice().into()
                },
            })
        );
    }
}

#[test]
fn a_live_kind_disagreeing_with_its_call_is_dropped() {
    for (kind, call) in [
        (smith_channel::LiveKind::Text, Some(call())),
        (smith_channel::LiveKind::Reasoning, Some(call())),
        (smith_channel::LiveKind::Input, None),
    ] {
        assert_eq!(worker_route::process(live(kind, call)), None);
    }
}

#[test]
fn the_start_passes_each_ask_and_omits_an_empty_one() {
    let agent = fixtures::agent();
    for text in [false, true] {
        for reasoning in [false, true] {
            for input in [false, true] {
                let jig_protocol_worker_link::Request::Down { down: host::LinkDown::Assign { assignment }, .. } =
                    fixtures::chat_assignment()
                else {
                    panic!("fixture assignment")
                };
                let source = start(assignment.assignment, host::LiveAsk { text, reasoning, input });
                assert_eq!(
                    worker_route::decode(RUN, source, &agent).expect("decoded fixture").live,
                    (text || reasoning || input).then_some(agent::channel::LiveAsk { text, reasoning, input })
                );
            }
        }
    }
}

#[test]
fn a_resolution_past_its_workers_charter_allowance_starts_cut() {
    // Standard hub, smith's floor, and the reference agent
    // (temper/docs/design/domain/agent/tools.md; temper/docs/design/protocol/configuration/settings.md, 7).
    // The smaller hub proves its bound still holds.
    const STANDARD: u64 = 262_144;
    const FLOOR: u64 = 65_536;
    const REFERENCE: u64 = 127_480;
    // Both sides of a large generated file, past each receiving allowance.
    const PULL_BYTES: usize = 200_000;
    const STOCK_PAYLOAD: u32 = 10_662;
    const STOCK_OUTPUT: u32 = 8192;
    let stock = fixtures::parsed_agent(fixtures::stock_configuration(STOCK_PAYLOAD, STOCK_OUTPUT).unwrap());
    assert_eq!(crate::assignment::WORKER_CHARTER_BYTES, stock.host_limits.charter_bytes);
    // The smaller hub holds the fixture's fixed charter, title and one cut line.
    let mut minimum = temper_engine_smith::charter(&fixtures::resolution_with_pull(PULL_BYTES));
    minimum.brief.sections.first_mut().unwrap().text =
        b"[cut 200000 bytes; whole: read_pull/read_pull_files]\n".as_slice().into();
    let encoded = smith_protocol_translate::encode_charter(
        &minimum,
        &smith_charter::CEILINGS,
        &fixtures::encoding(&fixtures::engine()).endpoints,
    )
    .unwrap();
    let smaller_hub = minimum.owned_bytes().unwrap().max(skein_lib::len_u64(encoded.len()));
    assert!(smaller_hub < FLOOR);
    let rows = [
        ("engine_alone_under_standard", STANDARD, None, FLOOR, FLOOR, 0),
        ("local_agent_takes_more", STANDARD, Some(REFERENCE), REFERENCE, REFERENCE, FLOOR),
        ("hub_below_its_agent", smaller_hub, Some(REFERENCE), REFERENCE, smaller_hub, 0),
    ];
    let mut outcomes = skein_lib::List::with_capacity(u32::try_from(rows.len()).unwrap());
    for (name, hub, local, worker, bound, more_than) in rows {
        let mut settings = fixtures::engine();
        settings.jig.limits.hub.charter_bytes = hub;
        settings.agent_charter_bytes = local;
        let encoding = fixtures::encoding(&settings);
        let Some(jig_protocol_engine_workers::Event::Down { down: host::LinkDown::Assign { assignment }, .. }) =
            crate::assignment::down(
                fixtures::resolution_with_pull(PULL_BYTES),
                &fixtures::forge_table(&fixtures::forges()),
                &encoding,
                &fixtures::values_with_identity(1, fixtures::FORGE_TOKEN),
                RUN,
                skein_lib::Time::ZERO,
            )
        else {
            panic!("{name}: encoded resolution")
        };
        let encoded = skein_lib::len_u64(assignment.assignment.charter.len());
        let live = assignment.assignment.live;
        let source = start(assignment.assignment, live);
        let mut agent = fixtures::agent();
        agent.host_limits.charter_bytes = worker;
        match worker_route::decode(RUN, source, &agent) {
            Ok(start) => {
                assert!(encoded <= bound, "{name}: encoded charter exceeds the receiving bound");
                assert!(encoded > more_than, "{name}: larger local allowance keeps more of the brief");
                let [section] = start.charter.value().brief.sections.as_ref() else {
                    panic!("{name}: pull section retained")
                };
                assert_eq!(section.title.as_ref(), b"Pull request", "{name}");
                assert!(section.text.ends_with(b" bytes; whole: read_pull/read_pull_files]\n"), "{name}");
                outcomes.push((name, Ok(()))).unwrap();
            }
            Err(why) => outcomes.push((name, Err(why))).unwrap(),
        }
    }
    assert_eq!(outcomes.as_slice(), &rows.map(|(name, ..)| (name, Ok(()))));
}

fn start(assignment: host::RunAssignment, live: host::LiveAsk) -> host::Start {
    host::Start {
        live,
        workspace: None,
        charter: assignment.charter,
        activation: 1,
        identity: assignment.identity,
        turns: Box::new([]),
        answered: Box::new([]),
        messages: Box::new([]),
        grants: Box::new([]),
    }
}

use super::fixtures;
use crate::identities::{IdentityTable, Unfit};
use crate::translate;
use jig_protocol_worker_wire::Part;
use skein_lib::{Reader, Secret, Token};
use temper_protocol_worker_wire::{self as payload, v1};
use temper_worker_domain::wire;

#[test]
fn an_engine_s_items_reach_the_worker_s_workspace_field_for_field() {
    let assignment = fixtures::assignment_with(fixtures::repository(3, b"acme", b"app"));
    let limits = fixtures::wire_limits();
    let items = translate::items(
        &assignment,
        &fixtures::forge_table(&fixtures::forges()),
        &crate::forge::tokens::Tokens::empty(),
        &limits,
    )
    .unwrap()
    .unwrap();
    for program in [None, Some(Box::from(b"/fake/git".as_slice()))] {
        let mut fields = items.clone().into_parts();
        let mut repositories = skein_lib::List::with_capacity(1);
        for repository in fields.repositories.into_boxed() {
            let mut fields = repository.into_parts();
            fields.git = program.clone();
            repositories.push(v1::Repository::new(&limits, fields).unwrap()).unwrap();
        }
        fields.repositories = repositories;
        let record = v1::Items::new(&limits, fields).unwrap();
        let parts = [Part { token: Token::new(9), bytes: payload::items(&record) }];
        let workspace = worker_route::items(&fixtures::worker_assignment(), &parts, &limits).unwrap();
        let mut expected = fixtures::one_repository();
        expected.repositories[0].git = program;
        expected.repositories[0].save = Some(b"temper/7/s7".as_slice().into());
        assert_eq!(workspace, expected);
    }
    assert_eq!(translate::workspace(None), wire::Workspace { repositories: Box::new([]) });
}

#[test]
fn items_whose_bytes_do_not_decode_are_undecodable() {
    let assignment = fixtures::worker_assignment();
    let limits = fixtures::wire_limits();
    for (row, parts) in [
        ("missing", Box::<[Part]>::default()),
        ("malformed", Box::from([Part { token: Token::new(9), bytes: Box::from([255, 0]) }])),
        ("wrong token", Box::from([Part { token: Token::new(8), bytes: Box::from([255, 0]) }])),
    ] {
        assert_eq!(worker_route::items(&assignment, &parts, &limits), Err(wire::Undecodable), "{row}");
    }
}

#[test]
fn left_and_saved_go_up_under_the_tokens_the_answer_names() {
    let work = wire::Work {
        landed: Box::new([wire::Landed { tag: 3, commit: [7; 32] }]),
        saved: Some(Box::new([wire::SavedRepository { tag: 3, from_saved: true, landing: wire::Landing::Moved }])),
    };
    let answer = fixtures::answer(host::Ending::Parked {
        work: host::Work { left: Some(Token::new(11)), saved: Some(Token::new(12)) },
    });
    let host::LinkUp::Answer { answer, .. } = answer else { panic!("answer fixture") };
    let parts = worker_route::answer_parts(&answer, &work, None);
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0].token, Token::new(11));
    let left = v1::Left::decode(&fixtures::wire_limits(), &mut Reader::new(&parts[0].bytes)).unwrap();
    let head = left.landed().get(0).unwrap();
    assert_eq!((head.tag(), head.commit()), (3, &[7; 32]));
    assert_eq!(parts[1].token, Token::new(12));
    let saved = v1::Saved::decode(&fixtures::wire_limits(), &mut Reader::new(&parts[1].bytes)).unwrap();
    let row = saved.repositories().get(0).unwrap();
    assert_eq!((row.tag(), row.from_saved(), row.landing()), (3, true, &v1::Landing::Moved));
    let (_, saved) = translate::work(&wire::Work { landed: Box::new([]), saved: None });
    assert_eq!(saved, None);
}

#[test]
fn a_permanent_preparation_goes_up_by_tag_and_a_transient_one_not() {
    let limits = fixtures::wire_limits();
    let work = wire::Work { landed: Box::new([]), saved: None };
    for (row, why, expected) in preparation_rows() {
        let answer = fixtures::answer(host::Ending::Failed {
            failure: host::Failure::Unprepared(host::Preparation::Permanent { resource: Some(Token::new(13)) }),
            detail: Box::new([]),
            work: host::Work { left: None, saved: None },
        });
        let host::LinkUp::Answer { answer, .. } = answer else { panic!("answer fixture") };
        let parts = worker_route::answer_parts(&answer, &work, Some(&why));
        let decoded = match parts.first() {
            Some(part) => {
                assert_eq!(part.token, Token::new(13), "{row}");
                Some(v1::Preparation::decode(&limits, &mut Reader::new(&part.bytes)).unwrap())
            }
            None => None,
        };
        assert_eq!(decoded, expected, "{row}");
    }
}

#[test]
fn an_identity_breaking_a_field_rule_is_dropped() {
    let limits = fixtures::wire_limits();
    for (row, login, name, email, expected) in [
        ("valid", b"temper-bot".as_slice(), b"Temper Bot".as_slice(), b"bot@example.test".as_slice(), true),
        ("login colon", b"bot:bad", b"Bot", b"bot@test", false),
        ("login control", b"bot\t", b"Bot", b"bot@test", false),
        ("name empty", b"bot", b"", b"bot@test", false),
        ("name newline", b"bot", b"Bot\n", b"bot@test", false),
        ("name NUL", b"bot", b"Bot\0", b"bot@test", false),
        ("email empty", b"bot", b"Bot", b"", false),
        ("email bracket", b"bot", b"Bot", b"<bot@test>", false),
    ] {
        let record = v1::Identity::new(
            &limits,
            v1::IdentityParts {
                kind: v1::Kind::Forgejo(
                    v1::Token::new(&limits, v1::TokenParts { token: Secret::new(b"credential".as_slice().into()) })
                        .unwrap(),
                ),
                login: login.into(),
                name: name.into(),
                email: email.into(),
            },
        )
        .unwrap();
        let value = worker_route::identity(&payload::identity(&record), &limits);
        assert_eq!(value.is_some(), expected, "{row}");
        if expected {
            assert_eq!(value, Some(crate::identities::Value::Forgejo(fixtures::identity(b"credential"))), "{row}");
        }
    }
    assert_eq!(worker_route::identity(&Secret::new(Box::from([255, 0])), &limits), None, "malformed");
}

#[test]
fn a_late_older_generation_leaves_the_newest_in_place() {
    let mut table = IdentityTable::new(2);
    for (generation, token, newest) in [
        (2, b"tok-2".as_slice(), b"tok-2".as_slice()),
        (1, b"tok-1", b"tok-2"),
        (3, b"tok-3", b"tok-3"),
        (1, b"late-1", b"tok-3"),
        (3, b"again-3", b"again-3"),
    ] {
        table
            .granted(
                1,
                generation,
                Some(crate::identities::Value::Forgejo(fixtures::identity(token))),
                skein_lib::Time::from_nanos(u64::MAX),
            )
            .unwrap();
        assert_eq!(
            table.newest(1, fixtures::NOW),
            Some(&crate::identities::Value::Forgejo(fixtures::identity(newest))),
            "generation {generation}"
        );
    }
    assert_eq!(table.newest(2, fixtures::NOW), None);
    table.granted(u32::from(u16::MAX), 1, None, fixtures::NOW).unwrap();
    for number in [0, 3] {
        assert_eq!(
            table.granted(
                number,
                1,
                Some(crate::identities::Value::Forgejo(fixtures::identity(b"bad"))),
                skein_lib::Time::from_nanos(u64::MAX)
            ),
            Err(Unfit),
            "number {number}"
        );
    }
}

fn preparation_rows() -> [(&'static str, wire::Preparation, Option<v1::Preparation>); 5] {
    let limits = fixtures::wire_limits();
    [
        (
            "missing repository",
            wire::Preparation::Missing { tag: 3, missing: wire::Missing::Repository },
            Some(v1::Preparation::Missing(
                v1::Missed::new(&limits, v1::MissedParts { tag: 3, missing: v1::Missing::Repository }).unwrap(),
            )),
        ),
        (
            "missing branch",
            wire::Preparation::Missing { tag: 7, missing: wire::Missing::Branch },
            Some(v1::Preparation::Missing(
                v1::Missed::new(&limits, v1::MissedParts { tag: 7, missing: v1::Missing::Branch }).unwrap(),
            )),
        ),
        (
            "missing commit",
            wire::Preparation::Missing { tag: 9, missing: wire::Missing::Commit },
            Some(v1::Preparation::Missing(
                v1::Missed::new(&limits, v1::MissedParts { tag: 9, missing: v1::Missing::Commit }).unwrap(),
            )),
        ),
        (
            "refused",
            wire::Preparation::Refused { tag: 4 },
            Some(v1::Preparation::Refused(v1::Tag::new(&limits, v1::TagParts { tag: 4 }).unwrap())),
        ),
        ("transient", wire::Preparation::Transient, None),
    ]
}

#[test]
#[expect(clippy::disallowed_macros, reason = "formatting Debug is necessary to prove it omits command secrets")]
fn program_resource_debug_exposes_only_counts() {
    let resources = crate::program_sources::Resources::new(
        smith_protocol_program::Programs {
            claude: b"/private/program".as_slice().into(),
            directory: b"/private/logins".as_slice().into(),
        },
        Box::new([Box::from(b"API_KEY=private-value".as_slice())]),
        crate::settings::PROGRAM_PATH_BYTES,
        1000,
        Some(skein_io::kernel::Fd::new(8)),
        None,
    )
    .expect("explicit command resources fit");
    assert_eq!(format!("{resources:?}"), "Resources { programs: 1, environment_entries: 1 }");
}

#[test]
fn an_empty_github_identity_is_decoded_and_kept() {
    let limits = fixtures::wire_limits();
    let record = v1::Identity::new(
        &limits,
        v1::IdentityParts {
            kind: v1::Kind::Github(
                v1::Entries::new(&limits, v1::EntriesParts { entries: skein_lib::List::with_capacity(0) }).unwrap(),
            ),
            login: b"x-access-token".as_slice().into(),
            name: b"Temper".as_slice().into(),
            email: b"bot@example.test".as_slice().into(),
        },
    )
    .unwrap();
    let secret = payload::identity(&record);
    assert!(payload::valid(&record));
    let expected = worker_route::identity(&secret, &limits).expect("valid GitHub value");
    let crate::identities::Value::GitHub { entries, .. } = &expected else { panic!("GitHub kind") };
    assert!(entries.is_empty());
    let mut worker = fixtures::worker();
    worker.link(
        fixtures::NOW,
        jig_protocol_worker_link::Request::Value {
            account: 1,
            generation: 1,
            expires: skein_lib::Time::from_nanos(u64::MAX),
            value: jig_protocol_worker_link::GrantValue::Application { secret },
        },
    );
    assert_eq!(worker.undecodable_values, 0);
    assert_eq!(worker.identities.newest(1, fixtures::NOW), Some(&expected));
}

#[test]
fn lazy_fetch_routing_selects_the_repository_owners_identity() {
    use temper_worker_domain_checkout::git;
    let at = git::Place { workspace: Token::new(1), repository: b"app".as_slice().into(), git: None };
    let commit = git::Commit::new([1; 32]);
    let value = crate::identities::Value::GitHub {
        login: b"x-access-token".as_slice().into(),
        name: b"Bot".as_slice().into(),
        email: b"bot@example.test".as_slice().into(),
        entries: skein_lib::List::from_boxed(Box::new([
            crate::identities::Entry { account: 10, token: Secret::new(b"other".as_slice().into()) },
            crate::identities::Entry { account: 22, token: Secret::new(b"owner".as_slice().into()) },
        ])),
    };
    for op in [
        git::Op::CheckOut { at: at.clone(), commit, clean: true, identity: 3, account: Some(22) },
        git::Op::Merge { at, theirs: commit, identity: 3, account: Some(22) },
    ] {
        assert_eq!(crate::worker_git::identity(&op), Some(3));
        let selected = crate::worker_git::pick(Some(&value), &op).expect("owner selected").expect("identity value");
        assert_eq!(selected.token.expose(), b"owner");
    }
}

//! List counts respect visibility; keyed retries retain their first answer
//! (jig/docs/design/client/domain/contract/requests.md, 1, and jig/docs/design/client/domain/contract/documents.md, 1).
use super::*;

#[test]
fn me_adds_the_largest_visible_list_omission_to_the_charters_count() {
    for (party, role, expected) in [
        (ALICE, Some(people::Role::Owner), 8),
        (ALICE, Some(people::Role::Observer), 8),
        (ALICE, None, 4),
        (BOB, None, 100),
    ] {
        let mut records = mixed();
        for record in &mut records {
            if let accounts::Stored::Account { number, list, .. } = record {
                *list = Some(accounts::ListRecord {
                    names: 0,
                    read: None,
                    tried: None,
                    failure: None,
                    left_out: match *number {
                        OWN => 4,
                        DEPLOYMENT => 8,
                        OTHER => 100,
                        other => panic!("unexpected account {other}"),
                    },
                });
            }
        }
        let (mut core, mut env) = fixture_lists(records, true);
        let mut out = Queue::uncapped();
        people::step(
            &mut core.people,
            &env.with(env.limits.people),
            people::Event::Roles {
                project: people::DEPLOYMENT,
                holdings: match role {
                    Some(role) => Box::new([people::Holding { person: party, role }]),
                    None => Box::new([]),
                },
            },
            &mut out,
        );
        // One extra catalogue model is omitted by a rest charter with no alternative room.
        let mut added = crate::settings::catalogue::entries(&core.current)[0].clone();
        added.number = 2;
        added.model.name = b"another".as_slice().into();
        core.current
            .defaults
            .insert(
                crate::settings::Setting::Model(2),
                crate::settings::Default { value: crate::settings::Value::Model(Box::new(added)), revision: 1 },
            )
            .unwrap();
        core.current.test_charter_mut(1).rest = true;
        core.current.test_templates(Box::new([Template {
            number: 1,
            executor: people::TemplateExecutor::Charter(1),
            name: b"chat".as_slice().into(),
            contract: tasks::Contract::Report { words: 32 },
            shallower: 1,
            budget: people::TemplateBudget::Given,
            deadline: people::TemplateDeadline::Given,
            tracked: None,
            flags: tasks::Flags {
                ended_by: tasks::EndedBy::Result,
                conversation: false,
                listed: true,
                retention: tasks::Retention::Kept,
            },
            label: 0,
            grace: None,
            llm: None,
            resources: Box::new([]),
        }]));
        env.limits.settings.alternatives = 0;
        env.limits.documents.lines = 1;
        if party == BOB {
            people::step(
                &mut core.people,
                &env.with(env.limits.people),
                people::Event::Roles {
                    project: 1,
                    holdings: Box::new([people::Holding { person: BOB, role: people::Role::Member }]),
                },
                &mut out,
            );
        }
        let Document::Frame(frame) =
            make(&core, &env.limits, party, &Subject::Frame { party }, &window(), env.wall).unwrap()
        else {
            panic!("me")
        };
        assert_eq!(frame.projects.get(0).unwrap().left_out, expected + 1, "party {party}, deployment role {role:?}");
    }
}

#[test]
fn read_lists_retains_its_count_and_a_retry_starts_no_second_read() {
    for provider in [PROVIDER, 99] {
        let (mut core, env) = fixture_lists(own_records(), true);
        drop(step(
            &mut core,
            &env,
            Event::Fleet(fleet::Event::Hello {
                worker: 0,
                channel: Token::new(77),
                hello: Hello {
                    jobs: Box::new([]),
                    bounds: fixtures::bounds(),
                    stop_bound: Duration::ZERO,
                    slots: 1,
                    workstreams: Box::new([]),
                    hosting: Box::new([]),
                },
            }),
        ));
        drop(step(
            &mut core,
            &env,
            Event::Fleet(fleet::Event::Logins { channel: Token::new(77), logins: Box::new([]) }),
        ));
        grant(&mut core, &env, FIRST_SOURCE);
        // Finish the automatic first read before the party presses read again.
        finish_read(&mut core, &env, 0, 1);
        let ask = people::Ask::AccountChange {
            holder: people::AccountHolder::Own,
            change: people::AccountChange::ReadLists { provider },
        };
        let expected = people::Outcome::AccountReading { count: u32::from(provider == PROVIDER) };
        for attempt in 0_u32..2 {
            let mut out = step(
                &mut core,
                &env,
                Event::People(people::Event::Ask {
                    models: Box::new([]),
                    reply_to: ReplyTo::new(Token::new(77)),
                    sign_in: ALICE_SIGN_IN,
                    key: [7; 16],
                    ask: ask.clone(),
                }),
            );
            let mut answered = 0_u32;
            let mut saved = 0_u32;
            while let Some(request) = out.pop() {
                match request {
                    Request::Held(held) => {
                        if let Held::PeopleReply { reply: people::Reply::Outcome(outcome), .. } = *held {
                            assert_eq!(outcome, expected);
                            answered += 1;
                        }
                    }
                    Request::Write(Write::Save(Record::People(people::Stored::Answer {
                        ask: kept, outcome, ..
                    }))) => {
                        assert_eq!(*kept, ask);
                        assert_eq!(outcome, expected);
                        saved += 1;
                    }
                    Request::Write(_) | Request::Now(_) | Request::Ask { .. } | Request::ProjectMade { .. } => {}
                }
            }
            assert_eq!(answered, 1);
            assert_eq!(saved, u32::from(attempt == 0));
            if attempt == 0 && provider == PROVIDER {
                finish_read(&mut core, &env, 1, 1);
            }
            core.reclaim();
        }
        // A fresh child request can start the next read: the retry left none in flight.
        let mut out = Queue::uncapped();
        accounts::step(
            &mut core.accounts,
            &env.with(env.limits.accounts),
            accounts::Event::Change {
                request: Token::new(78),
                holder: accounts::Holder::Party(ALICE),
                change: accounts::Change::ReadLists { provider },
            },
            &mut out,
        );
        let mut reads = 0_u32;
        while let Some(request) = out.pop() {
            if let accounts::Request::ReadList { read, .. } = request {
                assert_eq!(read, 2);
                reads += 1;
            }
        }
        assert_eq!(reads, u32::from(provider == PROVIDER));
    }
}

#[test]
fn a_key_first_accounts_list_takes_the_direct_way_where_configured() {
    for direct in [false, true] {
        let records =
            vec![account(OWN, accounts::Holder::Party(ALICE)), source(OWN, FIRST_SOURCE, 0, accounts::Kind::Key)];
        let (mut core, env) = fixture_lists(records, true);
        if direct {
            let mut program = core.settings.endpoints[0].clone();
            program.program = Some(b"claude_code".as_slice().into());
            core.settings.endpoints = Box::new([
                program,
                EndpointFacts {
                    providers: Box::new([]),
                    direct: Box::new([PROVIDER]),
                    number: 4,
                    dialect: 3,
                    program: None,
                },
            ]);
        }
        let mut bounds = fixtures::bounds();
        bounds.endpoints = Box::new([
            jig_domain_link::EndpointBound { endpoint: 1, window: 8192, output: 1024 },
            jig_domain_link::EndpointBound { endpoint: 4, window: 8192, output: 1024 },
        ]);
        drop(step(
            &mut core,
            &env,
            Event::Fleet(fleet::Event::Hello {
                worker: 0,
                channel: Token::new(77),
                hello: Hello {
                    jobs: Box::new([]),
                    bounds,
                    stop_bound: Duration::ZERO,
                    slots: 1,
                    workstreams: Box::new([]),
                    hosting: Box::new([]),
                },
            }),
        ));
        drop(step(
            &mut core,
            &env,
            Event::Fleet(fleet::Event::Logins { channel: Token::new(77), logins: Box::new([]) }),
        ));
        grant(&mut core, &env, FIRST_SOURCE);
        finish_read(&mut core, &env, 0, if direct { 4 } else { 1 });
    }
}

fn finish_read(core: &mut Core, env: &Env<Limits>, read: u64, expected_endpoint: u32) {
    let mut out = resume_fleet(core, env);
    let Request::Held(held) = out.pop().expect("a queued read reaches the worker") else { panic!("worker read") };
    let Held::ReadList { channel, read: sent, endpoint, grant } = *held else { panic!("worker read") };
    assert_eq!(sent, read);
    assert_eq!(endpoint, expected_endpoint);
    assert_eq!(grant.account, FIRST_SOURCE);
    assert!(out.is_empty());
    drop(step(
        core,
        env,
        Event::Fleet(fleet::Event::ListRead {
            channel,
            read,
            outcome: jig_domain_link::ListOutcome::Listed(Box::new([])),
        }),
    ));
}

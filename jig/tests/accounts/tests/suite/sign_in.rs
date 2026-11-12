//! A party's sign-in and sign-out as accounts.md, 3.3 and section 5 have them: decided by the
//! child, asked of the component, each change saved or erased in the step that makes it.
use jig_accounts_world::{
    ALICE, BOB, PROVIDER, World, account, answer, commit, provider, providers, refused, save, source, want,
};
use jig_domain_accounts::{
    Answer, Bound, DISPLAY, End, Ended, Event, Failure, FromComponent, Held, Holder, Key, Kind, Refusal, Request,
    Signing, State, Stored, ToComponent,
};
use jig_domain_link::{Form, Scheme};
use skein_lib::{Duration, Token};
use std::collections::BTreeMap;

const ACCOUNT: u32 = 5;
const SIGNED_IN_SOURCE: u32 = 6;
const KEY_SOURCE: u32 = 7;
const OTHER_ACCOUNT: u32 = 10;
const OTHER_SOURCE: u32 = 11;
const REPLACEMENT_SOURCE: u32 = 12;
const JOINED_SOURCE: u32 = 13;

/// Providers 1 and 2 offer a sign-in, 3 a key alone; none is a `none` provider, so the start
/// makes nothing.
fn fresh() -> World {
    World::with_start(
        jig_accounts_world::LIMITS,
        providers(vec![provider(PROVIDER, true, true), provider(2, true, false), provider(3, false, true)]),
        &[],
    )
}
fn started(records: Vec<Stored>) -> World {
    let mut world = fresh();
    world.restore(records);
    world.step(Event::Want);
    world
}
fn sign_in(request: u64, holder: Holder, provider: u32) -> Event {
    Event::SignIn { request: Token::new(request), holder, by: request, provider }
}
fn sign_out(request: u64, holder: Holder, source: u32) -> Event {
    Event::SignOut { request: Token::new(request), holder, source }
}
fn visited(source: u32) -> Event {
    Event::Visited { source, address: b"https://issuer.test/visit".as_slice().into() }
}
fn signed_in(source: u32, subject: &[u8], display: &[u8], declined: bool) -> Event {
    Event::SignedIn {
        source,
        subject: subject.into(),
        display: display.into(),
        declined,
        said: jig_domain_accounts::Said::default(),
    }
}
fn ended(request: u64, source: u32, end: End) -> Vec<Ended> {
    vec![Ended { request: Token::new(request), source, by: request, end }]
}
/// A sign-in starts: its source's number taken and the counter saved, the component asked, and
/// the account sign-in waits for its visit.
fn starts(_request: u64, provider: u32, source: u32) -> Vec<Request> {
    vec![save(Stored::Next { number: source + 1 }), Request::Signing(Signing::SignIn { source, provider })]
}
fn places(world: &World, account: u32) -> Vec<(u32, u32)> {
    let account = world.domain.account(account).expect("kept");
    account.sources().iter().map(|(place, source)| (*place, *source)).collect()
}

#[test]
fn a_sign_in_to_a_new_subject_makes_an_account_that_restores() {
    let mut store = BTreeMap::new();
    let mut world = started(vec![]);
    let asked = world.step(sign_in(1, ALICE, PROVIDER));
    assert_eq!(asked, starts(1, PROVIDER, 0));
    commit(&mut store, &asked);
    let made = Stored::Account {
        list: None,
        place: 0,
        uses: None,
        number: 1,
        provider: PROVIDER,
        holder: ALICE,
        subject: Box::from(&b"alice"[..]),
        display: Box::from(&b"Al"[..]),
    };
    world.step(visited(0));
    let asked = world.step(signed_in(0, b"alice", b"Al", false));
    assert_eq!(asked, vec![save(Stored::Next { number: 2 }), save(made), save(source(1, 0, 0, Kind::SignIn)), want(0)]);
    commit(&mut store, &asked);
    assert_eq!(world.ended(), ended(1, 0, End::Made { account: 1, declined: false }));
    let account = world.domain.account(1).expect("made");
    assert_eq!(
        (account.holder(), account.provider(), account.subject(), account.display()),
        (ALICE, PROVIDER, &b"alice"[..], &b"Al"[..])
    );
    assert_eq!(places(&world, 1), vec![(0, 0)]);
    assert_eq!(world.domain.source(0).expect("its source").state, State::Asking);
    let mut again = fresh();
    again.restore(store.into_values().collect());
    assert_eq!(
        again.step(Event::Want),
        vec![
            Request::Signing(Signing::Restore { source: 0, provider: PROVIDER, kind: Kind::SignIn }),
            Request::Signing(Signing::Restored),
            want(0)
        ],
        "its records restore it, and the component is asked for it"
    );
    assert_eq!((again.domain.account(1).expect("restored").display(), places(&again, 1)), (&b"Al"[..], vec![(0, 0)]));
    world.step(sign_in(2, BOB, 2));
    let long = vec![b'd'; skein_lib::length::usize_of(DISPLAY) + 1];
    world.step(visited(2));
    world.step(signed_in(2, b"bob", &long, false));
    assert_eq!(world.domain.account(3).expect("made").display(), b"", "a display name too long is not kept");
}

#[test]
fn a_sign_in_to_a_held_subject_renews_its_account() {
    let mut world = started(vec![
        account(ACCOUNT, PROVIDER, ALICE, b"alice"),
        source(ACCOUNT, SIGNED_IN_SOURCE, 0, Kind::SignIn),
        source(ACCOUNT, KEY_SOURCE, 1, Kind::Key),
        account(OTHER_ACCOUNT, 2, ALICE, b"kept"),
        source(OTHER_ACCOUNT, OTHER_SOURCE, 0, Kind::Key),
        Stored::Next { number: REPLACEMENT_SOURCE },
    ]);
    assert_eq!(world.step(sign_in(1, ALICE, PROVIDER)), starts(1, PROVIDER, REPLACEMENT_SOURCE));
    world.step(visited(REPLACEMENT_SOURCE));
    assert_eq!(
        world.step(signed_in(REPLACEMENT_SOURCE, b"alice", b"other", false)),
        vec![
            Request::Erase { key: Key::Source { account: ACCOUNT, number: SIGNED_IN_SOURCE } },
            Request::Signing(Signing::Replaced { source: SIGNED_IN_SOURCE, by: REPLACEMENT_SOURCE }),
            save(source(ACCOUNT, REPLACEMENT_SOURCE, 0, Kind::SignIn)),
            want(REPLACEMENT_SOURCE),
        ],
        "the new source takes the replaced one's place; the component revokes its record where it must"
    );
    assert_eq!(world.ended(), ended(1, REPLACEMENT_SOURCE, End::Renewed { account: ACCOUNT, declined: false }));
    assert_eq!(places(&world, ACCOUNT), vec![(0, REPLACEMENT_SOURCE), (1, KEY_SOURCE)]);
    assert!(world.domain.source(SIGNED_IN_SOURCE).is_none());
    assert_eq!(world.domain.account(ACCOUNT).expect("kept").display(), b"", "the holder's display name stays");
    assert_eq!(world.step(sign_in(2, ALICE, 2)), starts(2, 2, JOINED_SOURCE));
    world.step(visited(JOINED_SOURCE));
    assert_eq!(
        world.step(signed_in(JOINED_SOURCE, b"kept", b"", false)),
        vec![save(source(OTHER_ACCOUNT, JOINED_SOURCE, 1, Kind::SignIn)), want(JOINED_SOURCE)]
    );
    assert_eq!(world.ended(), ended(2, JOINED_SOURCE, End::Renewed { account: OTHER_ACCOUNT, declined: false }));
    assert_eq!(
        places(&world, OTHER_ACCOUNT),
        vec![(0, OTHER_SOURCE), (1, JOINED_SOURCE)],
        "with no signed-in source, it joins after the last"
    );
}

#[test]
fn a_sign_in_to_a_subject_another_holder_holds_is_refused_and_revoked() {
    let mut world =
        started(vec![account(ACCOUNT, PROVIDER, ALICE, b"alice"), source(ACCOUNT, SIGNED_IN_SOURCE, 0, Kind::SignIn)]);
    assert_eq!(world.step(sign_in(3, BOB, PROVIDER)), starts(3, PROVIDER, 7));
    world.step(visited(7));
    assert_eq!(world.step(signed_in(7, b"alice", b"", false)), vec![Request::Signing(Signing::SignOut { source: 7 })]);
    assert_eq!(world.ended(), ended(3, 7, End::Refused { refusal: Refusal::Held { provider: PROVIDER } }));
    assert_eq!((world.domain.accounts(), places(&world, ACCOUNT)), (1, vec![(0, SIGNED_IN_SOURCE)]));
    world.step(sign_in(4, BOB, PROVIDER));
    world.step(visited(8));
    assert_eq!(world.step(signed_in(8, b"", b"", false)), vec![Request::Signing(Signing::SignOut { source: 8 })]);
    assert_eq!(world.ended(), ended(4, 8, End::Refused { refusal: Refusal::Subject }));
}

#[test]
fn plan_usage_declined_keeps_the_sign_in_and_refuses_the_source() {
    let mut world = started(vec![]);
    world.step(sign_in(1, ALICE, PROVIDER));
    let refused = Stored::Source {
        account: 1,
        number: 0,
        place: 0,
        kind: Kind::SignIn,
        name: Box::new([]),
        held: Some(Held::Refused),
    };
    world.step(visited(0));
    let asked = world.step(signed_in(0, b"alice", b"", true));
    assert_eq!(asked[2..], [save(refused), want(0)]);
    assert_eq!(world.ended(), ended(1, 0, End::Made { account: 1, declined: true }));
    let granted = FromComponent::Granted {
        form: Form::Credential(Scheme::Bearer),
        account: 0,
        generation: 1,
        valid: Duration::from_secs(100),
    };
    world.step(Event::Component(granted));
    let fact = world.domain.source(0).expect("kept");
    assert_eq!((fact.state, fact.usable, fact.attention), (State::Refused, false, true), "until its holder acts");
}

#[test]
fn each_sign_in_refusal_names_its_bound_or_rule() {
    let mut world = fresh();
    assert_eq!(world.step(sign_in(1, ALICE, PROVIDER)), refused(1, Refusal::Closing), "before the start");
    world.step(Event::Want);
    assert_eq!(world.step(sign_in(2, ALICE, 3)), refused(2, Refusal::Provider { provider: 3 }));
    assert_eq!(world.step(sign_in(3, ALICE, 9)), refused(3, Refusal::Provider { provider: 9 }));
    assert_eq!(world.step(sign_in(4, ALICE, PROVIDER)), starts(4, PROVIDER, 0));
    assert_eq!(world.step(sign_in(5, BOB, 2)), refused(5, Refusal::Limit { bound: Bound::SignIns }));
    assert_eq!(
        world.step(Event::SignInFailed { source: 0, failure: Failure::Cancelled }),
        refused(4, Refusal::NotStarted)
    );
    assert_eq!(world.ended(), ended(4, 0, End::Failed { failure: Failure::Cancelled }));
    world.step(visited(0));
    assert_eq!(world.step(signed_in(0, b"alice", b"", false)), vec![], "an ended sign-in makes nothing");
    assert_eq!(world.step(sign_in(6, BOB, 2)), starts(6, 2, 1), "its room is free again");
    let mut world = started(vec![Stored::Next { number: u32::MAX }]);
    assert_eq!(world.step(sign_in(7, ALICE, PROVIDER)), refused(7, Refusal::Numbers));
}

#[test]
fn a_sign_out_removes_the_source_and_answers_once_revoked() {
    let mut world = started(vec![
        account(ACCOUNT, PROVIDER, ALICE, b"alice"),
        source(ACCOUNT, SIGNED_IN_SOURCE, 0, Kind::SignIn),
        source(ACCOUNT, KEY_SOURCE, 1, Kind::Key),
        account(OTHER_ACCOUNT, 2, ALICE, b"solo"),
        source(OTHER_ACCOUNT, OTHER_SOURCE, 0, Kind::SignIn),
    ]);
    assert_eq!(
        world.step(sign_out(1, BOB, SIGNED_IN_SOURCE)),
        refused(1, Refusal::Unknown),
        "only its holder signs it out"
    );
    assert_eq!(world.step(sign_out(1, ALICE, 9)), refused(1, Refusal::Unknown));
    assert_eq!(world.step(sign_out(1, ALICE, KEY_SOURCE)), refused(1, Refusal::Kind { kind: Kind::Key }));
    assert_eq!(
        world.step(sign_out(2, ALICE, SIGNED_IN_SOURCE)),
        vec![
            Request::Erase { key: Key::Source { account: ACCOUNT, number: SIGNED_IN_SOURCE } },
            save(source(ACCOUNT, KEY_SOURCE, 0, Kind::Key)),
            Request::Signing(Signing::SignOut { source: SIGNED_IN_SOURCE })
        ]
    );
    assert_eq!((places(&world, ACCOUNT), world.domain.source(SIGNED_IN_SOURCE)), (vec![(0, KEY_SOURCE)], None));
    assert_eq!(
        world.step(Event::SignedOut { source: SIGNED_IN_SOURCE, confirmed: false }),
        vec![answer(2, Answer::SignedOut { confirmed: false, failed_hosts: Box::new([]) })],
        "unconfirmed, its holder disconnects the application at the provider"
    );
    assert_eq!(
        world.step(sign_out(3, ALICE, OTHER_SOURCE)),
        vec![
            Request::Erase { key: Key::Source { account: OTHER_ACCOUNT, number: OTHER_SOURCE } },
            Request::Erase { key: Key::Account { number: OTHER_ACCOUNT } },
            Request::Signing(Signing::SignOut { source: OTHER_SOURCE })
        ],
        "an account goes with its last source, as the store keeps none without one"
    );
    assert!(world.domain.account(OTHER_ACCOUNT).is_none());
    assert_eq!(
        world.step(Event::SignedOut { source: OTHER_SOURCE, confirmed: true }),
        vec![answer(3, Answer::SignedOut { confirmed: true, failed_hosts: Box::new([]) })]
    );
    assert_eq!(world.step(Event::SignedOut { source: OTHER_SOURCE, confirmed: true }), vec![], "answered once");
}

#[test]
fn an_account_sign_in_is_answered_started_with_the_address_when_the_component_visits() {
    let mut world = started(vec![]);
    assert_eq!(world.step(sign_in(1, ALICE, PROVIDER)), starts(1, PROVIDER, 0));
    let address = b"https://issuer.test/visit".as_slice().into();
    assert_eq!(world.step(visited(0)), vec![answer(1, Answer::Started { source: 0, address })]);
    assert_eq!(world.domain.facts_lost(), 0);
}

#[test]
fn a_second_account_visit_or_the_visit_of_no_account_sign_in_is_ignored() {
    let mut world = started(vec![]);
    assert_eq!(world.step(visited(0)).len(), 0);
    world.step(sign_in(1, ALICE, PROVIDER));
    world.step(visited(0));
    assert_eq!(world.step(visited(0)).len(), 0);
    assert_eq!(world.step(visited(99)).len(), 0);
    assert_eq!(world.domain.facts_lost(), 0);
}

#[test]
fn an_account_sign_in_that_ends_before_its_visit_is_answered_not_started() {
    let mut world = started(vec![]);
    world.step(sign_in(1, ALICE, PROVIDER));
    assert_eq!(
        world.step(Event::SignInFailed { source: 0, failure: Failure::Cancelled }),
        refused(1, Refusal::NotStarted)
    );
    assert_eq!(world.ended(), ended(1, 0, End::Failed { failure: Failure::Cancelled }));
    assert_eq!(world.step(visited(0)).len(), 0);
    world.step(sign_in(2, ALICE, PROVIDER));
    let requests = world.step(signed_in(1, b"alice", b"", false));
    assert_eq!(requests.first(), refused(2, Refusal::NotStarted).first());
    assert_eq!(requests.iter().filter(|r| matches!(r, Request::Answer { .. })).count(), 1);
    assert_eq!(world.ended(), ended(2, 1, End::Made { account: 2, declined: false }));
}

#[test]
fn an_account_sign_in_that_visited_is_not_answered_again_when_it_ends() {
    for fails in [false, true] {
        let mut world = started(vec![]);
        world.step(sign_in(1, ALICE, PROVIDER));
        world.step(visited(0));
        let event = if fails {
            Event::SignInFailed { source: 0, failure: Failure::Cancelled }
        } else {
            signed_in(0, b"alice", b"", false)
        };
        let requests = world.step(event);
        assert!(!requests.iter().any(|r| matches!(r, Request::Answer { .. })));
        assert_eq!(world.ended().len(), 1);
        assert_eq!(world.step(visited(0)).len(), 0);
    }
}

#[test]
fn a_close_cancels_each_account_sign_in_and_answers_it_closing_before_its_visit() {
    for visit in [false, true] {
        let mut world = started(vec![]);
        world.step(sign_in(1, ALICE, PROVIDER));
        if visit {
            world.step(visited(0));
        }
        let mut expected = vec![Request::Signing(Signing::CancelSignIn { source: 0 })];
        if !visit {
            expected.extend(refused(1, Refusal::Closing));
        }
        expected.push(Request::Component(ToComponent::Close));
        assert_eq!(world.step(Event::Close), expected);
        assert_eq!(world.step(Event::Close).len(), 0);
        assert_eq!(world.step(visited(0)).len(), 0);
        assert_eq!(world.step(Event::Component(FromComponent::Closed)), vec![Request::Closed]);
    }
}

/// Reports and source removal keep one bounded facts record (accounts.md, 3.7).
#[test]
#[expect(clippy::too_many_lines, reason = "one report table checks the bounded record and its writes together")]
fn reports_keep_only_what_the_provider_says() {
    use jig_domain_accounts::{
        AccountFact, FactValue, KIND_EMAIL, KIND_NAME, ORIGIN_PROGRAM, ORIGIN_REFRESH, ORIGIN_SIGN_IN, Said,
    };
    use skein_lib::Wall;

    #[derive(Clone, Copy)]
    enum Act {
        SignIn { new: bool, declined: bool, holder: Holder },
        Refresh(u32),
        SignOut(u32),
        LastSignOut,
        RemoveKey(u32),
    }
    struct Case {
        name: &'static str,
        held: Vec<AccountFact>,
        said: Said,
        act: Act,
        after: Vec<AccountFact>,
    }
    fn fact(kind: u16, text: &[u8], source: u32, origin: u16, seen: u64) -> AccountFact {
        AccountFact {
            kind,
            label: Box::new([]),
            value: FactValue::Text(text.into()),
            source: Some(source),
            origin,
            seen: Wall::from_nanos(seen),
        }
    }
    fn says(email: Option<&[u8]>, name: Option<&[u8]>) -> Said {
        Said { email: email.map(Into::into), name: name.map(Into::into) }
    }
    let email = fact(KIND_EMAIL, b"first@example.com", SIGNED_IN_SOURCE, ORIGIN_SIGN_IN, 1);
    let name = fact(KIND_NAME, b"A Name", SIGNED_IN_SOURCE, ORIGIN_SIGN_IN, 1);
    let refreshed = fact(KIND_EMAIL, b"next@example.com", SIGNED_IN_SOURCE, ORIGIN_REFRESH, 2);
    let program = fact(KIND_EMAIL, b"program@example.com", KEY_SOURCE, ORIGIN_PROGRAM, 1);
    let renewal = Act::SignIn { new: false, declined: false, holder: ALICE };
    let mut cases = vec![
        Case {
            name: "new sign-in",
            held: vec![],
            said: says(Some(b"first@example.com"), Some(b"A Name")),
            act: Act::SignIn { new: true, declined: false, holder: ALICE },
            after: vec![
                fact(KIND_EMAIL, b"first@example.com", 0, ORIGIN_SIGN_IN, 2),
                fact(KIND_NAME, b"A Name", 0, ORIGIN_SIGN_IN, 2),
            ],
        },
        Case {
            name: "renewal drops absent claims",
            held: vec![email.clone(), name.clone()],
            said: Said::default(),
            act: renewal,
            after: vec![],
        },
        Case {
            name: "another holder's subject is refused",
            held: vec![],
            said: says(Some(b"refused@example.com"), None),
            act: Act::SignIn { new: false, declined: false, holder: BOB },
            after: vec![],
        },
        Case {
            name: "declining plan usage keeps claims",
            held: vec![],
            said: says(Some(b"first@example.com"), None),
            act: Act::SignIn { new: false, declined: true, holder: ALICE },
            after: vec![fact(KIND_EMAIL, b"first@example.com", REPLACEMENT_SOURCE, ORIGIN_SIGN_IN, 2)],
        },
        Case {
            name: "refresh replaces only email",
            held: vec![email.clone(), name.clone()],
            said: says(Some(b"next@example.com"), None),
            act: Act::Refresh(SIGNED_IN_SOURCE),
            after: vec![refreshed.clone(), name.clone()],
        },
        Case {
            name: "same refresh preserves seen and writes nothing",
            held: vec![refreshed.clone(), name.clone()],
            said: says(Some(b"next@example.com"), None),
            act: Act::Refresh(SIGNED_IN_SOURCE),
            after: vec![refreshed, name.clone()],
        },
        Case {
            name: "gone refresh source is ignored",
            held: vec![email.clone()],
            said: says(Some(b"next@example.com"), None),
            act: Act::Refresh(99),
            after: vec![email.clone()],
        },
        Case {
            name: "absent sign-in email keeps program email",
            held: vec![program.clone()],
            said: Said::default(),
            act: renewal,
            after: vec![program.clone()],
        },
        Case {
            name: "sign-in email replaces program email",
            held: vec![program.clone()],
            said: says(Some(b"first@example.com"), None),
            act: renewal,
            after: vec![fact(KIND_EMAIL, b"first@example.com", REPLACEMENT_SOURCE, ORIGIN_SIGN_IN, 2)],
        },
        Case {
            name: "sign-out takes its facts but keeps another source's",
            held: vec![email.clone(), fact(KIND_NAME, b"Other", KEY_SOURCE, ORIGIN_PROGRAM, 1)],
            said: Said::default(),
            act: Act::SignOut(SIGNED_IN_SOURCE),
            after: vec![fact(KIND_NAME, b"Other", KEY_SOURCE, ORIGIN_PROGRAM, 1)],
        },
        Case {
            name: "sign-out erases last fact",
            held: vec![email.clone()],
            said: Said::default(),
            act: Act::SignOut(SIGNED_IN_SOURCE),
            after: vec![],
        },
        Case {
            name: "key removal takes its facts",
            held: vec![program.clone(), name.clone()],
            said: Said::default(),
            act: Act::RemoveKey(KEY_SOURCE),
            after: vec![name.clone()],
        },
        Case {
            name: "key removal erases last fact",
            held: vec![program],
            said: Said::default(),
            act: Act::RemoveKey(KEY_SOURCE),
            after: vec![],
        },
    ];
    cases.push(Case {
        name: "the last source erases the account's whole facts record once",
        held: vec![email.clone(), AccountFact { source: None, ..fact(99, b"held", KEY_SOURCE, 99, 1) }],
        said: Said::default(),
        act: Act::LastSignOut,
        after: vec![],
    });
    for origin in [ORIGIN_SIGN_IN, ORIGIN_REFRESH, ORIGIN_PROGRAM, 99] {
        let unknown = fact(99, b"kept", SIGNED_IN_SOURCE, origin, 1);
        cases.push(Case {
            name: "unknown kind follows its origin",
            held: vec![unknown.clone()],
            said: Said::default(),
            act: renewal,
            after: if origin == ORIGIN_SIGN_IN || origin == ORIGIN_REFRESH { vec![] } else { vec![unknown] },
        });
    }
    for (text, kept) in [
        (vec![], false),
        (vec![b'a'; 255], true),
        (vec![b'a'; 256], false),
        (b"bad\nclaim".to_vec(), false),
        (b"bad\x7fclaim".to_vec(), false),
    ] {
        cases.push(Case {
            name: "claim bounds and controls",
            held: vec![],
            said: says(Some(&text), Some(&text)),
            act: renewal,
            after: if kept {
                vec![
                    fact(KIND_EMAIL, &text, REPLACEMENT_SOURCE, ORIGIN_SIGN_IN, 2),
                    fact(KIND_NAME, &text, REPLACEMENT_SOURCE, ORIGIN_SIGN_IN, 2),
                ]
            } else {
                vec![]
            },
        });
    }
    let full: Vec<_> = (100..124).map(|kind| fact(kind, b"held", KEY_SOURCE, ORIGIN_PROGRAM, 1)).collect();
    for act in [renewal, Act::Refresh(SIGNED_IN_SOURCE)] {
        cases.push(Case {
            name: "a full record keeps held facts",
            held: full.clone(),
            said: says(Some(b"new@example.com"), Some(b"Name")),
            act,
            after: full.clone(),
        });
    }
    for case in cases {
        let new = matches!(case.act, Act::SignIn { new: true, .. });
        let account_number = if new { 1 } else { ACCOUNT };
        let mut records = if new {
            vec![]
        } else {
            vec![
                account(ACCOUNT, PROVIDER, ALICE, b"alice"),
                source(ACCOUNT, SIGNED_IN_SOURCE, 0, Kind::SignIn),
                source(ACCOUNT, KEY_SOURCE, 1, Kind::Key),
                Stored::Next { number: REPLACEMENT_SOURCE },
            ]
        };
        if matches!(case.act, Act::LastSignOut) {
            records.retain(|row| !matches!(row, Stored::Source { number: KEY_SOURCE, .. }));
        }
        if !case.held.is_empty() {
            records.push(Stored::Facts { account: account_number, facts: case.held.clone().into_boxed_slice() });
        }
        let mut store = records.iter().cloned().map(|row| (row.key(), row)).collect();
        let mut world = started(records);
        world.wall = Wall::from_nanos(2);
        let requests = match case.act {
            Act::SignIn { new, declined, holder } => {
                let requests = world.step(sign_in(1, holder, PROVIDER));
                commit(&mut store, &requests);
                let source = if new { 0 } else { REPLACEMENT_SOURCE };
                world.step(visited(source));
                world.step(Event::SignedIn {
                    source,
                    subject: b"alice".as_slice().into(),
                    display: Box::new([]),
                    declined,
                    said: case.said,
                })
            }
            Act::Refresh(source) => {
                world.step(Event::Component(FromComponent::Refreshed { account: source, said: case.said }))
            }
            Act::SignOut(source) => world.step(sign_out(1, ALICE, source)),
            Act::LastSignOut => world.step(sign_out(1, ALICE, SIGNED_IN_SOURCE)),
            Act::RemoveKey(source) => world.step(Event::Change {
                request: Token::new(1),
                holder: ALICE,
                change: jig_domain_accounts::Change::RemoveKey { source },
            }),
        };
        assert_eq!(world.domain.account_facts(account_number), case.after, "{}", case.name);
        let writes: Vec<_> = requests
            .iter()
            .filter(|request| {
                matches!(
                    request,
                    Request::Save { record: Stored::Facts { .. } } | Request::Erase { key: Key::Facts { .. } }
                )
            })
            .cloned()
            .collect();
        let expected = if case.held == case.after {
            vec![]
        } else if case.after.is_empty() {
            vec![Request::Erase { key: Key::Facts { account: account_number } }]
        } else {
            vec![save(Stored::Facts { account: account_number, facts: case.after.clone().into_boxed_slice() })]
        };
        assert_eq!(writes, expected, "{}: writes in the same decision", case.name);
        commit(&mut store, &requests);
        let expected = if case.after.is_empty() {
            None
        } else {
            Some(Stored::Facts { account: account_number, facts: case.after.into_boxed_slice() })
        };
        assert_eq!(
            store.get(&Key::Facts { account: account_number }),
            expected.as_ref(),
            "{}: durable facts",
            case.name
        );
    }
}

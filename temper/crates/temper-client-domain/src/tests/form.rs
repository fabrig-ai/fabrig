use super::fixtures;
use crate::{Event, Request, forge};
use skein_lib::{Queue, RequestEnvelope};

#[test]
fn a_new_form_holds_the_designs_defaults() {
    let root = fixtures::on_adoption_page();
    let form = root.forge().form().expect("form entered");
    let rows: [(forge::Text, &[u8]); 6] = [
        (forge::Text::Forge, b"1"),
        (forge::Text::Owner, b""),
        (forge::Text::Name, b""),
        (forge::Text::Prefix, b"temper/"),
        (forge::Text::Landing, b""),
        (forge::Text::Checks, b""),
    ];
    for (field, expected) in rows {
        assert_eq!(&*form.texts[field.index()].text, expected, "{field:?}");
    }
    assert_eq!(
        (form.project, form.role, form.home.on, form.ci.on),
        (fixtures::PROJECT, forge::Role::Owned, true, true)
    );
    assert_eq!((&form.pending, &form.outcome), (&None, &None));
}
#[test]
fn each_rule_the_draft_breaks_marks_its_field_and_sends_nothing() {
    let rows = [
        (fixtures::owner_empty(), Some(forge::Text::Owner), forge::Broken::Required),
        (fixtures::forge_zero(), Some(forge::Text::Forge), forge::Broken::ForgeNumber),
        (fixtures::checks_twice(), Some(forge::Text::Checks), forge::Broken::Checks),
        (fixtures::context_home(), None, forge::Broken::ContextHome),
        (fixtures::no_ci_no_check(), Some(forge::Text::Checks), forge::Broken::NoCheck),
    ];
    for (acts, place, broken) in rows {
        let mut root = fixtures::on_adoption_page();
        let sent = fixtures::act_then_adopt(&mut root, &acts);
        assert_eq!(sent, 0, "nothing sent: {broken:?}");
        assert_eq!(fixtures::marked(&root, place), Some(broken), "{broken:?} marks {place:?}");
    }
}
#[test]
fn a_value_past_its_bound_is_not_taken() {
    let mut root = fixtures::filled();
    let long = Box::from([b'x'; 257]);
    let out = fixtures::form_act(
        &mut root,
        forge::Act::Write { field: forge::Text::Owner, text: long, typed: forge::Typed::Bytes },
    );
    assert!(out.is_empty());
    let form = root.forge().form().expect("form remains");
    assert_eq!(&*form.texts[forge::Text::Owner.index()].text, b"acme");
    assert_eq!(form.texts[forge::Text::Owner.index()].written, 2, "the retained value is written back");
    assert_eq!(fixtures::marked(&root, Some(forge::Text::Owner)), Some(forge::Broken::Long { bound: 256 }));
    assert!(fixtures::form_act(&mut root, forge::Act::Adopt).is_empty(), "long write cannot send old value");
}
#[test]
fn adopt_sends_one_keyed_request_and_a_second_press_none() {
    let mut root = fixtures::filled();
    let mut out = fixtures::form_act(&mut root, forge::Act::Adopt);
    let Some(Request::Unlinked {
        request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Save { record } },
    }) = out.pop()
    else {
        panic!("saved before sent");
    };
    assert_eq!(record.scope, fixtures::PARTY, "confirmed party scopes the table, distinct from project");
    let Some(Request::Unlinked {
        request:
            jig_client::UnlinkedRequest::Down {
                down: jig_client::Down::Send { key, request: jig_client::Ask::Family { record: family, .. }, .. },
            },
    }) = out.pop()
    else {
        panic!("one keyed adoption");
    };
    assert_eq!(key, record.key);
    assert_eq!(record.request.key.asker, fixtures::FORGE_ASKER);
    assert_eq!(family.family, temper_client_protocol_wire::number(temper_client_protocol_wire::Owner::Forge));
    assert_eq!(family.version, temper_client_protocol_wire::CLIENT_VERSION);
    let adoption = (match temper_client_protocol_wire::v1::Requesting::decode(
        &forge::wire_limits(&fixtures::limits().forge).expect("wire bounds"),
        &mut skein_lib::Reader::new(&family.bytes),
    )
    .expect("adoption bytes")
    .into_parts()
    .request
    {
        temper_client_protocol_wire::v1::Request::Adopt(adopt) => adopt,
        temper_client_protocol_wire::v1::Request::SetUp(_)
        | temper_client_protocol_wire::v1::Request::RemoveForge(_)
        | temper_client_protocol_wire::v1::Request::DismissMaking(_)
        | temper_client_protocol_wire::v1::Request::MakeRepository(_)
        | temper_client_protocol_wire::v1::Request::StartPlayground
        | temper_client_protocol_wire::v1::Request::AddDevice(_)
        | temper_client_protocol_wire::v1::Request::RemoveDevice(_)
        | temper_client_protocol_wire::v1::Request::SetAway(_)
        | temper_client_protocol_wire::v1::Request::Overturn(_)
        | temper_client_protocol_wire::v1::Request::Looked(_)
        | temper_client_protocol_wire::v1::Request::TryAccount(_)
        | temper_client_protocol_wire::v1::Request::Dismiss(_)
        | temper_client_protocol_wire::v1::Request::EndPlayground
        | temper_client_protocol_wire::v1::Request::Reachable(_)
        | temper_client_protocol_wire::v1::Request::Replace(_)
        | temper_client_protocol_wire::v1::Request::Connect(_)
        | temper_client_protocol_wire::v1::Request::Accept(_)
        | temper_client_protocol_wire::v1::Request::Install(_) => {
            panic!("expected adoption")
        }
    })
    .into_parts();
    assert_eq!(
        (adoption.project, adoption.forge, &*adoption.owner, &*adoption.name),
        (1, 1, &b"acme"[..], &b"shop"[..])
    );
    assert!(out.is_empty(), "one send");
    assert!(fixtures::form_act(&mut root, forge::Act::Adopt).is_empty(), "second press sends nothing");
}
#[test]
fn an_adopted_outcome_resets_the_fields_and_stays_shown() {
    let mut root = fixtures::filled();
    let mut out = fixtures::form_act(&mut root, forge::Act::Adopt);
    let attempt = sent(&mut out);
    let limits = fixtures::limits();
    crate::step(
        &mut root,
        &fixtures::env(&limits),
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Family(fixtures::adopted_bytes())),
                    why: None,
                },
            },
        },
        &mut out,
    );
    let form = root.forge().form().expect("page stays");
    assert_eq!(form.outcome, Some(fixtures::adopted()));
    assert_eq!(&*form.texts[forge::Text::Owner.index()].text, b"");
    assert_eq!((form.home.on, form.ci.on, form.role, form.pending), (true, true, forge::Role::Owned, None));
    crate::fire(&mut root, &fixtures::env(&limits), &mut out);
    assert_eq!(root.forge().form().expect("page stays").outcome, Some(fixtures::adopted()), "outcome stays shown");
    fixtures::form_act(&mut root, fixtures::write(forge::Text::Owner, b"other", forge::Typed::Bytes));
    assert_eq!(root.forge().form().expect("page stays").outcome, None, "edit clears outcome");
}
#[test]
fn out_of_date_disables_adopt_and_keeps_the_pending_request() {
    let mut root = fixtures::filled();
    let mut out = fixtures::form_act(&mut root, forge::Act::Adopt);
    sent(&mut out);
    let (request, _) = root.forge().form().expect("page stays").pending.expect("pending adoption");
    let limits = fixtures::limits();
    let mut out = fixtures::forge_event(&mut root, forge::Event::Served { served: forge::Served::OutOfDate });
    assert_eq!(root.forge().form().expect("page stays").served, forge::Served::OutOfDate);
    assert_eq!(root.forge().form().expect("page stays").pending, Some((request, skein_lib::RequestProgress::Parked)));
    assert!(fixtures::form_act(&mut root, forge::Act::Adopt).is_empty(), "disabled sends nothing");
    crate::step(
        &mut root,
        &fixtures::env(&limits),
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::FamilyServed { family: fixtures::FORGE, versions: None },
            },
        },
        &mut out,
    );
    crate::take(&mut root, &fixtures::env(&limits), &mut out);
    assert_eq!(fixtures::sends(&mut out), 0, "out-of-date requests stay held after reconnect");
    assert!(
        root.client()
            .pending(jig_client::Key { asker: crate::owner::Owner::Forge.asker(), number: request.raw() })
            .is_some(),
        "pending adoption remains in the durable table"
    );
}
fn sent(out: &mut Queue<Request>) -> skein_lib::Token {
    for _ in 0..out.len() {
        if let Some(Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt, .. } },
        }) = out.pop()
        {
            return attempt;
        }
    }
    panic!("one send present")
}

#[test]
fn an_adoption_is_the_forges_family_bytes_under_its_asker() {
    let mut root = fixtures::filled();
    let mut out = fixtures::form_act(&mut root, forge::Act::Adopt);
    let Some(Request::Unlinked {
        request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Save { record } },
    }) = out.pop()
    else {
        panic!("kept adoption")
    };
    assert_eq!(record.request.key.asker, fixtures::FORGE_ASKER);
    let jig_client::Ask::Family { record, .. } = record.request.request else { panic!("opaque family") };
    let mut reader = skein_lib::Reader::new(&record.bytes);
    let adoption = (match temper_client_protocol_wire::v1::Requesting::decode(
        &forge::wire_limits(&fixtures::limits().forge).expect("wire bounds"),
        &mut reader,
    )
    .expect("family adoption")
    .into_parts()
    .request
    {
        temper_client_protocol_wire::v1::Request::Adopt(adopt) => adopt,
        temper_client_protocol_wire::v1::Request::SetUp(_)
        | temper_client_protocol_wire::v1::Request::RemoveForge(_)
        | temper_client_protocol_wire::v1::Request::DismissMaking(_)
        | temper_client_protocol_wire::v1::Request::MakeRepository(_)
        | temper_client_protocol_wire::v1::Request::StartPlayground
        | temper_client_protocol_wire::v1::Request::AddDevice(_)
        | temper_client_protocol_wire::v1::Request::RemoveDevice(_)
        | temper_client_protocol_wire::v1::Request::SetAway(_)
        | temper_client_protocol_wire::v1::Request::Overturn(_)
        | temper_client_protocol_wire::v1::Request::Looked(_)
        | temper_client_protocol_wire::v1::Request::TryAccount(_)
        | temper_client_protocol_wire::v1::Request::Dismiss(_)
        | temper_client_protocol_wire::v1::Request::EndPlayground
        | temper_client_protocol_wire::v1::Request::Reachable(_)
        | temper_client_protocol_wire::v1::Request::Replace(_)
        | temper_client_protocol_wire::v1::Request::Connect(_)
        | temper_client_protocol_wire::v1::Request::Accept(_)
        | temper_client_protocol_wire::v1::Request::Install(_) => {
            panic!("expected adoption")
        }
    })
    .into_parts();
    assert!(reader.is_empty());
    assert_eq!(
        (record.family, record.version, adoption.project, adoption.owner.as_ref(), adoption.name.as_ref()),
        (
            temper_client_protocol_wire::FORGE,
            temper_client_protocol_wire::CLIENT_VERSION,
            fixtures::PROJECT,
            b"acme".as_slice(),
            b"shop".as_slice()
        )
    );
}
#[test]
fn an_answer_under_the_forges_asker_reaches_the_form() {
    let mut root = fixtures::filled();
    let mut out = fixtures::form_act(&mut root, forge::Act::Adopt);
    let Some(Request::Unlinked {
        request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Save { record } },
    }) = out.pop()
    else {
        panic!("kept adoption")
    };
    let limits = fixtures::limits();
    crate::route::told(
        &mut root,
        &fixtures::env(&limits),
        jig_client::Told::Answered {
            key: record.request.key,
            outcome: jig_client::Outcome::Answered(jig_client::Answered::Family(fixtures::adopted_bytes())),
        },
        &mut out,
    );
    assert_eq!(root.forge().form().expect("form").outcome, Some(fixtures::adopted()));
}
#[test]
fn a_kept_adoption_of_another_version_is_dropped_and_its_form_shows_none_pending() {
    const OTHER_VERSION: u32 = temper_client_protocol_wire::CLIENT_VERSION + 1;
    let limits = fixtures::limits();
    let mut root = fixtures::filled();
    let mut out = fixtures::form_act(&mut root, forge::Act::Adopt);
    let Some(Request::Unlinked {
        request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Save { mut record } },
    }) = out.pop()
    else {
        panic!("kept adoption")
    };
    let key = record.request.key;
    let jig_client::Ask::Family { record: family, .. } = &mut record.request.request else { panic!("family") };
    family.version = OTHER_VERSION;
    let mut kept = skein_lib::List::with_capacity(1);
    kept.push(record).expect("one kept request");
    let mut restarted = crate::unlinked(&limits, fixtures::SEED);
    let frame = fixtures::start(&mut restarted, kept);
    fixtures::confirm(&mut restarted, frame, fixtures::PARTY, &mut out);
    crate::step(
        &mut restarted,
        &fixtures::env(&limits),
        Event::Went { place: crate::Place::Address(crate::Address::Adopt { project: fixtures::PROJECT }) },
        &mut out,
    );
    assert_eq!(restarted.client().pending(key), None);
    assert_eq!(restarted.forge().form().expect("form").pending, None);
}

#[test]
fn a_family_the_engine_does_not_serve_marks_the_form_out_of_date() {
    let mut root = fixtures::filled();
    let limits = fixtures::limits();
    let mut out = Queue::uncapped();
    crate::step(
        &mut root,
        &fixtures::env(&limits),
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::FamilyServed { family: fixtures::FORGE, versions: None },
            },
        },
        &mut out,
    );
    assert_eq!(root.forge().form().expect("form").served, forge::Served::OutOfDate);
    assert!(root.client().link().not_served.contains(&fixtures::FORGE));
    assert!(fixtures::form_act(&mut root, forge::Act::Adopt).is_empty());
}

const NEXT_FORGE_VERSION: u32 = temper_client_protocol_wire::CLIENT_VERSION + 1;
fn version_answer(limits: &jig_client_protocol_link::Limits) -> Box<[u8]> {
    use jig_client_protocol_wire as w;
    let owner = w::Owner::Family(w::Family::new(limits.wire(), w::FamilyParts { number: fixtures::FORGE }).unwrap());
    let mut served = skein_lib::List::with_capacity(1);
    served
        .push(
            w::Served::new(
                limits.wire(),
                w::ServedParts { owner: owner.clone(), first: NEXT_FORGE_VERSION, last: NEXT_FORGE_VERSION },
            )
            .unwrap(),
        )
        .unwrap();
    let answer =
        w::KeyedAnswer::NotServed(w::NotServed::new(limits.wire(), w::NotServedParts { owner, served }).unwrap());
    let mut bytes = skein_lib::Writer::new(usize::try_from(answer.measure()).unwrap());
    answer.encode(&mut bytes).unwrap();
    bytes.finish()
}
fn version_refused() -> (crate::Domain, jig_client::Key) {
    use jig_client_protocol_link as link;
    let mut root = fixtures::filled();
    let mut out = fixtures::form_act(&mut root, forge::Act::Adopt);
    let Some(Request::Unlinked {
        request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Save { record } },
    }) = out.pop()
    else {
        panic!("kept adoption")
    };
    let stable = record.request.key;
    let Some(Request::Unlinked {
        request:
            jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt, key, request: ask, value } },
    }) = out.pop()
    else {
        panic!("adoption sent")
    };
    let limits = fixtures::limits();
    let bounds = link::Limits::new(limits.client.domain, limits.client.exchanges).unwrap();
    let mut link = link::Link::new(&bounds).unwrap();
    let mut sent = Queue::uncapped();
    link.step(skein_lib::Time::ZERO, link::Event::Send { attempt, key, ask, value }, &mut sent);
    let Some(link::Request::Exchange { exchange, .. }) = sent.pop() else { panic!("keyed exchange") };
    link.step(skein_lib::Time::ZERO, link::Event::Reply { exchange, message: version_answer(&bounds) }, &mut sent);
    while let Some(request) = sent.pop() {
        let up = match request {
            link::Request::Hello { family: Some(family), versions } => {
                jig_client::Up::FamilyServed { family, versions }
            }
            link::Request::Answer { attempt, envelope, answer, why } => {
                jig_client::Up::Answer { attempt, envelope, answer, why }
            }
            // M4's Authorize place answers it
            link::Request::Approval(_) | link::Request::Authorized(_) => continue,
            link::Request::Save { .. }
            | link::Request::Erase { .. }
            | link::Request::Kept { .. }
            | link::Request::StreamsFull { .. }
            | link::Request::Dropped
            | link::Request::Exchange { .. }
            | link::Request::NotKept { .. }
            | link::Request::ServiceCredential { .. }
            | link::Request::OpenStream { .. }
            | link::Request::Signing { .. }
            | link::Request::CloseStream
            | link::Request::Watch { .. }
            | link::Request::FamilyDelivery { .. }
            | link::Request::Ended { .. }
            | link::Request::Hello { family: None, .. }
            | link::Request::Read { .. }
            | link::Request::Fact { .. } => panic!("version refusal's typed answer: {request:?}"),
        };
        crate::step(
            &mut root,
            &fixtures::env(&limits),
            Event::Client { event: jig_client::Event::Up { up } },
            &mut out,
        );
    }
    (root, stable)
}
#[test]
fn a_version_not_served_answer_marks_the_form_out_of_date() {
    let (mut root, _) = version_refused();
    assert_eq!(root.forge().form().expect("form").served, forge::Served::OutOfDate);
    assert!(fixtures::form_act(&mut root, forge::Act::Adopt).is_empty(), "disabled sends nothing");
}
#[test]
fn an_out_of_date_form_keeps_its_pending_adoption() {
    let (root, key) = version_refused();
    assert_eq!(
        root.forge().form().expect("form").pending,
        Some((skein_lib::Token::new(key.number), skein_lib::RequestProgress::Parked))
    );
    assert_eq!(root.client().pending(key).expect("kept adoption").progress, jig_client::Progress::NotServed);
}

fn first_rows(
    state: temper_client_protocol_wire::v1::FirstState,
) -> skein_lib::List<temper_client_protocol_wire::v1::ForgeRow> {
    use temper_client_protocol_wire::v1 as w;
    let mut rows = skein_lib::List::with_capacity(3);
    for (number, mode) in [(1, w::Mode::Fake), (2, w::Mode::Owned), (3, w::Mode::Owned)] {
        rows.push(
            w::ForgeRow::new(
                &w::CEILINGS,
                w::ForgeRowParts {
                    starts_with: w::StartsWith::Repository,
                    makings: skein_lib::List::with_capacity(0),
                    host: b"git.example:3000".as_slice().into(),
                    number,
                    kind: w::Kind::Forgejo,
                    mode,
                    connection: w::Connection::NotConnected,
                    offer: w::Offer::NotOffered,
                    first: Some(
                        w::FirstRepository::new(
                            &w::CEILINGS,
                            w::FirstRepositoryParts {
                                playground: number == 1,
                                org: b"work".as_slice().into(),
                                repository: b"hello".as_slice().into(),
                                state: state.clone(),
                                label: b"docker".as_slice().into(),
                            },
                        )
                        .unwrap(),
                    ),
                    installations: skein_lib::List::with_capacity(0),
                    ended: None,
                    app: None,
                    web: None,
                    repositories: skein_lib::List::with_capacity(1),
                    owned: None,
                    admin: false,
                    read_only: false,
                    ended_stop: None,
                },
            )
            .unwrap(),
        )
        .unwrap();
    }
    rows
}
fn made() -> temper_client_protocol_wire::v1::FirstState {
    use temper_client_protocol_wire::v1 as w;
    w::FirstState::Made(w::Made::new(&w::CEILINGS, w::MadeParts { id: 7 }).unwrap())
}
fn deliver_first(
    root: &mut crate::Domain,
    watch: skein_lib::Token,
    rows: skein_lib::List<temper_client_protocol_wire::v1::ForgeRow>,
) {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    crate::step(
        root,
        &fixtures::env(&fixtures::limits()),
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::FamilyDelivered {
                    watch,
                    record: jig_client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::forges(
                            &w::Forges::new(&w::CEILINGS, w::ForgesParts { forges: rows, linked: false }).unwrap(),
                        ),
                    },
                },
            },
        },
        &mut Queue::uncapped(),
    );
}
fn open_form(root: &mut crate::Domain) {
    crate::step(
        root,
        &fixtures::env(&fixtures::limits()),
        Event::Went { place: crate::Place::Address(crate::Address::Adopt { project: fixtures::PROJECT }) },
        &mut Queue::uncapped(),
    );
}
fn assert_first(form: &forge::Form) {
    for (field, text, written) in [
        (forge::Text::Forge, b"2".as_slice(), 1),
        (forge::Text::Owner, b"work", 1),
        (forge::Text::Name, b"hello", 1),
        (forge::Text::Landing, b"main", 1),
        (forge::Text::Prefix, b"temper/", 0),
        (forge::Text::Checks, b"", 0),
    ] {
        assert_eq!((form.texts[field.index()].text.as_ref(), form.texts[field.index()].written), (text, written));
    }
    assert_eq!((form.role, form.home.on, form.ci.on), (forge::Role::Owned, true, true));
    assert_eq!(forge::adoption(form, fixtures::limits().forge.checks).unwrap().forge, 2);
}
#[test]
fn the_form_opens_filled_for_the_lowest_real_unadopted_first_repository() {
    let (mut root, _, watch) = super::connect::opened(true);
    deliver_first(&mut root, watch, first_rows(made()));
    open_form(&mut root);
    assert_first(root.forge().form().unwrap());
}
#[test]
fn the_form_leaves_defaults_for_adopted_unmade_and_fake_first_repositories() {
    use temper_client_protocol_wire::v1 as w;
    let mut adopted = first_rows(made());
    let mut parts = adopted.get(0).unwrap().clone().into_parts();
    parts
        .repositories
        .push(
            w::Adoption::new(
                &w::CEILINGS,
                w::AdoptionParts { owner: b"work".as_slice().into(), name: b"hello".as_slice().into(), project: 99 },
            )
            .unwrap(),
        )
        .unwrap();
    *adopted.get_mut(0).unwrap() = w::ForgeRow::new(&w::CEILINGS, parts).unwrap();
    let mut fake = first_rows(made());
    fake.truncate(1);
    let failed = w::FirstState::Failed(
        w::CreateFailed::new(&w::CEILINGS, w::CreateFailedParts { step: w::CreateStep::Repository }).unwrap(),
    );
    for (case, rows) in [
        ("adopted", adopted),
        ("fake", fake),
        ("making", first_rows(w::FirstState::Making)),
        ("failed", first_rows(failed)),
    ] {
        let (mut root, _, watch) = super::connect::opened(true);
        deliver_first(&mut root, watch, rows);
        open_form(&mut root);
        let form = root.forge().form().unwrap();
        assert_eq!(form.texts, forge::form(fixtures::PROJECT).texts, "case {case}");
    }
}
#[test]
fn a_late_first_repository_delivery_fills_only_an_unwritten_form() {
    for written in [false, true] {
        let (mut root, _, watch) = super::connect::opened(true);
        open_form(&mut root);
        if written {
            fixtures::form_act(&mut root, fixtures::write(forge::Text::Owner, b"mine", forge::Typed::Bytes));
        }
        deliver_first(&mut root, watch, first_rows(made()));
        let form = root.forge().form().unwrap();
        if written {
            assert_eq!(form.texts[forge::Text::Owner.index()].text.as_ref(), b"mine");
            assert_eq!(form.texts[forge::Text::Landing.index()].text.as_ref(), b"");
        } else {
            assert_first(form);
        }
    }
}
#[test]
fn adopting_the_first_repository_resets_to_plain_defaults_even_on_later_deliveries() {
    let (mut root, _, watch) = super::connect::opened(true);
    deliver_first(&mut root, watch, first_rows(made()));
    open_form(&mut root);
    let mut out = fixtures::form_act(&mut root, forge::Act::Adopt);
    let attempt = sent(&mut out);
    crate::step(
        &mut root,
        &fixtures::env(&fixtures::limits()),
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Family(fixtures::adopted_bytes())),
                    why: None,
                },
            },
        },
        &mut out,
    );
    deliver_first(&mut root, watch, first_rows(made()));
    let form = root.forge().form().unwrap();
    let defaults = forge::form(fixtures::PROJECT);
    for (field, default) in form.texts.iter().zip(defaults.texts) {
        assert_eq!(field.text, default.text);
        assert!(field.written > 0);
    }
    assert_eq!(form.outcome, Some(fixtures::adopted()));
}

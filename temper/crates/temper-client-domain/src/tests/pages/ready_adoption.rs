//! Ready's adoption and Home's retry (temper/docs/design/client/README.md, 3.5).
use super::welcome::{default_answer, deliver, event, press_default, ready_default, ready_without_first};
use crate::tests::fixtures as f;
use crate::{
    Action, Address, Domain, Event, Request, forge,
    pages::{self, Act, welcome::AdoptionEnded},
};
use jig_client::{Down, Told, contract as c};
use skein_lib::{List, Queue};
use temper_client_protocol_wire::{self as wire, v1 as w};
const PROJECT: u32 = 77;
fn made() -> jig_client::Outcome {
    jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
        c::DocumentOutcome::ProjectMade { project: PROJECT },
    ))))
}
fn sends(mut out: Queue<Request>) -> List<(skein_lib::RequestKey, jig_client::Ask)> {
    let mut sent = List::with_capacity(wire::FIRST_COPIES);
    while let Some(request) = out.pop() {
        if let Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: Down::Send { key, request, .. } },
        } = request
        {
            sent.push((key, request)).unwrap();
        }
    }
    sent
}
fn project_answer(root: &mut Domain, outcome: jig_client::Outcome) -> Queue<Request> {
    let mut out = default_answer(root, outcome);
    let mut kept = Queue::uncapped();
    while let Some(request) = out.pop() {
        if let Request::Show { address, .. } = &request {
            event(root, Event::Went { place: crate::Place::Address(address.clone()) });
        }
        kept.push(request);
    }
    kept
}
fn began() -> Domain {
    let (mut root, _) = ready_default();
    press_default(&mut root);
    project_answer(&mut root, made());
    root
}
fn tell(root: &mut Domain, outcome: jig_client::Outcome) {
    let key = root.pages().adopting().unwrap().out.unwrap();
    crate::route::told(root, &f::env(&f::limits()), Told::Answered { key, outcome }, &mut Queue::uncapped());
}
fn refused(reason: w::ForgeReason) -> jig_client::Outcome {
    let answer = w::Answer::new(
        &w::CEILINGS,
        w::AnswerParts {
            outcome: w::Outcome::ForgeRefused(
                w::ForgeRefused::new(&w::CEILINGS, w::ForgeRefusedParts { reason }).unwrap(),
            ),
        },
    )
    .unwrap();
    jig_client::Outcome::Answered(jig_client::Answered::Family(jig_client::Family {
        family: wire::FORGE,
        version: wire::CLIENT_VERSION,
        bytes: wire::answer(&answer),
    }))
}
fn again(root: &mut Domain) -> List<(skein_lib::RequestKey, jig_client::Ask)> {
    sends(event(root, Event::Act { action: Action::Pages(Act::AdoptFirst) }))
}
#[test]
fn ready_sends_make_then_ctl_adoption_and_opens_home_before_the_answer() {
    let (mut root, _) = ready_default();
    let make = sends(press_default(&mut root));
    assert_eq!(make.len(), 1);
    let jig_client::Ask::Jig(ask) = &make.get(0).unwrap().1 else { panic!("MakeProject first") };
    assert_eq!(
        ask.as_ref(),
        &c::Ask::MakeProject { name: c::Text::new(b"Hello".as_slice().into(), 64).unwrap(), policy: 1 }
    );
    assert!(root.pages().adopting().is_none());
    let send = sends(project_answer(&mut root, made()));
    assert_eq!(root.pages().address(), &Address::Home { project: PROJECT });
    assert_eq!(send.len(), 1);
    let expected = forge::Adoption {
        project: PROJECT,
        forge: 3,
        owner: b"work".as_slice().into(),
        name: b"hello".as_slice().into(),
        prefix: b"temper/".as_slice().into(),
        role: forge::Role::Owned,
        home: true,
        landing: b"main".as_slice().into(),
        ci: true,
        checks: Box::default(),
    };
    assert_eq!(
        *send.get(0).unwrap(),
        (forge::first_repository_key(3, false), forge::ask(&expected, &f::limits().forge).unwrap())
    );
    let adopting = root.pages().adopting().unwrap();
    assert_eq!(adopting.project, PROJECT);
    assert_eq!(adopting.first, root.pages().welcome_first().unwrap().clone());
    assert!(adopting.out.is_some());
    assert_eq!(adopting.ended, None);
}
#[test]
fn no_first_repository_sends_no_adoption() {
    let mut root = ready_without_first();
    assert!(root.pages().welcome_first().is_none());
    press_default(&mut root);
    assert!(sends(project_answer(&mut root, made())).is_empty());
    assert_eq!(root.pages().address(), &Address::Home { project: PROJECT });
    assert!(root.pages().adopting().is_none());
    assert!(again(&mut root).is_empty());
}
#[test]
fn double_press_and_replayed_project_answer_send_only_one_adoption() {
    let (mut root, _) = ready_default();
    press_default(&mut root);
    let key = root.pages().default_project().making.unwrap();
    assert!(sends(press_default(&mut root)).is_empty());
    assert_eq!(sends(project_answer(&mut root, made())).len(), 1);
    let adopting = root.pages().adopting().unwrap().out;
    let mut out = Queue::uncapped();
    crate::route::told(&mut root, &f::env(&f::limits()), Told::Answered { key, outcome: made() }, &mut out);
    assert!(sends(out).is_empty());
    assert!(again(&mut root).is_empty());
    assert_eq!(root.pages().adopting().unwrap().out, adopting);
}
#[test]
fn adopted_ends_the_request_without_an_ending_and_keeps_home() {
    let mut root = began();
    tell(&mut root, jig_client::Outcome::Answered(jig_client::Answered::Family(f::adopted_bytes())));
    let adopting = root.pages().adopting().unwrap();
    assert_eq!(adopting.out, None);
    assert_eq!(adopting.ended, None);
    assert_eq!(root.pages().address(), &Address::Home { project: PROJECT });
}
#[test]
fn ordinary_refusals_keep_the_project_and_the_answer_for_home() {
    for (reason, expected) in [
        (w::ForgeReason::HomeHeld, forge::Reason::HomeHeld),
        (w::ForgeReason::AnotherProject, forge::Reason::AnotherProject),
    ] {
        let mut root = began();
        tell(&mut root, refused(reason));
        let adopting = root.pages().adopting().unwrap();
        assert_eq!(adopting.out, None);
        assert_eq!(adopting.ended, Some(AdoptionEnded::Refused(forge::Answer::ForgeRefused(expected))));
        assert_eq!(root.pages().address(), &Address::Home { project: PROJECT });
    }
}
#[test]
fn unanswered_adoption_keeps_the_project_and_its_unknown_ending() {
    let mut root = began();
    tell(&mut root, jig_client::Outcome::Unknown);
    let adopting = root.pages().adopting().unwrap();
    assert_eq!(adopting.out, None);
    assert_eq!(adopting.ended, Some(AdoptionEnded::Unknown));
    assert_eq!(root.pages().address(), &Address::Home { project: PROJECT });
}
#[test]
fn name_taken_folds_ascii_and_adopts_into_the_persons_project() {
    let (mut root, me) = ready_default();
    let mut frame = pages::accounts::current_me(root.client()).unwrap().clone();
    frame.projects.get_mut(0).unwrap().name =
        c::DocumentText::complete(c::Text::new(b"hello".as_slice().into(), 64).unwrap());
    let project = frame.projects.get(0).unwrap().project;
    deliver(&mut root, me, c::Delivery::Me(Box::new(frame)));
    press_default(&mut root);
    let send = sends(project_answer(
        &mut root,
        jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
            refusal: c::Refusal::NameTaken,
            at: None,
        })),
    ));
    assert_eq!(send.len(), 1);
    assert_eq!(root.pages().address(), &Address::Home { project });
    assert_eq!(root.pages().adopting().unwrap().project, project);
    tell(&mut root, refused(w::ForgeReason::HomeHeld));
    assert_eq!(
        root.pages().adopting().unwrap().ended,
        Some(AdoptionEnded::Refused(forge::Answer::ForgeRefused(forge::Reason::HomeHeld)))
    );
}
#[test]
fn each_refused_retry_gets_a_new_name_and_only_one_request_at_a_time() {
    let mut root = began();
    for (attempt, name) in [(1, b"first-repository-again-3-1"), (2, b"first-repository-again-3-2")] {
        let previous = root.pages().adopting().unwrap().out;
        tell(&mut root, refused(w::ForgeReason::HomeHeld));
        let send = again(&mut root);
        assert_eq!(send.len(), 1);
        assert_eq!(send.get(0).unwrap().0, forge::first_repository_again_key(3, attempt));
        assert_eq!(send.get(0).unwrap().0, crate::key::named(crate::key::TOOLS, &crate::key::Name::new(name).unwrap()));
        assert_ne!(root.pages().adopting().unwrap().out, previous);
        assert_eq!(root.pages().adopting().unwrap().attempt, attempt);
        assert_eq!(root.pages().adopting().unwrap().ended, None);
        assert!(again(&mut root).is_empty());
    }
}

#[test]
fn switcher_creation_opens_home_once_membership_arrives() {
    use pages::switcher;
    let (mut root, me) = ready_default();
    event(&mut root, Event::Went { place: crate::Place::Address(Address::Chats { filter: crate::ChatsFilter::All }) });
    event(
        &mut root,
        Event::Act {
            action: Action::Pages(Act::Switcher(switcher::Act::NewWords { words: b"New".as_slice().into() })),
        },
    );
    assert_eq!(
        sends(event(&mut root, Event::Act { action: Action::Pages(Act::Switcher(switcher::Act::Create)) })).len(),
        1
    );
    let key = jig_client::Key { asker: crate::owner::Owner::Pages.asker(), number: root.pages.serial };
    let mut out = Queue::uncapped();
    crate::route::told(&mut root, &f::env(&f::limits()), Told::Answered { key, outcome: made() }, &mut out);
    assert!(out.is_empty(), "the switcher waits for membership");
    let mut frame = pages::accounts::current_me(root.client()).unwrap().clone();
    frame.projects.get_mut(0).unwrap().project = PROJECT;
    let out = deliver(&mut root, me, c::Delivery::Me(Box::new(frame)));
    let mut home = false;
    for request in &out {
        home |= *request == Request::Show { address: Address::Home { project: PROJECT }, replace: false };
    }
    assert!(home);
    assert!(root.pages().adopting().is_none(), "the switcher makes a project alone");
}

const COPY_FORGE: u16 = 3;
const ADDRESSES: &[&[u8]] =
    &[b"https://github.com/acme/widget", b"https://source.invalid/team/tool", b"https://github.com/other/library"];
fn copy_row(number: u16, mode: w::Mode, addresses: &[&[u8]], project: Option<u32>) -> w::ForgeRow {
    let row = crate::tests::forges::wire_connection_row(
        number,
        w::Kind::Forgejo,
        mode,
        w::Connection::Connected(
            w::Connected::new(&w::CEILINGS, w::ConnectedParts { id: 42, login: b"temper".as_slice().into() }).unwrap(),
        ),
        w::Offer::NotOffered,
        None,
    );
    let mut parts = row.into_parts();
    let mut copies = List::with_capacity(wire::FIRST_COPIES);
    for address in addresses {
        copies.push((*address).into()).unwrap();
    }
    parts.starts_with = w::StartsWith::Copies(
        w::CopyAddresses::new(&w::CEILINGS, w::CopyAddressesParts { addresses: copies }).unwrap(),
    );
    parts.repositories.clear();
    if let Some(project) = project {
        parts
            .repositories
            .push(
                w::Adoption::new(
                    &w::CEILINGS,
                    w::AdoptionParts { project, owner: b"work".as_slice().into(), name: b"existing".as_slice().into() },
                )
                .unwrap(),
            )
            .unwrap();
    }
    w::ForgeRow::new(&w::CEILINGS, parts).unwrap()
}
fn ready_copies(addresses: &[&[u8]], home: Option<u32>) -> (Domain, skein_lib::Token) {
    let (mut root, me, watch) = super::welcome::ready_default_with_forges();
    let mut forges = List::with_capacity(1);
    forges.push(copy_row(COPY_FORGE, w::Mode::Owned, addresses, home)).unwrap();
    super::welcome::up(
        &mut root,
        jig_client::Up::FamilyDelivered {
            watch,
            record: jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::forges(&w::Forges::new(&w::CEILINGS, w::ForgesParts { forges }).unwrap()),
            },
        },
    );
    (root, me)
}
type Expected<'a> = (&'a [u8], &'a [u8], &'a [u8], &'a [u8]);
fn assert_copies(
    sent: &List<(skein_lib::RequestKey, jig_client::Ask)>,
    project: u32,
    home: bool,
    expected: &[Expected<'_>],
) {
    assert_eq!(sent.len(), u32::try_from(expected.len()).unwrap(), "all copies sent without waiting for answers");
    for ((key, ask), (key_name, org, name, address)) in sent.iter().zip(expected) {
        assert_eq!(*key, crate::key::named(crate::key::TOOLS, &crate::key::Name::new(key_name).unwrap()));
        let jig_client::Ask::Family { scope, record } = ask else { panic!("copy is a family request") };
        assert_eq!(*scope, c::Scope::Project(project));
        let request = wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap();
        let w::Request::MakeRepository(request) = request.request() else { panic!("only MakeRepository, no adoption") };
        assert_eq!(request.forge(), COPY_FORGE);
        assert_eq!(request.org(), *org);
        assert_eq!(request.name(), *name);
        let w::MakeFrom::Copy(copy) = request.from() else { panic!("a public copy") };
        assert_eq!(copy.address(), *address);
        assert_eq!(request.into().as_ref().unwrap().project(), project);
        assert_eq!(request.into().as_ref().unwrap().home(), home && *key_name == b"welcome-copy-1");
    }
}
const EXPECTED: &[Expected<'_>] = &[
    (b"welcome-copy-1", b"acme", b"widget", b"https://github.com/acme/widget"),
    (b"welcome-copy-2", b"team", b"tool", b"https://source.invalid/team/tool"),
    (b"welcome-copy-3", b"other", b"library", b"https://github.com/other/library"),
];
#[test]
fn ready_makes_project_then_sends_every_copy_at_once() {
    for (addresses, expected) in [(&ADDRESSES[..1], &EXPECTED[..1]), (ADDRESSES, EXPECTED)] {
        let (mut root, _) = ready_copies(addresses, None);
        let sent = sends(press_default(&mut root));
        let [(_, jig_client::Ask::Jig(ask))] = sent.as_slice() else { panic!("MakeProject alone first") };
        assert_eq!(
            ask.as_ref(),
            &c::Ask::MakeProject { name: c::Text::new(b"Widget".as_slice().into(), 64).unwrap(), policy: 1 }
        );
        let serial = root.pages.serial;
        let sent = sends(project_answer(&mut root, made()));
        assert_copies(&sent, PROJECT, true, expected);
        assert_eq!(root.pages.serial, serial + u64::try_from(addresses.len()).unwrap());
        assert_eq!(root.pages().address(), &Address::Home { project: PROJECT });
        assert!(root.pages().welcome_first().is_none());
        assert!(root.pages().adopting().is_none());
        let serial = root.pages.serial;
        assert!(again(&mut root).is_empty());
        assert_eq!(root.pages.serial, serial, "an adoption retry asks no copies");
    }
}
#[test]
fn taken_name_copies_into_existing_project_respecting_its_home() {
    for has_home in [false, true] {
        let held = if has_home { Some(f::PROJECT) } else { None };
        let (mut root, me) = ready_copies(ADDRESSES, held);
        let mut frame = pages::accounts::current_me(root.client()).unwrap().clone();
        frame.projects.get_mut(0).unwrap().name =
            c::DocumentText::complete(c::Text::new(b"widget".as_slice().into(), 64).unwrap());
        deliver(&mut root, me, c::Delivery::Me(Box::new(frame)));
        press_default(&mut root);
        let sent = sends(project_answer(
            &mut root,
            jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
                refusal: c::Refusal::NameTaken,
                at: None,
            })),
        ));
        assert_copies(&sent, f::PROJECT, !has_home, EXPECTED);
        assert_eq!(root.pages().address(), &Address::Home { project: f::PROJECT });
        assert!(root.pages().adopting().is_none());
    }
}
#[test]
fn unreadable_kept_address_is_omitted_preserving_other_places() {
    let (mut root, _) = ready_copies(&[b"not an address", ADDRESSES[1]], None);
    press_default(&mut root);
    let sent = sends(project_answer(&mut root, made()));
    assert_copies(&sent, PROJECT, true, &EXPECTED[1..2]);
}
#[test]
fn copies_choose_lowest_real_forge_and_offer_its_first_name() {
    let mut rows = List::with_capacity(4);
    for row in [
        copy_row(1, w::Mode::Fake, &[ADDRESSES[1]], None),
        copy_row(2, w::Mode::Owned, &[ADDRESSES[1]], None),
        copy_row(COPY_FORGE, w::Mode::Owned, ADDRESSES, None),
        copy_row(9, w::Mode::Owned, &[ADDRESSES[1]], None),
    ] {
        rows.push(row).unwrap();
    }
    let record = jig_client::Family {
        family: wire::FORGE,
        version: wire::CLIENT_VERSION,
        bytes: wire::forges(&w::Forges::new(&w::CEILINGS, w::ForgesParts { forges: rows }).unwrap()),
    };
    let mut rows = forge::forges::rows(&record, &f::limits().forge).unwrap();
    rows.reverse();
    let rows = forge::forges::State::Rows(rows);
    let copies = forge::making::copies(&rows, Some(2)).unwrap();
    assert_eq!(copies.forge, COPY_FORGE);
    assert_eq!(copies.addresses.len(), ADDRESSES.len());
    for (address, expected) in copies.addresses.iter().zip(ADDRESSES) {
        assert_eq!(address.as_ref(), *expected);
    }
    assert_eq!(pages::welcome::offered(None, Some(&copies)).as_ref(), b"Widget");
    let first =
        pages::welcome::First { forge: COPY_FORGE, org: b"work".as_slice().into(), name: b"hello".as_slice().into() };
    assert_eq!(pages::welcome::offered(Some(&first), Some(&copies)).as_ref(), b"Hello");
}

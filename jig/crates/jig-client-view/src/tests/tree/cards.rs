#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary tests collect text, query trees and select scripted requests"
)]

use crate::tests::fixtures::{Fixture, Watched};
use crate::tests::kit::Goldens;
pub(crate) use crate::tests::kit::words as shared_words;
pub(crate) use crate::tests::kit::{contains, limits, text, words};
use crate::words::phrases::english;
use crate::words::phrases::jig::{JigPhrase, shapes};
use crate::{
    Event, Request,
    state::{action::Act as Action, address::Address},
};
use crate::{JigBindings, JigMeaning, Owner, decode};
use jig_client::contract::*;
use skein_lib::{Duration, List, Time, Token, Wall};
use skein_ui::Act;
use skein_ui::phrases::{Hole, PhraseLimits, Phrasebook};
use skein_ui::{Builder, Role, SlotRef, Tree};

pub(crate) const CLIENT: crate::tests::fixtures::Profile = crate::tests::fixtures::Profile {
    view: crate::Config {
        notices: 2,
        notice_life: Duration::from_secs(5),
        linger: Duration::from_secs(5),
        save_delay: Duration::from_secs(1),
        intents: 2,
        objects: 3,
        sources: 8,
        drafts: 1,
        roles: 2,
        conversation_words: 2,
        navigation_entries: 8,
        updates_count: 99,
    },
    client: jig_client::DomainLimits {
        watches: 9,
        fields: 8,
        reads: 8,
        parts: 8,
        delegates: 2,
        document_bytes: 65536,
        records: 2,
        kept: 4,
        projects: 2,
        text: 128,
        rows: 2,
        segments: 4,
        accounts: jig_client::AccountCounts { accounts: 8, sources: 8, sign_ins: 8 },
        ..crate::tests::fixtures::CLIENT
    },
};

pub(in crate::tests) fn drive(d: &mut Fixture, event: impl crate::tests::fixtures::Input) -> Vec<Request> {
    d.drive(Time::ZERO, event).into_vec()
}
pub(in crate::tests) fn document() -> Me {
    let projects = crate::tests::kit::list_in(
        2,
        [MeProject {
            models: List::with_capacity(0),
            templates: crate::tests::fixtures::chat_templates(),
            creating: Some(crate::tests::fixtures::creating()),
            project: 4,
            name: DocumentText::complete(text(b"One")),
            role: HeldRole::Policy { role: 1 },
            pool: Some(PoolLedger {
                source: Funder::Pool { project: 4, person: 1, period: 1 },
                parent: None,
                numbers: OwnerNumbers { budget: 0, spent: 0, spent_below: 0, reserved: 0 },
                left: 0,
                closed: false,
                began: None,
                next: None,
            }),
        }],
    );
    Me {
        providers: List::with_capacity(0),
        providers_cut: false,
        key_bytes: 256,
        redirect_bytes: 4096,
        party: MeParty {
            number: 1,
            kind: PartyKind::Person,
            login: DocumentText::complete(text(b"fixture-login")),
            name: DocumentText::complete(text(b"Pat")),
        },
        projects,
        needs: Needs { questions: 1, proposals: 2, escalations: 3, person_tasks: 4 },
        needs_total: 10,
        results_seen: 0,
        cut: false,
        updates: Count { value: 120, limited: false },

        retention: Duration::from_secs(1000),
        deployment_role: Some(HeldRole::Policy { role: 1 }),
    }
}
pub(in crate::tests) fn start() -> (Fixture, jig_client::Handle) {
    start_arranged(crate::state::page::Arrangement::none())
}
pub(in crate::tests) fn start_arranged(arrangement: crate::state::page::Arrangement) -> (Fixture, jig_client::Handle) {
    let mut d = Fixture::new(&CLIENT);
    d.configure_arrangement(arrangement).unwrap();
    let out = drive(
        &mut d,
        Event::Start {
            seed: 8,
            place: crate::state::address::Place::Jig(Address::Inbox { project: None }),
            records: List::with_capacity(0),
        },
    );
    let token = out
        .iter()
        .find_map(|r| match r {
            Request::Client { asked: jig_client::Event::Watch { handle: watch, .. } } => Some(*watch),
            _ => None,
        })
        .unwrap();
    (d, token)
}

pub(crate) fn tree(
    fixture: &Fixture,
    section: Option<crate::state::page::Section>,
    card: Option<skein_lib::Id<crate::state::objects::Object>>,
) -> Tree {
    let bounds = crate::sizes::limits::of_words(&fixture.ui.limits, fixture.client.domain_limits(), words()).unwrap();
    let mut builder = Builder::new(&bounds);
    let mut bindings = JigBindings::new(bounds.bindings, fixture.client.domain_limits().text);
    builder.open(Role::Page);
    if let Some(card) = card {
        crate::tree::cards::card(
            &mut builder,
            &fixture.view(),
            card,
            &crate::tree::cards::Place::Elsewhere,
            words(),
            &mut bindings,
        );
    } else {
        match section {
            Some(section) => {
                builder.open(Role::Main);
                crate::tree::sections::section(&mut builder, &fixture.view(), words(), &mut bindings, section);
                builder.close();
            }
            None => {
                crate::tree::frame::banner(&mut builder, &fixture.view(), words(), &mut bindings);
                crate::tree::pages::main(&mut builder, &fixture.view(), words(), &mut bindings);
                crate::tree::notices::notices(&mut builder, &fixture.view(), words(), &mut bindings);
            }
        }
    }
    builder.close();
    builder.finish()
}

fn frame_builds_statuses_counts_navigation_and_number_submission_in_every_link_state() {
    let mut goldens = Goldens::new("tree/cards/frame_builds_statuses");
    let (mut d, watch) = start();
    let words = words();
    let mut bindings = JigBindings::new(64, CLIENT.client.text);
    for expected in [b"Connecting".as_slice(), b"Live", b"Offline"] {
        if expected == b"Live" {
            drive(&mut d, Watched { watch, event: WatchEvent::Delivered(Delivery::Me(Box::new(document()))) });
        }
        if expected == b"Offline" {
            d.set_link(jig_client::Link::Offline);
        }
        bindings.begin();
        let mut b = Builder::new(&limits());
        b.open(Role::Page);
        crate::tree::frame::banner(&mut b, &d.view(), words, &mut bindings);
        crate::tree::frame::navigation(
            &mut b,
            &d.view(),
            SlotRef { owner: (Owner::Application { number: 1 }).opaque(), token: Token::new(1) },
            words,
            &mut bindings,
        );
        b.close();
        let tree = b.finish();
        crate::tests::tree::cards_access::check(&tree);
        goldens.check(&tree);
        let field = tree.nodes().iter().find(|n| n.role == Role::TextField).unwrap().binding.unwrap();
        assert_eq!(decode(&mut bindings, field.token, Act::Input { value: b"123".as_slice().into() }), None);
        assert_eq!(decode(&mut bindings, field.token, Act::Submit), Some(Action::Go { address: Address::task(123) }));
        assert_eq!(decode(&mut bindings, field.token, Act::Input { value: b"x".as_slice().into() }), None);
        assert_eq!(decode(&mut bindings, field.token, Act::Submit), None);
    }
}

fn every_binding_kind_decodes_and_old_tokens_are_retired() {
    let mut bindings = JigBindings::new(64, 4);
    let object = Token::new(7);
    let actions = [
        Action::Go { address: Address::task(1) },
        Action::Submit { field: crate::state::fields::Purpose::Answer },
        Action::Intend { object, act: ActKind::Cancel },
        Action::Choose { option: 2 },
        Action::Confirm,
        Action::Dismiss,
        Action::Fold { object, open: true },
        Action::Filter { group: InboxGroup::Questions, project: None },
        Action::More { window: object, end: jig_client::thread::window::End::Before },
    ];
    for action in actions {
        let token = bindings.insert(JigMeaning::Action(action.clone())).token;
        assert_eq!(decode(&mut bindings, token, Act::Press), Some(action));
        assert_eq!(decode(&mut bindings, token, Act::Input { value: Box::new([]) }), None);
    }
    let field = crate::state::fields::Purpose::Answer;
    let token = bindings.insert(JigMeaning::Field(field)).token;
    assert_eq!(
        decode(&mut bindings, token, Act::Input { value: b"yes".as_slice().into() }),
        Some(Action::Edit { field, text: text(b"yes") })
    );
    assert_eq!(decode(&mut bindings, token, Act::Submit), Some(Action::Submit { field }));
    assert_eq!(decode(&mut bindings, token, Act::Input { value: b"large".as_slice().into() }), None);
    bindings.begin();
    assert_eq!(decode(&mut bindings, token, Act::Submit), None);
    assert_ne!(bindings.insert(JigMeaning::Field(field)).token, token);
}

fn jig_phrasebook_requires_every_sentence_and_says_wall_times_in_person_offset() {
    let mut goldens = Goldens::new("tree/cards/jig_phrasebook_requires");
    let mut language = english::language();
    let missing = language.phrases.last().unwrap().id;
    let size = language.phrases.len().checked_sub(1).unwrap();
    language.phrases = language.phrases.into_vec().into_iter().take(size).collect::<Vec<_>>().into_boxed_slice();
    assert_eq!(
        Phrasebook::load(
            shapes(128),
            Box::new([language]),
            &PhraseLimits {
                languages: 1,
                phrases: crate::words::owner_words::phrase_count(),
                holes: 4,
                segments: 1000,
                text_bytes: 10000,
                hole_bytes: 128,
            }
        )
        .unwrap_err(),
        skein_ui::phrases::Invalid::Missing { id: missing }
    );
    let words = words();
    let mut b = Builder::new(&limits());
    b.open(Role::Page);
    words.wall(&mut b, Wall::EPOCH);
    words.waiting(&mut b, Wall::from_nanos(240_000_000_000), Wall::EPOCH);
    words.amount(&mut b, u64::MAX);
    b.close();
    let tree = b.finish();
    crate::tests::tree::cards_access::check(&tree);
    goldens.check(&tree);
    assert_eq!(words.say(JigPhrase::Live, &[]).as_ref(), b"Live");
    assert_eq!(words.say(JigPhrase::Amount, &[Hole::Amount(12)]).as_ref(), b"12");
}

pub(in crate::tests) fn party() -> Party {
    Party { number: 1, kind: PartyKind::Person, name: text(b"Pat") }
}

#[test]
fn jig_card_frames_bindings_and_phrase_contracts() {
    frame_builds_statuses_counts_navigation_and_number_submission_in_every_link_state();
    every_binding_kind_decodes_and_old_tokens_are_retired();
    jig_phrasebook_requires_every_sentence_and_says_wall_times_in_person_offset();
}

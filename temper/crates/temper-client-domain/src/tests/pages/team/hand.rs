//! Hand-made reads pin Team's whole-record hand-off (client/pages/team.md, 1).
mod stock;
use super::*;
use crate::pages::team::hand::{self, Act};
use jig_client::{Answered, Down, Outcome, Readout};
use skein_lib::{Reader, Writer};
use temper_client_protocol_wire::v1 as w;

const REVISION: u32 = 4;
fn party(number: u64) -> w::AreaOwner {
    w::AreaOwner::Party(w::AreaParty::new(&w::CEILINGS, w::AreaPartyParts { number }).unwrap())
}
fn task(number: u64) -> w::AreaOwner {
    w::AreaOwner::Task(w::AreaTask::new(&w::CEILINGS, w::AreaTaskParts { number }).unwrap())
}
fn charter(number: u32) -> w::AreaOwner {
    w::AreaOwner::Charter(w::AreaCharter::new(&w::CEILINGS, w::AreaCharterParts { number }).unwrap())
}
fn owner_role() -> w::AreaOwner {
    w::AreaOwner::Role(w::AreaRole::new(&w::CEILINGS, w::AreaRoleParts { number: 0 }).unwrap())
}
fn holder() -> w::Holder {
    w::Holder::Task(w::HolderTask::new(&w::CEILINGS, w::HolderTaskParts { number: 52 }).unwrap())
}
fn record(rows: &[(&[u8], w::AreaOwner)]) -> w::ProjectAreas {
    let mut areas = List::with_capacity(u32::try_from(rows.len()).unwrap());
    for (name, owner) in rows {
        areas
            .push(
                w::Area::new(
                    &w::CEILINGS,
                    w::AreaParts {
                        name: (*name).into(),
                        words: if *name == b"tests" {
                            b"Tests, fuzzing and the tiers".as_slice().into()
                        } else {
                            b"The core".as_slice().into()
                        },
                        owner: owner.clone(),
                    },
                )
                .unwrap(),
            )
            .unwrap();
    }
    w::ProjectAreas::new(&w::CEILINGS, w::ProjectAreasParts { areas }).unwrap()
}
fn fixture(owner: bool, rows: &[(&[u8], w::AreaOwner)]) -> Reads {
    let mut f = reads_fixture();
    f.env.limits.client.domain.long_text = 1024;
    let mut me = reader();
    me.projects.get_mut(0).unwrap().name = text(b"vsr");
    me.projects.get_mut(0).unwrap().role = if owner { c::HeldRole::Owner } else { c::HeldRole::Member };
    let me_watch = f.me;
    super::super::welcome::deliver(&mut f.root, me_watch, c::Delivery::Me(Box::new(me)));
    feed(&mut f, Feed::Open);
    feed(&mut f, Feed::Project(Box::new(read_people(&[YOU, SAM]))));
    let (handle, watch) = f.roles.unwrap();
    let mut line = super::super::chats::chat_line();
    line.chip.task = 52;
    line.chip.title = text(b"Prover");
    line.project = PROJECT;
    line.label = 3;
    line.ended_at = None;
    delivered(
        &mut f,
        handle,
        watch,
        c::Delivery::Tasks(Box::new(c::Tasks { rows: List::from_boxed(Box::new([line])), more: false })),
    );
    let (_, handle, watch) = f.card.unwrap();
    let mut document = fixtures::chat::task();
    document.chip.task = 52;
    document.requester = c::Requester::Person(YOU);
    delivered(&mut f, handle, watch, c::Delivery::Task(Box::new(document)));
    let mut areas = List::with_capacity(u32::try_from(rows.len()).unwrap());
    for row in record(rows).areas() {
        let held = match row.owner() {
            w::AreaOwner::Task(task) => {
                w::Holder::Task(w::HolderTask::new(&w::CEILINGS, w::HolderTaskParts { number: task.number() }).unwrap())
            }
            w::AreaOwner::Party(party) => w::Holder::Party(
                w::HolderParty::new(&w::CEILINGS, w::HolderPartyParts { number: party.number() }).unwrap(),
            ),
            w::AreaOwner::Role(role) => {
                w::Holder::Role(w::HolderRole::new(&w::CEILINGS, w::HolderRoleParts { number: role.number() }).unwrap())
            }
            w::AreaOwner::Charter(_) => holder(),
        };
        areas
            .push(team::Area {
                name: row.name().into(),
                words: row.words().into(),
                owner: row.owner().clone(),
                holder: Some(held),
            })
            .unwrap();
    }
    areas::send(&mut f, team::Areas { rows: areas, triager: Some(task(52)), holder: holder() });
    let settings = f.settings;
    super::super::welcome::deliver(
        &mut f.root,
        settings,
        c::Delivery::Settings(Box::new(c::SettingsDocument {
            newest: REVISION,
            settings: List::with_capacity(0),
            held: List::with_capacity(0),
            pending: List::with_capacity(0),
        })),
    );
    f
}
fn act(f: &mut Reads, act: Act) -> Queue<crate::Request> {
    let mut out = Queue::uncapped();
    crate::route::event(
        &mut f.root,
        &f.env,
        crate::Event::Act { action: crate::Action::Pages(pages::Act::HandArea(act)) },
        &mut out,
    );
    out
}
fn saves(out: &Queue<crate::Request>) -> Box<[(jig_client::Key, c::Ask)]> {
    let mut sent = List::with_capacity(out.len());
    for r in out {
        if let crate::Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Save { record } } } =
            r
            && let jig_client::Ask::Jig(ask) = &record.request.request
        {
            sent.push((record.request.key, *ask.clone())).unwrap();
        }
    }
    sent.into_boxed()
}
fn reading(out: &Queue<crate::Request>, revision: u32) -> Token {
    assert!(saves(out).is_empty(), "nothing is saved before the whole record is read");
    let mut reads = List::with_capacity(out.len());
    for r in out {
        if let crate::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down { down: Down::Read { read, what: Readable::Jig(c::Read::Setting(what)) } },
        } = r
        {
            assert_eq!(
                *what,
                c::ReadSetting {
                    setting: c::Setting::Application { kind: 14, scope: PROJECT, number: 1 },
                    revision,
                    proposal: None
                }
            );
            reads.push(*read).unwrap();
        }
    }
    assert_eq!(reads.len(), 1);
    *reads.get(0).unwrap()
}
fn deliver_record(f: &mut Reads, read: Token, revision: u32, value: Option<w::ProjectAreas>) -> Queue<crate::Request> {
    let value = match value {
        Some(record) => {
            let mut writer = Writer::new(usize::try_from(record.measure()).unwrap());
            record.encode(&mut writer).unwrap();
            Some(c::SettingValue::Application(writer.finish()))
        }
        None => None,
    };
    super::super::welcome::up(
        &mut f.root,
        jig_client::Up::ReadDone {
            read,
            result: Readout::Jig(c::ReadResult::Value(c::ReadValue::Setting(Box::new(c::SettingDocument {
                overlay: None,
                setting: c::Setting::Application { kind: 14, scope: PROJECT, number: 1 },
                revision,
                value,
                application: None,
                proposed: None,
            })))),
        },
    )
}
fn item(items: &List<c::SettingItem>) -> w::ProjectAreas {
    assert_eq!(items.len(), 1);
    let item = items.get(0).unwrap();
    assert_eq!(item.setting, c::Setting::Application { kind: 14, scope: PROJECT, number: 1 });
    let c::SettingEdit::Set(c::SettingSet { value: c::SettingValue::Application(bytes) }) = &item.edit else {
        panic!("set whole record")
    };
    let mut reader = Reader::new(bytes);
    let record = w::ProjectAreas::decode(&w::CEILINGS, &mut reader).unwrap();
    assert!(reader.is_empty());
    record
}
fn answer(f: &mut Reads, key: jig_client::Key, answer: c::Answer) -> Queue<crate::Request> {
    let mut out = Queue::uncapped();
    crate::route::told(
        &mut f.root,
        &f.env,
        Told::Answered { key, outcome: Outcome::Answered(Answered::Jig(answer)) },
        &mut out,
    );
    out
}
#[test]
fn handing_an_area_reads_then_sends_one_whole_change_or_proposal() {
    for owner in [true, false] {
        let rows = [(b"core".as_slice(), party(3)), (b"tests".as_slice(), owner_role())];
        let mut f = fixture(owner, &rows);
        act(&mut f, Act::Open);
        let open = f.root.pages.hand_area().unwrap();
        assert_eq!(open.area.as_deref(), Some(b"core".as_slice()));
        assert_eq!(open.role, Some(52));
        act(&mut f, Act::Area(Some(b"tests".as_slice().into())));
        let out = act(&mut f, Act::Send);
        let read = reading(&out, REVISION);
        let out = deliver_record(&mut f, read, REVISION, Some(record(&rows)));
        let sent = saves(&out);
        assert_eq!(sent.len(), 1);
        let (key, ask) = &sent[0];
        let items = if owner {
            let c::Ask::ChangeSettings(c::ChangeSettings { against, items, running }) = ask else {
                panic!("owner saves")
            };
            assert_eq!((*against, running), (REVISION, &c::Running::Stays));
            items
        } else {
            let c::Ask::Propose { project, action: c::ProposeAction::Settings(settings), reason } = ask else {
                panic!("member proposes")
            };
            assert_eq!((*project, settings.against, reason.bytes()), (0, REVISION, b"Hand tests to Prover".as_slice()));
            &settings.items
        };
        assert_eq!(item(items), record(&[(b"core", party(3)), (b"tests", task(52))]));
        assert!(f.root.pages.hand_area().unwrap().busy());
        if owner {
            answer(
                &mut f,
                *key,
                c::Answer::Done(c::Outcome::SettingsChanged(c::OutcomeSettingsChanged { revision: 5 })),
            );
            assert_eq!(
                f.root.pages.needs().toast(),
                Some(&needs::Toasted::AreaHanded { title: b"Prover".as_slice().into(), revision: 5 })
            );
        } else {
            answer(
                &mut f,
                *key,
                c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::Proposed { proposal: 9, to: 1 })),
            );
            assert_eq!(f.root.pages.needs().toast(), Some(&needs::Toasted::AreasProposed));
        }
        assert!(f.root.pages.hand_area().is_none());
    }
}
#[test]
fn a_new_area_is_judged_on_press_and_appended_only_when_valid() {
    let mut f = fixture(true, &[(b"core", party(3))]);
    act(&mut f, Act::Open);
    act(&mut f, Act::Area(None));
    assert!(!f.root.pages.hand_area().unwrap().judged);
    let long_name = [b'a'; 33];
    let long_words = [b'a'; 481];
    for (name, words, problem) in [
        (b"".as_slice(), b"".as_slice(), Some(hand::NameProblem::Shape)),
        (b"Core", b"The core", Some(hand::NameProblem::Shape)),
        (b"core", b"The core", Some(hand::NameProblem::Duplicate)),
        (b"tests", b"two\nlines", None),
        (&long_name, b"The tests", Some(hand::NameProblem::Shape)),
        (b"tests", &long_words, None),
        (b"tests", b"a\xc2\x85b", None),
    ] {
        act(&mut f, Act::Name(name.into()));
        act(&mut f, Act::Words(words.into()));
        let out = act(&mut f, Act::Send);
        assert!(saves(&out).is_empty());
        let open = f.root.pages.hand_area().unwrap();
        assert!(open.judged && !open.busy());
        assert_eq!(open.name_problem(), problem);
    }
    act(&mut f, Act::Words(b"Tests, fuzzing and the tiers".as_slice().into()));
    let out = act(&mut f, Act::Send);
    let read = reading(&out, REVISION);
    let out = deliver_record(&mut f, read, REVISION, Some(record(&[(b"core", party(3))])));
    let sent = saves(&out);
    let c::Ask::ChangeSettings(c::ChangeSettings { items, .. }) = &sent[0].1 else { panic!("saves") };
    assert_eq!(item(items), record(&[(b"core", party(3)), (b"tests", task(52))]));
    let mut empty = fixture(true, &[]);
    act(&mut empty, Act::Open);
    act(&mut empty, Act::Name(b"tests".as_slice().into()));
    act(&mut empty, Act::Words(b"Tests, fuzzing and the tiers".as_slice().into()));
    let out = act(&mut empty, Act::Send);
    let read = reading(&out, REVISION);
    let out = deliver_record(&mut empty, read, REVISION, None);
    let sent = saves(&out);
    let c::Ask::ChangeSettings(c::ChangeSettings { items, .. }) = &sent[0].1 else { panic!("saves empty default") };
    assert_eq!(item(items), record(&[(b"tests", task(52))]));
}
#[test]
fn stale_handoffs_reread_once_and_other_refusals_keep_the_sheet() {
    for owner in [true, false] {
        let rows = [(b"tests".as_slice(), party(3))];
        let mut f = fixture(owner, &rows);
        act(&mut f, Act::Open);
        let out = act(&mut f, Act::Send);
        let read = reading(&out, REVISION);
        let out = deliver_record(&mut f, read, REVISION, Some(record(&rows)));
        let key = saves(&out)[0].0;
        let stale = c::Answer::Refused { refusal: c::Refusal::Stale(c::RefusalStale { newest: 8 }), at: None };
        let out = answer(&mut f, key, stale.clone());
        let read = reading(&out, 8);
        let out = deliver_record(&mut f, read, 8, Some(record(&[(b"core", party(7)), (b"tests", party(3))])));
        let sent = saves(&out);
        let (against, items) = if owner {
            let c::Ask::ChangeSettings(c::ChangeSettings { against, items, .. }) = &sent[0].1 else {
                panic!("retried change")
            };
            (*against, items)
        } else {
            let c::Ask::Propose { action: c::ProposeAction::Settings(settings), .. } = &sent[0].1 else {
                panic!("retried proposal")
            };
            (settings.against, &settings.items)
        };
        assert_eq!(against, 8);
        assert_eq!(item(items), record(&[(b"core", party(7)), (b"tests", task(52))]));
        let out = answer(&mut f, sent[0].0, stale);
        assert!(saves(&out).is_empty());
        assert!(!f.root.pages.hand_area().unwrap().busy());
        assert!(f.root.pages.hand_area().unwrap().failure.is_some());
        let out = act(&mut f, Act::Send);
        let read = reading(&out, REVISION);
        let out = deliver_record(&mut f, read, REVISION, Some(record(&rows)));
        answer(&mut f, saves(&out)[0].0, c::Answer::Refused { refusal: c::Refusal::NotOffered, at: None });
        assert!(f.root.pages.hand_area().unwrap().failure.is_some());
        act(&mut f, Act::Close);
        assert!(f.root.pages.hand_area().is_none());
    }
}
#[test]
fn ask_temper_gets_the_area_words_unsent_even_for_an_unnamed_new_area() {
    for (area, expected) in [
        (
            Some(b"tests".as_slice().into()),
            b"Write a standing role to own tests on vsr: Tests, fuzzing and the tiers".as_slice(),
        ),
        (None, b"Write a standing role to own a new area on vsr.".as_slice()),
    ] {
        let mut f = fixture(true, &[(b"tests", party(3))]);
        act(&mut f, Act::Open);
        act(&mut f, Act::Area(area));
        act(&mut f, Act::Role(None));
        let out = act(&mut f, Act::Send);
        assert!(f.root.pages.hand_area().is_none());
        assert!(saves(&out).is_empty());
        page(&mut f, Event::Went { address: Address::Ask { project: PROJECT } });
        assert_eq!(f.root.pages.chats().unwrap().words.as_ref(), expected);
        let mut opens_ask = false;
        for r in &out {
            opens_ask |= *r == crate::Request::Show { address: Address::Ask { project: PROJECT }, replace: false };
        }
        assert!(opens_ask);
    }
}
#[test]
fn give_back_changes_task_and_standing_charter_owners_but_not_the_triager() {
    for owner in [true, false] {
        let rows = [(b"tests".as_slice(), task(52)), (b"core".as_slice(), charter(15))];
        let mut f = fixture(owner, &rows);
        act(&mut f, Act::GiveBack);
        assert_eq!(f.root.pages.hand_area().unwrap().give.as_ref().unwrap().names.len(), 2);
        let out = act(&mut f, Act::Send);
        let read = reading(&out, REVISION);
        let out = deliver_record(&mut f, read, REVISION, Some(record(&rows)));
        let sent = saves(&out);
        let items = if owner {
            let c::Ask::ChangeSettings(c::ChangeSettings { items, .. }) = &sent[0].1 else { panic!("owner saves") };
            items
        } else {
            let c::Ask::Propose { action: c::ProposeAction::Settings(settings), .. } = &sent[0].1 else {
                panic!("member proposes")
            };
            &settings.items
        };
        assert_eq!(item(items), record(&[(b"tests", party(YOU)), (b"core", party(YOU))]));
        assert_eq!(team::shown(&f.root, PROJECT).facts().unwrap().areas.as_ref().unwrap().triager, Some(task(52)));
        let done = if owner {
            c::Outcome::SettingsChanged(c::OutcomeSettingsChanged { revision: 5 })
        } else {
            c::Outcome::Document(c::DocumentOutcome::Proposed { proposal: 9, to: 1 })
        };
        answer(&mut f, sent[0].0, c::Answer::Done(done));
        assert!(f.root.pages.hand_area().is_none());
        assert_eq!(
            f.root.pages.needs().toast(),
            Some(&if owner { needs::Toasted::AreasGivenBack { revision: 5 } } else { needs::Toasted::AreasProposed })
        );
    }
}
#[test]
fn controls_require_membership_and_give_back_counts_only_named_held_areas() {
    let mut f = fixture(true, &[(b"tests", task(52)), (b"core", charter(15))]);
    let mut facts = team::shown(&f.root, PROJECT).facts().unwrap().clone();
    assert!(team::parts(&facts).hand_area);
    assert_eq!(team::parts(&facts).give_areas, 2);
    facts.areas.as_mut().unwrap().rows.get_mut(0).unwrap().holder = None;
    assert_eq!(team::parts(&facts).give_areas, 1);
    facts.areas.as_mut().unwrap().rows.get_mut(1).unwrap().holder = None;
    assert_eq!(team::parts(&facts).give_areas, 0, "triager-only holder cannot be given back");
    facts.member = false;
    assert!(!team::parts(&facts).hand_area);
    assert_eq!(team::parts(&facts).give_areas, 0);
    facts.member = true;
    facts.areas = None;
    assert!(!team::parts(&facts).hand_area);
    act(&mut f, Act::Open);
    act(&mut f, Act::Close);
    assert!(f.root.pages.hand_area().is_none());
}

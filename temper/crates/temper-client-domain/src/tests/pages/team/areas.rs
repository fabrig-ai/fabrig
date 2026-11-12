//! Team's ownership read and parts (client/pages/team.md, 1).
use super::*;
use temper_client_protocol_wire::{self as wire, v1 as w};

fn task(number: u64) -> w::Holder {
    w::Holder::Task(w::HolderTask::new(&w::CEILINGS, w::HolderTaskParts { number }).unwrap())
}
fn owner(number: u64) -> w::AreaOwner {
    w::AreaOwner::Task(w::AreaTask::new(&w::CEILINGS, w::AreaTaskParts { number }).unwrap())
}
fn owners() -> w::Holder {
    w::Holder::Role(w::HolderRole::new(&w::CEILINGS, w::HolderRoleParts { number: 0 }).unwrap())
}
fn areas() -> team::Areas {
    team::Areas {
        rows: List::from_boxed(Box::new([team::Area {
            name: b"tests".as_slice().into(),
            words: b"Tests and fuzzing".as_slice().into(),
            owner: owner(52),
            holder: Some(task(52)),
        }])),
        triager: Some(owner(52)),
        holder: task(52),
    }
}
fn facts() -> Facts {
    let mut facts = team::gather(&read_people(&[YOU]), YOU, None, &[], None, Wall::EPOCH);
    facts.roles = List::from_boxed(Box::new([role(52)]));
    facts.selected = Some(52);
    facts.card = Some(Box::new(card(52)));
    facts
}
#[test]
fn areas_parts_follow_holders_and_offered_talk() {
    let mut facts = facts();
    let detail = facts.card.as_mut().unwrap();
    detail.talk = true;
    facts.areas = Some(areas());
    let parts = team::parts(&facts);
    assert!(parts.areas && parts.areas_table && parts.who_routes && parts.lede_areas);
    assert!(!parts.areas_empty && !parts.standing_roles);
    assert!(parts.standing.is_empty());
    assert_eq!(parts.area_rows.len(), 2);
    assert_eq!(
        parts.owns.as_slice(),
        &[team::AreaName::Named(b"tests".as_slice().into()), team::AreaName::AnythingElse]
    );
    let first = parts.area_rows.get(0).unwrap();
    let last = parts.area_rows.get(1).unwrap();
    assert_eq!(first.role, Some(52));
    assert!(first.first && first.current);
    assert_eq!(last.role, Some(52));
    assert!(!last.first && !last.current);
    assert!(parts.talk);
    facts.card.as_mut().unwrap().talk = false;
    assert!(!team::parts(&facts).talk);
    let person = w::Holder::Party(w::HolderParty::new(&w::CEILINGS, w::HolderPartyParts { number: YOU }).unwrap());
    let area = facts.areas.as_mut().unwrap();
    area.rows.get_mut(0).unwrap().holder = Some(person.clone());
    area.holder = person;
    let parts = team::parts(&facts);
    assert!(parts.standing_roles);
    assert_eq!(parts.standing.as_slice(), &[52]);
    assert!(parts.owns.is_empty());
    assert_eq!(parts.box_areas.get(0).unwrap().0, YOU);
    assert_eq!(parts.box_areas.get(0).unwrap().1.len(), 2);
    facts.areas.as_mut().unwrap().rows.get_mut(0).unwrap().holder = None;
    assert_eq!(team::parts(&facts).box_areas.get(0).unwrap().1.len(), 1);
    facts.roles_cut = true;
    facts.roles = List::with_capacity(0);
    assert!(team::parts(&facts).standing_roles && team::parts(&facts).roles_cut);
    facts.roles = List::from_boxed(Box::new([role(52)]));
    facts.roles_cut = false;
    facts.areas = Some(team::Areas { rows: List::with_capacity(0), triager: None, holder: owners() });
    let parts = team::parts(&facts);
    assert!(parts.areas && parts.areas_empty && parts.standing_roles);
    assert!(!parts.areas_table && !parts.who_routes && !parts.lede_areas);
    assert!(parts.area_rows.is_empty() && parts.owns.is_empty() && parts.box_areas.is_empty());
    facts.areas = None;
    let parts = team::parts(&facts);
    assert!(!parts.areas && !parts.areas_empty);
    assert_eq!(parts.standing.as_slice(), &[52]);
}
fn send(f: &mut Reads, areas: team::Areas) {
    let (handle, watch, subject) = f.findings.last().unwrap().clone();
    let mut rows = List::with_capacity(areas.rows.len());
    for area in areas.rows.into_boxed() {
        rows.push(
            w::AreaRow::new(
                &w::CEILINGS,
                w::AreaRowParts { name: area.name, words: area.words, owner: area.owner, holder: area.holder },
            )
            .unwrap(),
        )
        .unwrap();
    }
    let document = w::Findings::new(
        &w::CEILINGS,
        w::FindingsParts {
            project: subject.project(),
            rows: List::with_capacity(0),
            areas: rows,
            triager: areas.triager,
            holder: areas.holder,
            cut: false,
        },
    )
    .unwrap();
    client(
        f,
        jig_client::Event::Up {
            up: jig_client::Up::FamilyDelivered {
                watch,
                record: jig_client::Family {
                    family: wire::FORGE,
                    version: wire::CLIENT_VERSION,
                    bytes: wire::findings(&document),
                },
            },
        },
    );
    page(f, Event::Told { told: Told::Changed { handle } });
}
#[test]
fn team_watches_areas_redraws_and_closes_without_retrying_refusals() {
    let mut f = reads_fixture();
    feed(&mut f, Feed::Open);
    feed(&mut f, Feed::Project(Box::new(read_people(&[YOU]))));
    assert_eq!(f.findings.len(), 1);
    feed(&mut f, Feed::Open);
    assert_eq!(f.findings.len(), 1, "one watch while Team stays open");
    let (handle, roles) = f.roles.unwrap();
    let mut line = super::super::chats::chat_line();
    line.chip.task = 52;
    line.project = PROJECT;
    line.label = 3;
    line.ended_at = None;
    delivered(
        &mut f,
        handle,
        roles,
        c::Delivery::Tasks(Box::new(c::Tasks { rows: List::from_boxed(Box::new([line])), more: false })),
    );
    let (_, card_handle, card_watch) = f.card.unwrap();
    let mut document = fixtures::chat::task();
    document.chip.task = 52;
    document.acts = List::from_boxed(Box::new([c::Offered {
        act: c::ActKind::Talk,
        admission: c::Admission::Allowed,
        effect: None,
    }]));
    delivered(&mut f, card_handle, card_watch, c::Delivery::Task(Box::new(document.clone())));
    assert!(team::parts(team::shown(&f.root, PROJECT).facts().unwrap()).talk);
    document.acts = List::with_capacity(0);
    delivered(&mut f, card_handle, card_watch, c::Delivery::Task(Box::new(document)));
    assert!(!team::parts(team::shown(&f.root, PROJECT).facts().unwrap()).talk);
    assert!(team::shown(&f.root, PROJECT).facts().unwrap().areas.is_none());
    let mut document = areas();
    let mut rows = List::with_capacity(2);
    rows.push(document.rows.get(0).unwrap().clone()).unwrap();
    rows.push(team::Area {
        name: b"core".as_slice().into(),
        words: b"The protocol".as_slice().into(),
        owner: w::AreaOwner::Party(w::AreaParty::new(&w::CEILINGS, w::AreaPartyParts { number: YOU }).unwrap()),
        holder: Some(w::Holder::Party(w::HolderParty::new(&w::CEILINGS, w::HolderPartyParts { number: YOU }).unwrap())),
    })
    .unwrap();
    document.rows = rows;
    // Kind 14's second delivery changes the person's area to the role, not the watch.
    send(&mut f, document.clone());
    assert_eq!(team::shown(&f.root, PROJECT).facts().unwrap().areas, Some(document.clone()));
    let parts = team::parts(team::shown(&f.root, PROJECT).facts().unwrap());
    assert_eq!(parts.area_rows.len(), 3);
    assert_eq!(parts.box_areas.get(0).unwrap().1.as_slice(), &[team::AreaName::Named(b"core".as_slice().into())]);
    document.rows.get_mut(1).unwrap().owner = owner(52);
    document.rows.get_mut(1).unwrap().holder = Some(task(52));
    send(&mut f, document.clone());
    assert_eq!(team::shown(&f.root, PROJECT).facts().unwrap().areas, Some(document));
    let parts = team::parts(team::shown(&f.root, PROJECT).facts().unwrap());
    assert_eq!(parts.owns.len(), 3);
    assert!(parts.box_areas.is_empty());
    assert_eq!(parts.area_rows.get(1).unwrap().role, Some(52));
    let (handle, watch, _) = f.findings.last().unwrap().clone();
    client(&mut f, jig_client::Event::Up { up: jig_client::Up::WatchEnded { watch, end: c::WatchEnd::NotAllowed } });
    page(&mut f, Event::Told { told: Told::Ended { handle, why: jig_client::Why::Unknown } });
    let facts = team::shown(&f.root, PROJECT).facts().unwrap().clone();
    assert!(facts.areas.is_none());
    assert_eq!(team::parts(&facts).standing.as_slice(), &[52]);
    feed(&mut f, Feed::Open);
    assert_eq!(f.findings.len(), 1, "no automatic retry");
    page(&mut f, Event::Went { address: Address::Home { project: PROJECT } });
    assert!(f.closed.as_slice().contains(&handle), "leaving Team closes its watch even in the same project");
    feed(&mut f, Feed::Open);
    let second = f.findings.last().unwrap().0;
    assert_eq!(f.findings.len(), 2);
    page(&mut f, Event::Went { address: Address::Team { project: OTHER_PROJECT, role: None } });
    assert!(f.closed.as_slice().contains(&second), "switching project closes the old watch");
    assert_eq!(f.findings.len(), 3);
    assert_eq!(f.findings.last().unwrap().2.project(), OTHER_PROJECT);
    assert_eq!(team::shown(&f.root, OTHER_PROJECT), team::Shown::Reading);
}

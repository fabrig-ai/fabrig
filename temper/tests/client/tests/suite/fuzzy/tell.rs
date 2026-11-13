//! Tell's three routing steps (client/pages/needs-you.md, 1, Where it goes).
use skein_lib::{List, Rng};
use std::panic::catch_unwind;
use temper_client_domain::{
    Address,
    address::{ChatsFilter, Filter, GoalsFilter, Step, WorkFilter},
    pages::{
        team::Holder,
        tell::{self, About, Goes},
    },
};
use temper_client_protocol_wire::v1 as w;

const SEEDS: u64 = 256;
const SAMPLES: u32 = 32;

fn holder(value: &Holder) -> w::Holder {
    match *value {
        Holder::Task(number) => {
            w::Holder::Task(w::HolderTask::new(&w::CEILINGS, w::HolderTaskParts { number }).expect("small task"))
        }
        Holder::Party(number) => {
            w::Holder::Party(w::HolderParty::new(&w::CEILINGS, w::HolderPartyParts { number }).expect("small party"))
        }
        Holder::Role(number) => {
            w::Holder::Role(w::HolderRole::new(&w::CEILINGS, w::HolderRoleParts { number }).expect("small role"))
        }
    }
}
fn owner(kind: u64, number: u32) -> w::AreaOwner {
    match kind {
        0 => w::AreaOwner::Task(
            w::AreaTask::new(&w::CEILINGS, w::AreaTaskParts { number: u64::from(number) }).expect("task"),
        ),
        1 => w::AreaOwner::Party(
            w::AreaParty::new(&w::CEILINGS, w::AreaPartyParts { number: u64::from(number) }).expect("party"),
        ),
        2 => w::AreaOwner::Role(w::AreaRole::new(&w::CEILINGS, w::AreaRoleParts { number }).expect("role")),
        3 => w::AreaOwner::Charter(w::AreaCharter::new(&w::CEILINGS, w::AreaCharterParts { number }).expect("charter")),
        _ => unreachable!("four owner kinds"),
    }
}

// The model keeps generated facts, not a second reading of the wire document.
// Only a task holder can match a page, and a configured but absent triager
// is distinct from the document's owners fallback.
struct Model {
    areas: Vec<(Box<[u8]>, Option<Holder>)>,
    triager: Option<Holder>,
}
impl Model {
    fn route(&self, task: Option<u64>) -> Goes {
        let first = self.areas.iter().find(|(_, holder)| task.is_some_and(|task| *holder == Some(Holder::Task(task))));
        if let Some((name, Some(holder))) = first {
            return Goes::Area { name: name.clone(), holder: holder.clone() };
        }
        let areas = !self.areas.is_empty();
        self.triager.as_ref().map_or(Goes::Owners { areas }, |holder| Goes::Triager { holder: holder.clone(), areas })
    }
}
fn document(rng: &mut Rng) -> (w::Findings, Model) {
    let mut areas = List::with_capacity(16);
    let mut model = Model { areas: Vec::new(), triager: None };
    for index in 0..rng.between(0, 16) {
        let number = u32::try_from(rng.between(1, 3)).expect("small holder number");
        let held = match rng.below(4) {
            0 => Some(Holder::Task(u64::from(number))),
            1 => Some(Holder::Party(u64::from(number))),
            2 => Some(Holder::Role(number)),
            _ => None,
        };
        let name: Box<[u8]> = format!("area-{index}").into_bytes().into();
        areas
            .push(
                w::AreaRow::new(
                    &w::CEILINGS,
                    w::AreaRowParts {
                        name: name.clone(),
                        words: b"Work in this area".as_slice().into(),
                        owner: owner(rng.below(4), number),
                        holder: held.as_ref().map(holder),
                    },
                )
                .expect("bounded area"),
            )
            .expect("up to sixteen areas");
        model.areas.push((name, held));
    }
    let number = u32::try_from(rng.between(1, 3)).expect("small owner number");
    let kind = rng.below(5);
    let triager = (kind < 4).then(|| owner(kind, number));
    let standing = kind < 4 && rng.chance(500);
    let held = if standing {
        match kind {
            0 => Holder::Task(u64::from(number)),
            1 => Holder::Party(u64::from(number)),
            2 => Holder::Role(number),
            3 => Holder::Task(rng.between(1, 3)),
            _ => unreachable!("four configured kinds"),
        }
    } else {
        Holder::Role(0)
    };
    if standing {
        model.triager = Some(held.clone());
    }
    let document = w::Findings::new(
        &w::CEILINGS,
        w::FindingsParts {
            project: 1,
            rows: List::with_capacity(0),
            areas,
            triager,
            holder: holder(&held),
            cut: rng.chance(500),
        },
    )
    .expect("small findings document");
    (document, model)
}

// The task is an input to the generated address and the model, rather than
// extracted by the production address match. Other pages can carry the same
// number as a project or charter without naming a task.
fn about(rng: &mut Rng) -> (Option<About>, Option<u64>) {
    let task = rng.between(1, 4);
    let number = u32::try_from(task).expect("small project or charter");
    let (address, named) = match rng.below(32) {
        0 => return (None, None),
        1 => (Address::Task { task }, Some(task)),
        2 => (Address::Talk { project: 1, task }, Some(task)),
        3 => (Address::Chat { task }, Some(task)),
        4 => (Address::Team { project: 1, role: Some(task) }, Some(task)),
        5 => (Address::Team { project: 1, role: None }, None),
        6 => (Address::Inbox { filter: Filter::ALL }, None),
        7 => (Address::Chats { filter: ChatsFilter::All }, None),
        8 => (Address::NewChat { draft: rng.chance(500) }, None),
        9 => (Address::Accounts, None),
        10 => (Address::Forges, None),
        11 => (Address::ConnectForge { forge: None }, None),
        12 => (Address::ConnectModel { way: None }, None),
        13 => (Address::Welcome { step: Step::Welcome }, None),
        14 => (Address::Home { project: number }, None),
        15 => (Address::AllHome, None),
        16 => (Address::Goals { project: number, filter: GoalsFilter::All }, None),
        17 => (Address::Changes { project: number }, None),
        18 => (Address::Settings { project: number }, None),
        19 => (Address::Repositories { project: number }, None),
        20 => (Address::Models { project: number }, None),
        21 => (Address::Builders { project: number }, None),
        22 => (Address::Role { project: 1, charter: number }, None),
        23 => (Address::Rule { project: 1, kind: 14, number }, None),
        24 => (Address::Ask { project: number }, None),
        25 => (Address::Adopt { project: number }, None),
        26 => (Address::Work { project: number, filter: WorkFilter::All }, None),
        27 => (Address::Members, None),
        28 => (Address::Remembered, None),
        29 => (Address::Told, None),
        30 => (Address::Limits, None),
        // An Ask temper chat has no special routing rule.
        _ => (Address::Chat { task: 99 }, Some(99)),
    };
    (Some(About { address, title: rng.chance(500).then(|| b"Prover".as_slice().into()) }), named)
}
fn sweep(seed: u64) -> Vec<Goes> {
    let mut rng = Rng::new(seed);
    let mut results = Vec::new();
    for _ in 0..SAMPLES {
        let (document, model) = document(&mut rng);
        let (about, task) = about(&mut rng);
        let actual = tell::route(&document, about.as_ref());
        assert_eq!(actual, model.route(task), "seed {seed}: {about:?}, {document:?}");
        if let Goes::Area { name, holder } = &actual {
            let task = task.expect("only task addresses route to an area");
            let first = document
                .areas()
                .iter()
                .find(|area| matches!(area.holder(), Some(w::Holder::Task(held)) if held.number() == task))
                .expect("a destination area belongs to the page's task");
            assert_eq!(name.as_ref(), first.name(), "first in document order");
            assert_eq!(holder, &Holder::Task(task));
        }
        results.push(actual);
    }
    results
}
#[test]
fn tell_route_follows_the_three_steps_and_replays() {
    for seed in 1..=SEEDS {
        match catch_unwind(|| {
            let first = sweep(seed);
            assert_eq!(first, sweep(seed), "seed {seed} replays");
        }) {
            Ok(()) => (),
            Err(error) => panic!("seed {seed}: Tell routing sweep panicked: {error:?}"),
        }
    }
}

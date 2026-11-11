//! Team's parts and moves (temper/docs/design/client/pages/README.md, Team;
//! temper/docs/design/draft/surfaces.md, 12 and 14).
use jig_client::contract as c;
use jig_client_kit::stories;
use skein_lib::{List, Rng};
use std::panic::catch_unwind;
use temper_client_domain::pages::team::{self, Person, Waits};

// Small documents with repeated parties and readers who need not belong to
// the project, each seed independently replayable.
const SEEDS: u64 = 256;
const MAX_MEMBERS: u32 = 12;
const MEMBER_PARTIES: u64 = 6;
const READER_PARTIES: u64 = 7;
const MAX_OTHERS: u64 = 8;

fn random_count(rng: &mut Rng) -> u32 {
    u32::try_from(rng.between(0, u64::from(u32::MAX))).expect("a u32 count")
}

fn random_role(rng: &mut Rng) -> c::HeldRole {
    match rng.below(5) {
        0 => c::HeldRole::Owner,
        1 => c::HeldRole::Maintainer,
        2 => c::HeldRole::Member,
        3 => c::HeldRole::Observer,
        _ => c::HeldRole::Policy { role: random_count(rng) },
    }
}

fn random_waits(rng: &mut Rng) -> Waits {
    Waits {
        questions: random_count(rng),
        reviews: random_count(rng),
        proposals: random_count(rng),
        held: random_count(rng),
        person_tasks: random_count(rng),
        cut: rng.chance(500),
    }
}

fn random_project(rng: &mut Rng) -> c::Project {
    let mut project = stories::project();
    project.members = List::with_capacity(MAX_MEMBERS);
    let shared = if rng.chance(500) { Some(random_role(rng)) } else { None };
    for member in 0..rng.between(0, u64::from(MAX_MEMBERS)) {
        let party = rng.between(1, MEMBER_PARTIES);
        let kind = if rng.chance(500) { c::PartyKind::Person } else { c::PartyKind::Service };
        let role = shared.unwrap_or_else(|| random_role(rng));
        let name = if rng.chance(500) { String::new() } else { format!("name-{member}") };
        let login = format!("login-{member}");
        project
            .members
            .push(c::ProjectMember {
                party,
                kind,
                role,
                name: c::DocumentText::complete(stories::text(name.as_bytes())),
                login: c::DocumentText::complete(stories::text(login.as_bytes())),
            })
            .expect("random members fit");
    }
    project.cut = rng.chance(500);
    project
}

// Read the document once, keeping the first person of each party, then move
// the reader to the front independently of gather's ordering algorithm.
fn oracle(project: &c::Project, you: u64, yours: Option<Waits>, others: &[(u64, Waits)]) -> Vec<Person> {
    let mut people: Vec<Person> = Vec::new();
    for member in &project.members {
        if member.kind != c::PartyKind::Person || people.iter().any(|person| person.party == member.party) {
            continue;
        }
        let name = if member.name.bytes().is_empty() { member.login.bytes() } else { member.name.bytes() };
        let waits = if member.party == you {
            yours
        } else {
            others.iter().find(|(party, _)| *party == member.party).map(|(_, waits)| *waits)
        };
        people.push(Person {
            party: member.party,
            you: member.party == you,
            name: name.into(),
            role: member.role,
            waits,
        });
    }
    if let Some(at) = people.iter().position(|person| person.party == you) {
        let reader = people.remove(at);
        people.insert(0, reader);
    }
    people
}

// Sets ignore document order; the first person of each party owns its role.
fn shared_role(project: &c::Project) -> Option<c::HeldRole> {
    let mut parties = std::collections::BTreeSet::new();
    let mut roles = Vec::new();
    let mut shared = None;
    for member in &project.members {
        if member.kind == c::PartyKind::Person && parties.insert(member.party) {
            if !roles.contains(&member.role) {
                roles.push(member.role);
            }
            shared = Some(member.role);
        }
    }
    if project.cut || parties.len() < 2 || roles.len() != 1 {
        return None;
    }
    match shared {
        Some(c::HeldRole::Policy { .. }) | None => None,
        Some(role) => Some(role),
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "each seed checks independent people and standing-role facts, parts and moves"
)]
fn sweep(seed: u64) -> bool {
    let mut rng = Rng::new(seed);
    let mut project = random_project(&mut rng);
    let you = rng.between(1, READER_PARTIES);
    let yours = if rng.chance(500) { Some(random_waits(&mut rng)) } else { None };
    let others: Vec<_> =
        (0..rng.between(0, MAX_OTHERS)).map(|_| (rng.between(1, READER_PARTIES), random_waits(&mut rng))).collect();
    let expected = oracle(&project, you, yours, &others);
    let mut facts = team::gather(&project, you, yours, &others);
    assert_eq!(
        facts.people.as_slice(),
        expected,
        "seed {seed}: first Person members once, reader first, document order, names, roles and first waits"
    );
    assert_eq!(facts.cut, project.cut, "seed {seed}: document cut");
    assert_eq!(facts.triager, None, "seed {seed}: triager is unread");
    let count = rng.between(0, 16);
    facts.roles = List::with_capacity(16);
    for task in 1..=count {
        facts
            .roles
            .push(team::Role {
                task,
                name: format!("role-{task}").into_bytes().into_boxed_slice(),
                now: if rng.chance(500) { Some(b"Its first line".as_slice().into()) } else { None },
                spent: rng.next_u64(),
                budget: rng.next_u64(),
            })
            .expect("roles fit watch");
    }
    facts.roles_cut = rng.chance(500);
    facts.selected = if rng.chance(500) { Some(rng.between(0, 20)) } else { None };
    let chosen = facts.selected.filter(|task| (1..=count).contains(task)).or(if count == 0 { None } else { Some(1) });
    if rng.chance(750) {
        let task = if rng.chance(750) { chosen.unwrap_or(99) } else { 99 };
        let mut held = None;
        if rng.chance(500) {
            held = Some(Box::new(c::ChipHold {
                placed: None,
                words: None,
                circle: None,
                account: None,
                source: None,
                reason: c::HoldCategory::Budget,
                revision: 1,
                before: c::PriorPhase::Waiting,
                escalation: None,
                tries: c::FailureTries { transient: 0, permanent: 0, run: 0, agent: 0, lost: 0, invalid: 0 },
                hold_wait_since: None,
            }));
        }
        facts.card = Some(Box::new(team::Card {
            task,
            name: b"Role".as_slice().into(),
            made: None,
            requester: None,
            hold: held,
            paused: rng.chance(500),
            pause: rng.chance(500),
            resume: rng.chance(500),
            sessions: random_sessions(&mut rng),
            now: if rng.chance(500) {
                Some(team::Now {
                    body: if rng.chance(500) { Box::default() } else { b"- Now".as_slice().into() },
                    changed: None,
                    cut: rng.chance(500),
                })
            } else {
                None
            },
        }));
    }
    let detail = facts.card.as_ref().filter(|card| Some(card.task) == chosen);
    let parts = team::parts(&facts);
    assert_eq!(parts.standing_roles, count != 0, "seed {seed}: Standing roles needs a listed role");
    assert_eq!(parts.roles_cut, count != 0 && facts.roles_cut, "seed {seed}: cut belongs to the list");
    assert_eq!(parts.card, chosen, "seed {seed}: listed selection, else first");
    assert_eq!(
        parts.pause,
        detail.is_some_and(|card| card.hold.is_none() && card.pause),
        "seed {seed}: Pause needs Stop and no hold"
    );
    assert_eq!(
        parts.resume,
        detail.is_some_and(|card| card.hold.is_some() && card.paused && card.resume),
        "seed {seed}: Resume needs Release and paused alone"
    );
    assert_eq!(
        parts.sessions,
        detail.is_some_and(|card| !card.sessions.is_empty()),
        "seed {seed}: Sessions needs rows in the shown card"
    );
    assert_eq!(parts.right, parts.sessions, "seed {seed}: right column needs a part");
    assert_eq!(
        parts.now,
        detail.is_some_and(|card| card.now.as_ref().is_some_and(|now| !now.body.is_empty())),
        "seed {seed}: Now needs a nonempty read"
    );
    assert_eq!(
        team::parts(&facts),
        team::Parts { people: !expected.is_empty(), shared: shared_role(&project), triager: false, ..parts },
        "seed {seed}: parts show exactly when their facts exist"
    );

    for member in project.members.iter_mut() {
        member.role = random_role(&mut rng);
    }
    let moved = team::gather(&project, you, yours, &others);
    assert_eq!(team::parts(&moved).shared, shared_role(&project), "seed {seed}: shared after roles move");
    assert_eq!(
        moved.people.iter().map(|person| person.party).collect::<Vec<_>>(),
        expected.iter().map(|person| person.party).collect::<Vec<_>>(),
        "seed {seed}: a move of roles never hides or reorders a person"
    );
    team::parts(&facts).shared.is_some()
}

#[test]
fn team_parts_follow_their_rules() {
    let mut shared = 0;
    for seed in 1..=SEEDS {
        match catch_unwind(|| sweep(seed)) {
            Ok(some) => shared += u32::from(some),
            Err(error) => panic!("seed {seed}: Team parts sweep panicked: {error:?}"),
        }
    }
    assert!(shared > 0, "the sweep reaches a shared built-in role");
}

fn random_sessions(rng: &mut Rng) -> List<team::Session> {
    let mut rows = List::with_capacity(8);
    for _ in 0..rng.between(0, 8) {
        rows.push(team::Session {
            start: skein_lib::Wall::from_nanos(rng.next_u64()),
            end: if rng.chance(500) { Some(skein_lib::Wall::from_nanos(rng.next_u64())) } else { None },
            words: if rng.chance(500) { Some(b"Finished words".as_slice().into()) } else { None },
            spent: rng.next_u64(),
        })
        .expect("eight sessions");
    }
    rows
}

//! Team's parts and moves (temper/docs/design/client/pages/README.md, Team;
//! temper/docs/design/draft/surfaces.md, 12 and 14).
use jig_client::contract as c;
use jig_client_kit::stories;
use skein_lib::{List, Rng, Wall};
use std::panic::catch_unwind;
use temper_client_domain::pages::team::{self, Person, Waits};
use temper_client_protocol_wire::v1 as w;

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
fn oracle(
    project: &c::Project,
    you: u64,
    yours: Option<Waits>,
    others: &[(u64, Waits)],
    changes: Option<&w::ChangesPanel>,
) -> Vec<Person> {
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
            signed: signed_oracle(changes, member.party),
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

fn random_rule(rng: &mut Rng) -> c::WakeRule {
    use skein_lib::Duration;
    match rng.below(4) {
        0 => c::WakeRule::Never,
        1 => c::WakeRule::Immediate,
        2 => c::WakeRule::Paced { span: Duration::from_secs(rng.between(1, 7200)) },
        _ => c::WakeRule::Batch { count: random_count(rng), age: Duration::from_secs(rng.between(1, 7200)) },
    }
}
fn random_wakes(rng: &mut Rng) -> skein_lib::List<team::wakes::Row> {
    use skein_lib::{Duration, Wall};
    use team::wakes::{Interest, Row};
    let policy = c::WakePolicy {
        words: random_rule(rng),
        notices: random_rule(rng),
        news: random_rule(rng),
        results: c::ResultsWake::Never,
        questions: rng.chance(500),
        answers: rng.chance(500),
        timers: rng.chance(500),
    };
    let entries = (0..rng.between(0, 20))
        .map(|number| c::Subscription {
            number,
            kind: match rng.below(3) {
                0 => c::SubscriptionKind::Timer {
                    at: Wall::from_nanos(rng.next_u64()),
                    period: if rng.chance(500) { Some(Duration::from_secs(rng.between(1, 604_800))) } else { None },
                },
                1 => c::SubscriptionKind::Task {
                    target: rng.between(1, 100),
                    held: rng.chance(500),
                    result: rng.chance(500),
                },
                _ => c::SubscriptionKind::Topic {
                    connector: u16::try_from(rng.between(1, 4)).expect("bounded connector"),
                    topic: rng.next_u64(),
                },
            },
        })
        .collect::<Vec<_>>();
    let subscriptions = c::Subscriptions { entries: List::from_boxed(entries.clone().into_boxed_slice()) };
    let mut seen = std::collections::BTreeSet::new();
    let expected = entries
        .iter()
        .filter_map(|entry| {
            Some(match entry.kind {
                c::SubscriptionKind::Timer { at, period } => {
                    Row { interest: Interest::Timer { at, period, wakes: policy.timers }, class: c::WakeRule::Never }
                }
                c::SubscriptionKind::Task { target, held, result } => {
                    Row { interest: Interest::Task { target, held, result }, class: policy.notices }
                }
                c::SubscriptionKind::Topic { connector, .. } => {
                    if !seen.insert(connector) {
                        return None;
                    }
                    let count = u32::try_from(
                        entries
                            .iter()
                            .filter(
                                |e| matches!(e.kind, c::SubscriptionKind::Topic { connector: c, .. } if c == connector),
                            )
                            .count(),
                    )
                    .expect("bounded topics");
                    Row { interest: Interest::Topics { connector, count }, class: policy.news }
                }
            })
        })
        .collect::<Vec<_>>();
    let rows = team::wakes::rows(&subscriptions, &policy);
    assert_eq!(rows.as_slice(), expected, "random subscription ordering, grouping and classes");
    rows
}
fn random_rules(rng: &mut Rng) -> Option<team::Rules> {
    use temper_client_domain::pages::roles::Read;
    use temper_client_protocol_wire::v1 as w;
    let state = rng.below(6);
    let fact = w::Fact::new(
        &w::CEILINGS,
        w::FactParts {
            label: if state == 3 { b"Model".as_slice() } else { b"Asked to" }.into(),
            value: if state == 4 { b"".as_slice() } else { b"Reproduce before routing." }.into(),
        },
    )
    .expect("bounded charter view");
    let card = w::Card::new(
        &w::CEILINGS,
        w::CardParts {
            name: b"Prover".as_slice().into(),
            who: b"an agent".as_slice().into(),
            facts: List::from_boxed(Box::new([fact])),
        },
    )
    .expect("bounded charter view");
    let view = w::View::Charter(
        w::CharterView::new(&w::CEILINGS, w::CharterViewParts { charter: 13, card }).expect("bounded charter view"),
    );
    let read = match state {
        0 => Read::Reading,
        1 => Read::Unreadable,
        _ => Read::Drawn(&view),
    };
    let rules = team::asked(13, read);
    assert_eq!(rules.is_some(), state == 2 || state == 5, "random read states and Asked to presence");
    rules
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
    let changes = random_changes(&mut rng);
    let expected = oracle(&project, you, yours, &others, changes.as_ref());
    let mut facts = team::gather(&project, you, yours, &others, changes.as_ref(), WALL);
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
                tries: c::FailureTries {
                    transient: 0,
                    permanent: 0,
                    run: 0,
                    agent: 0,
                    lost: 0,
                    invalid: 0,
                    infrastructure: 0,
                },
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
            rules: random_rules(&mut rng),
            wakes: random_wakes(&mut rng),
            wakes_cut: rng.chance(500),
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
    assert_eq!(parts.right, parts.sessions || parts.wakes, "seed {seed}: right column needs a part");
    assert_eq!(
        parts.now,
        detail.is_some_and(|card| card.now.as_ref().is_some_and(|now| !now.body.is_empty())),
        "seed {seed}: Now needs a nonempty read"
    );
    assert_eq!(parts.rules, detail.is_some_and(|card| card.rules.is_some()), "seed {seed}: rules need Asked to read");
    assert_eq!(parts.wakes, detail.is_some_and(|card| !card.wakes.is_empty()), "seed {seed}: wakes need subscriptions");
    assert_eq!(
        parts.wakes_cut,
        parts.wakes && detail.expect("wakes need a detail").wakes_cut,
        "seed {seed}: cut only with rows"
    );
    assert!(!parts.wakes || parts.right, "seed {seed}: wakes keep their right column");
    assert_eq!(
        team::parts(&facts),
        team::Parts { people: !expected.is_empty(), shared: shared_role(&project), triager: false, ..parts.clone() },
        "seed {seed}: parts show exactly when their facts exist"
    );
    for person in &expected {
        let sentence = parts.signed(person.party);
        assert_eq!(
            sentence.is_some(),
            person.signed.is_some_and(|count| count.count > 0),
            "seed {seed}: only positive counts speak"
        );
        if let Some(sentence) = sentence {
            assert!(!sentence.is_empty(), "seed {seed}: no empty sentence");
            assert!(sentence.starts_with(b"Signed ") && sentence.ends_with(b" last week."));
        }
    }
    assert_eq!(
        parts.signed.len(),
        u32::try_from(expected.iter().filter(|person| person.signed.is_some_and(|count| count.count > 0)).count())
            .expect("few people")
    );

    for member in project.members.iter_mut() {
        member.role = random_role(&mut rng);
    }
    let moved = team::gather(&project, you, yours, &others, changes.as_ref(), WALL);
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

// A small landing document with repeated tasks, signers and both sides of the window.
const DAY: u64 = 86_400_000_000_000;
const WALL: Wall = Wall::from_nanos(20 * DAY);
fn random_changes(rng: &mut Rng) -> Option<w::ChangesPanel> {
    if rng.chance(200) {
        return None;
    }
    let mut rows = List::with_capacity(8);
    for _ in 0..rng.between(0, 8) {
        let mut signed = List::with_capacity(8);
        for _ in 0..rng.between(0, 8) {
            signed
                .push(
                    w::ChangesSigned::new(
                        &w::CEILINGS,
                        w::ChangesSignedParts { party: rng.between(1, READER_PARTIES), head: [1; 32], at: 0 },
                    )
                    .expect("signer"),
                )
                .expect("eight signers");
        }
        let at = if rng.chance(200) { WALL.as_nanos() + 1 } else { WALL.as_nanos() - rng.between(0, 9) * DAY };
        rows.push(
            w::ChangesLanded::new(
                &w::CEILINGS,
                w::ChangesLandedParts {
                    task: rng.between(1, MEMBER_PARTIES),
                    title: b"Change".as_slice().into(),
                    at,
                    commit: [2; 32],
                    default_branch: true,
                    signed,
                },
            )
            .expect("landing"),
        )
        .expect("eight rows");
    }
    Some(
        w::ChangesPanel::new(
            &w::CEILINGS,
            w::ChangesPanelParts {
                project: 1,
                landing: b"main".as_slice().into(),
                queues: List::with_capacity(0),
                more: 0,
                landed: List::with_capacity(0),
                landed_more: 0,
                landed_default: rows,
                landed_default_more: u32::from(rng.chance(500)),
            },
        )
        .expect("changes"),
    )
}
fn signed_oracle(changes: Option<&w::ChangesPanel>, party: u64) -> Option<team::Signed> {
    let changes = changes?;
    let in_week = |at| at <= WALL.as_nanos() && WALL.as_nanos() - at <= 7 * DAY;
    let tasks: std::collections::BTreeSet<_> = changes
        .landed_default()
        .iter()
        .filter(|row| in_week(row.at()) && row.signed().iter().any(|signer| signer.party() == party))
        .map(w::ChangesLanded::task)
        .collect();
    let floor = changes.landed_default_more() > 0
        && changes.landed_default().iter().map(w::ChangesLanded::at).min().is_some_and(in_week);
    Some(team::Signed { count: u32::try_from(tasks.len()).expect("few tasks"), floor })
}

//! Density and settings moves (temper/docs/design/draft/surfaces.md, 12 and 14;
//! temper/docs/design/client/pages/README.md and temper/docs/design/client/view.md, 3).
use skein_lib::{List, Rng, Wall};
use std::{cmp::Reverse, collections::BTreeMap, panic::catch_unwind};
use temper_client_domain::{
    forge::ways::Ways,
    pages::{rail, work},
};
use work::{Belongs, Card, Change, Chat, Goal, Item, Landed, Where};

// The sibling public-address sweep uses 2,000 independently replayable seeds.
const SEEDS: u64 = 2_000;
// Small task pools deliberately repeat rows; two more owner numbers can be absent.
const TASKS: u64 = 6;
const OWNERS: u64 = TASKS + 2;
const ROWS: u64 = 8;
const CHANGE_ROWS: u64 = 20;
// C4's inclusive seven-day window, expressed in Wall's nanoseconds.
const WEEK: u64 = 604_800_000_000_000;

fn small_count(rng: &mut Rng) -> u32 {
    let counts = [0, 1, 2, u32::MAX];
    *counts.get(usize::try_from(rng.below(skein_lib::len_u64(counts.len()))).expect("count index")).expect("count")
}

fn maybe_count(rng: &mut Rng) -> Option<u32> {
    if rng.chance(250) { None } else { Some(small_count(rng)) }
}

fn rail_facts(rng: &mut Rng) -> rail::Facts {
    rail::Facts {
        projects: small_count(rng),
        needs: small_count(rng),
        needs_cut: rng.chance(500),
        members: maybe_count(rng),
        standing_roles: maybe_count(rng),
        asking: rng.chance(500),
        away: rng.chance(500),
        drafts: rng.chance(500),
        beginning: rng.chance(500),
    }
}

// Read the navigation table first, then apply its separate To begin cut.
fn rail_oracle(facts: &rail::Facts) -> rail::Parts {
    let mut parts = rail::Parts {
        all_projects: facts.projects > 1,
        needs_you: true,
        count: facts.needs,
        cut: facts.needs_cut,
        work: true,
        team: facts.members.is_some_and(|count| count > 1) || facts.standing_roles.is_some_and(|count| count > 0),
        ask: facts.asking,
        how: true,
        away: facts.away,
        new: if facts.drafts { rail::New::DraftOrChange } else { rail::New::Change },
    };
    if facts.beginning {
        parts.needs_you = false;
        parts.work = false;
        parts.team = false;
        parts.new = rail::New::Hidden;
    }
    parts
}

// Read the phone's navigation table directly from its facts, without the rail oracle.
fn tabs_oracle(facts: &rail::Facts) -> rail::Tabs {
    rail::Tabs {
        needs: !facts.beginning,
        count: facts.needs,
        cut: facts.needs_cut,
        work: !facts.beginning,
        ask: facts.asking,
        deployment: facts.beginning,
    }
}

fn check_rail(facts: &rail::Facts, seed: u64) -> rail::Parts {
    let parts = rail::parts(facts);
    assert_eq!(parts, rail_oracle(facts), "seed {seed}: rail facts {facts:?}");
    assert_eq!(rail::tabs(&parts), tabs_oracle(facts), "seed {seed}: phone tabs for rail facts {facts:?}");
    parts
}

fn wall(rng: &mut Rng) -> Wall {
    let arbitrary = rng.next_u64();
    let values = [0, 1, WEEK - 1, WEEK, WEEK + 1, u64::MAX - 1, u64::MAX, arbitrary];
    Wall::from_nanos(
        *values.get(usize::try_from(rng.below(skein_lib::len_u64(values.len()))).expect("wall index")).expect("wall"),
    )
}

fn activity(rng: &mut Rng, now: Wall) -> Wall {
    // A small time pool gives ties; every edge of the landed interval gets a draw.
    let end = now.as_nanos();
    let start = end.saturating_sub(WEEK);
    let values =
        [0, start.saturating_sub(1), start, start.saturating_add(1), end.saturating_sub(1), end, end.saturating_add(1)];
    Wall::from_nanos(
        *values
            .get(usize::try_from(rng.below(skein_lib::len_u64(values.len()))).expect("activity index"))
            .expect("activity"),
    )
}

fn work_facts(rng: &mut Rng, now: Wall) -> work::Facts {
    let mut facts = work::Facts::default();
    for _ in 0..rng.between(0, ROWS) {
        facts
            .goals
            .push(Goal { task: rng.between(1, TASKS), live: rng.chance(500), activity: activity(rng, now) })
            .expect("uncapped goals");
    }
    for _ in 0..rng.between(0, ROWS) {
        facts
            .chats
            .push(Chat { task: rng.between(1, TASKS), activity: activity(rng, now), draft: rng.chance(500) })
            .expect("uncapped chats");
    }
    for _ in 0..rng.between(0, CHANGE_ROWS) {
        let at = if rng.chance(500) {
            Where::Queued { place: u32::try_from(rng.between(1, 3)).expect("queue place"), since: activity(rng, now) }
        } else {
            Where::Landed { at: activity(rng, now) }
        };
        let belongs = match rng.below(3) {
            0 => Belongs::Goal(rng.between(1, OWNERS)),
            1 => Belongs::Chat(rng.between(1, OWNERS)),
            _ => Belongs::Nothing,
        };
        facts.changes.push(Change { task: rng.between(1, CHANGE_ROWS), at, belongs }).expect("uncapped changes");
    }
    for _ in 0..rng.between(0, ROWS) {
        let ready = rng.chance(500); let goal = rng.chance(500).then(|| rng.between(1, TASKS));
        let draft = Landed {
            forge: u16::try_from(rng.below(3)).expect("forge"),
            owner: format!("owner{}", rng.below(2)).into_bytes().into(), name: format!("repo{}", rng.below(2)).into_bytes().into(),
            path: format!("docs/draft{}.md", rng.below(TASKS)).into_bytes().into(), at: activity(rng, now), title: b"Draft".as_slice().into(), listed: ready && goal.is_none(),
        };
        facts.landed.push(draft).expect("uncapped landed drafts");
    }
    facts
}
type DraftKey = (u16, Vec<u8>, Vec<u8>, Vec<u8>);
type Activity = (Reverse<Wall>, u8, u64, DraftKey);
fn draft_key(row: &Landed) -> DraftKey { (row.forge, row.owner.to_vec(), row.name.to_vec(), row.path.to_vec()) }

// The oracle uses first-row maps and standard sorting, independent of place's
// traversal and insertion order. No production placement helper is its oracle.
struct Oracle {
    goals: BTreeMap<u64, Goal>,
    changes: BTreeMap<u64, Change>,
    chats: BTreeMap<u64, Chat>,
    landed: BTreeMap<DraftKey, (u32, Landed)>,
}

impl Oracle {
    fn new(facts: &work::Facts) -> Self {
        let mut oracle = Self { goals: BTreeMap::new(), changes: BTreeMap::new(), chats: BTreeMap::new(), landed: BTreeMap::new() };
        for row in &facts.goals {
            oracle.goals.entry(row.task).or_insert(*row);
        }
        for row in &facts.changes {
            oracle.changes.entry(row.task).or_insert(*row);
        }
        for row in &facts.chats {
            oracle.chats.entry(row.task).or_insert(*row);
        }
        for (index, row) in facts.landed.iter().enumerate() {
            oracle.landed.entry(draft_key(row)).or_insert((u32::try_from(index).expect("draft index"), row.clone()));
        }
        oracle
    }

    fn owner(&self, change: &Change) -> Item {
        match change.belongs {
            Belongs::Goal(task) if self.goals.get(&task).is_some_and(|goal| goal.live) => Item::Goal(task),
            Belongs::Chat(task) if self.chats.contains_key(&task) => Item::Chat(task),
            Belongs::Goal(_) | Belongs::Chat(_) | Belongs::Nothing => Item::Change(change.task),
        }
    }

    fn held(&self, owner: Belongs) -> Vec<u64> {
        let mut queued = Vec::new();
        let mut landed = Vec::new();
        for row in self.changes.values().filter(|row| row.belongs == owner) {
            match row.at {
                Where::Queued { place, .. } => queued.push((place, row.task)),
                Where::Landed { at } => landed.push((Reverse(at), row.task)),
            }
        }
        queued.sort_unstable();
        landed.sort_unstable();
        queued.into_iter().map(|(_, task)| task).chain(landed.into_iter().map(|(_, task)| task)).collect()
    }

    fn activity(&self, item: Item) -> Activity {
        let (at, task) = match item {
            Item::Goal(task) => (self.goals.get(&task).expect("listed goal").activity, task),
            Item::Chat(task) | Item::Draft(task) => (self.chats.get(&task).expect("listed chat").activity, task),
            Item::Landed(index) => {
                let row = &self.landed.values().find(|(i, _)| *i == index).expect("listed landed draft").1;
                return (Reverse(row.at), 1, 0, draft_key(row));
            }
            Item::Change(task) => {
                let Where::Queued { since, .. } = self.changes.get(&task).expect("listed change").at else {
                    panic!("a standalone card must be queued")
                };
                (since, task)
            }
        };
        (Reverse(at), 0, task, (0, Vec::new(), Vec::new(), Vec::new()))
    }

    fn check(&self, placed: &work::Work, now: Wall, seed: u64) {
        let mut flight = Vec::new();
        let mut drafts = Vec::new();
        for goal in self.goals.values().filter(|goal| goal.live) {
            flight.push((item_key(Item::Goal(goal.task)), self.held(Belongs::Goal(goal.task))));
        }
        for chat in self.chats.values() {
            if self.changes.values().any(|row| row.belongs == Belongs::Chat(chat.task) && queued(row)) {
                flight.push((item_key(Item::Chat(chat.task)), self.held(Belongs::Chat(chat.task))));
            }
            if chat.draft {
                drafts.push((item_key(Item::Draft(chat.task)), Vec::new()));
            }
        }
        for change in self.changes.values().filter(|row| queued(row)) {
            if let Item::Change(task) = self.owner(change) {
                flight.push((item_key(Item::Change(task)), Vec::new()));
            }
        }
        for (index, row) in self.landed.values() {
            if row.listed { drafts.push((item_key(Item::Landed(*index)), Vec::new())); }
        }
        flight.sort_unstable();
        drafts.sort_unstable();
        assert_eq!(
            cards(&placed.in_flight),
            flight,
            "seed {seed}: In flight places each live item exactly once on its owner"
        );
        assert_eq!(cards(&placed.drafts), drafts, "seed {seed}: each draft has one empty Draft card");
        for (name, section) in [("In flight", &placed.in_flight), ("Drafts", &placed.drafts)] {
            let order: Vec<_> = section.iter().map(|card| self.activity(card.item)).collect();
            assert!(
                order.windows(2).all(|pair| pair[0] <= pair[1]),
                "seed {seed}: {name} newest first, ties by task then landed identity: {order:?}"
            );
        }
        let mut landed: Vec<_> = self
            .changes
            .values()
            .filter_map(|row| match row.at {
                Where::Landed { at } if at <= now && now.as_nanos() - at.as_nanos() <= WEEK => {
                    Some((Reverse(at), row.task))
                }
                Where::Queued { .. } | Where::Landed { .. } => None,
            })
            .collect();
        landed.sort_unstable();
        assert_eq!(
            placed.landed.iter().copied().collect::<Vec<_>>(),
            landed.into_iter().map(|(_, task)| task).collect::<Vec<_>>(),
            "seed {seed}: Landed is exactly the inclusive week, newest first, ties by task"
        );
        let chats = self
            .chats
            .keys()
            .filter(|task| self.changes.values().any(|row| row.belongs == Belongs::Chat(**task)))
            .count();
        let counts = work::Filters {
            drafts: u32::try_from(drafts.len()).expect("draft count"),
            goals: u32::try_from(self.goals.len()).expect("goal count"),
            changes: u32::try_from(self.changes.len()).expect("change count"),
            chats: u32::try_from(chats).expect("chat count"),
        };
        assert_eq!(
            placed.filters, counts,
            "seed {seed}: C4 counts first rows, including ended goals and landed-only chats"
        );
    }
}

fn queued(change: &Change) -> bool {
    matches!(change.at, Where::Queued { .. })
}

fn item_key(item: Item) -> (u8, u64) {
    match item {
        Item::Goal(task) => (0, task),
        Item::Chat(task) => (1, task),
        Item::Change(task) => (2, task),
        Item::Draft(task) => (3, task),
        Item::Landed(index) => (4, u64::from(index)),
    }
}

fn cards(section: &List<Card>) -> Vec<((u8, u64), Vec<u64>)> {
    let mut cards: Vec<_> =
        section.iter().map(|card| (item_key(card.item), card.changes.iter().copied().collect())).collect();
    cards.sort_unstable();
    cards
}

fn ways(rng: &mut Rng) -> Ways {
    Ways { drafts: rng.chance(500), goals: rng.chance(500), changes: rng.chance(500), way: small_count(rng) }
}

fn check_shown(filters: &work::Filters, settings: Option<Ways>, seed: u64) {
    let overrides = settings
        .map_or_else(work::Overrides::default, |ways| work::Overrides { goals: !ways.goals, changes: !ways.changes });
    let shown = work::shown(filters, overrides);
    assert_eq!(
        shown,
        work::Shown {
            drafts: filters.drafts != 0,
            goals: filters.goals != 0 && settings.is_none_or(|ways| ways.goals),
            changes: filters.changes != 0 && settings.is_none_or(|ways| ways.changes),
            chats: filters.chats != 0,
        },
        "seed {seed}: C7 nonempty filters and read Hidden overrides, settings {settings:?}"
    );
}

fn sweep(seed: u64) {
    let mut rng = Rng::new(seed);
    let mut rail = rail_facts(&mut rng);
    check_rail(&rail, seed);
    let now = wall(&mut rng);
    let facts = work_facts(&mut rng, now);
    let saved = facts.clone();
    let oracle = Oracle::new(&facts);
    let before = work::place(&facts, now);
    oracle.check(&before, now, seed);
    let first = ways(&mut rng);
    let mut second = ways(&mut rng);
    if second == first {
        second.drafts = !second.drafts;
    }
    // Away is shown by the classes or an on Away row, independently of kind 7.
    let classes_ask = rng.chance(500);
    let away_on = rng.chance(500);
    let mut revisions = Vec::new();
    check_shown(&before.filters, None, seed);
    for settings in [first, second] {
        check_shown(&before.filters, Some(settings), seed);
        rail.drafts = settings.drafts;
        rail.away = classes_ask || away_on;
        revisions.push(check_rail(&rail, seed));
    }
    let [old, new] = revisions.as_slice() else { panic!("two settings revisions") };
    assert_eq!(
        (old.needs_you, old.work, old.how, old.ask, old.all_projects, old.team),
        (new.needs_you, new.work, new.how, new.ask, new.all_projects, new.team),
        "seed {seed}: a settings move preserves the rail's independent entries"
    );
    assert_eq!(rail::tabs(old), rail::tabs(new), "seed {seed}: a settings move preserves the phone's tabs");
    assert_eq!(work::place(&facts, now), before, "seed {seed}: settings never hide a live item under All");
    assert_eq!(facts, saved, "seed {seed}: placement preserves its facts");
}

#[test]
fn parts_and_live_work_survive_random_settings_moves() {
    for seed in 1..=SEEDS {
        // sweep has no input but this seed; replay a failure with sweep(seed) alone.
        assert!(catch_unwind(|| sweep(seed)).is_ok(), "seed {seed}: rail and Work sweep panicked");
    }
}

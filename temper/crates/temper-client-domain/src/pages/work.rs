//! Work places each item by its own facts (temper/docs/design/client/README.md, 3.5).

use skein_lib::{Duration, List, Set, Wall};

/// The goals, changes and person's chats read for the project.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Facts {
    pub goals: List<Goal>,
    pub changes: List<Change>,
    pub chats: List<Chat>,
}

/// A goal, live or ended, with its last activity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Goal {
    pub task: u64,
    pub live: bool,
    pub activity: Wall,
}

/// A change on its way or landed, and where it came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Change {
    pub task: u64,
    pub at: Where,
    pub belongs: Belongs,
}

/// A change's place in its queue or its landing time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Where {
    Queued { place: u32, since: Wall },
    Landed { at: Wall },
}

/// The goal or chat a change belongs to, else nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Belongs {
    Goal(u64),
    Chat(u64),
    Nothing,
}

/// A person's chat, with its last activity and whether it has a draft.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chat {
    pub task: u64,
    pub activity: Wall,
    pub draft: bool,
}

/// The live goal, change, chat or draft a card opens.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item {
    Goal(u64),
    Change(u64),
    Chat(u64),
    Draft(u64),
}

/// An item's card, with changes on their way by place, then landed newest first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Card {
    pub item: Item,
    pub changes: List<u64>,
}

/// Work's In flight, Drafts and Landed this week sections and its filters' counts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Work {
    pub in_flight: List<Card>,
    pub drafts: List<Card>,
    pub landed: List<u64>,
    pub filters: Filters,
}

/// What each of Work's filters holds; a filter is drawn while its count is more than zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Filters {
    pub drafts: u32,
    pub goals: u32,
    pub changes: u32,
    pub chats: u32,
}

impl Default for Facts {
    fn default() -> Self {
        Self { goals: List::uncapped(), changes: List::uncapped(), chats: List::uncapped() }
    }
}

impl Default for Work {
    fn default() -> Self {
        Self {
            in_flight: List::uncapped(),
            drafts: List::uncapped(),
            landed: List::uncapped(),
            filters: Filters::default(),
        }
    }
}

#[expect(clippy::derivable_impls, reason = "the strict subset forbids deriving Default")]
impl Default for Filters {
    fn default() -> Self {
        Self { drafts: 0, goals: 0, changes: 0, chats: 0 }
    }
}

const LANDED_THIS_WEEK: Duration = Duration::from_secs(604_800);

#[derive(Debug)]
struct ActiveCard {
    activity: Wall,
    task: u64,
    card: Card,
}

/// Place each item by its own facts, never by the project's settings or its way.
#[must_use]
pub fn place(facts: &Facts, wall: Wall) -> Work {
    let changes = ordered_changes(&facts.changes);
    let mut work = Work::default();
    work.filters.changes = changes.len();
    let mut in_flight = List::uncapped();
    let mut drafts = List::with_capacity(facts.chats.len());
    let mut goals = Set::with_capacity(facts.goals.len());
    let mut live_goals = Set::with_capacity(facts.goals.len());
    for goal in &facts.goals {
        if !goals.insert(goal.task).expect("one key per goal row") {
            continue;
        }
        if goal.live {
            live_goals.insert(goal.task).expect("no more live goals than goal rows");
            let card = Card { item: Item::Goal(goal.task), changes: belonging(&changes, Belongs::Goal(goal.task)) };
            insert_card(&mut in_flight, ActiveCard { activity: goal.activity, task: goal.task, card });
        }
    }
    work.filters.goals = goals.len();
    let mut chats = Set::with_capacity(facts.chats.len());
    for chat in &facts.chats {
        if !chats.insert(chat.task).expect("one key per chat row") {
            continue;
        }
        let held = belonging(&changes, Belongs::Chat(chat.task));
        if !held.is_empty() {
            work.filters.chats = work.filters.chats.saturating_add(1);
        }
        if has_queued(&changes, Belongs::Chat(chat.task)) {
            let card = Card { item: Item::Chat(chat.task), changes: held };
            insert_card(&mut in_flight, ActiveCard { activity: chat.activity, task: chat.task, card });
        }
        if chat.draft {
            let card = Card { item: Item::Draft(chat.task), changes: List::with_capacity(0) };
            insert_card(&mut drafts, ActiveCard { activity: chat.activity, task: chat.task, card });
        }
    }
    let week_start = Wall::from_nanos(wall.as_nanos().saturating_sub(LANDED_THIS_WEEK.as_nanos()));
    for change in &changes {
        match change.at {
            Where::Queued { place: _, since } => {
                let held = match change.belongs {
                    Belongs::Goal(task) => live_goals.contains(&task),
                    Belongs::Chat(task) => chats.contains(&task),
                    Belongs::Nothing => false,
                };
                if !held {
                    let card = Card { item: Item::Change(change.task), changes: List::with_capacity(0) };
                    insert_card(&mut in_flight, ActiveCard { activity: since, task: change.task, card });
                }
            }
            Where::Landed { at } => {
                if at >= week_start && at <= wall {
                    work.landed.push(change.task).expect("no more landed rows than changes");
                }
            }
        }
    }
    work.filters.drafts = drafts.len();
    for active in in_flight.into_boxed() {
        work.in_flight.push(active.card).expect("one card per live item");
    }
    for active in drafts.into_boxed() {
        work.drafts.push(active.card).expect("one card per draft");
    }
    work
}

fn newer(left: Wall, left_task: u64, right: Wall, right_task: u64) -> bool {
    left > right || (left == right && left_task < right_task)
}

fn insert_card(cards: &mut List<ActiveCard>, mut card: ActiveCard) {
    for next in cards.iter_mut() {
        if newer(card.activity, card.task, next.activity, next.task) {
            core::mem::swap(next, &mut card);
        }
    }
    cards.push(card).expect("one card per listed item");
}

fn ordered_changes(changes: &List<Change>) -> List<Change> {
    let mut seen = Set::with_capacity(changes.len());
    let mut ordered = List::with_capacity(changes.len());
    for change in changes {
        if !seen.insert(change.task).expect("one key per change row") {
            continue;
        }
        let mut change = *change;
        for next in ordered.iter_mut() {
            if change_before(&change, next) {
                core::mem::swap(next, &mut change);
            }
        }
        ordered.push(change).expect("one change per input row");
    }
    ordered
}

fn change_before(left: &Change, right: &Change) -> bool {
    match left.at {
        Where::Queued { place: left_place, since: _ } => match right.at {
            Where::Queued { place: right_place, since: _ } => {
                left_place < right_place || (left_place == right_place && left.task < right.task)
            }
            Where::Landed { at: _ } => true,
        },
        Where::Landed { at: left_at } => match right.at {
            Where::Queued { place: _, since: _ } => false,
            Where::Landed { at: right_at } => newer(left_at, left.task, right_at, right.task),
        },
    }
}

fn belonging(changes: &List<Change>, owner: Belongs) -> List<u64> {
    let mut held = List::with_capacity(changes.len());
    for change in changes {
        if change.belongs == owner {
            held.push(change.task).expect("no more owned changes than changes");
        }
    }
    held
}

fn has_queued(changes: &List<Change>, owner: Belongs) -> bool {
    for change in changes {
        if change.belongs == owner {
            match change.at {
                Where::Queued { place: _, since: _ } => return true,
                Where::Landed { at: _ } => {}
            }
        }
    }
    false
}

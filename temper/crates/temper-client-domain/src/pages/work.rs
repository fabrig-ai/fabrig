//! Work places each item by its own facts (temper/docs/design/client/pages/work.md, 1).

use alloc::boxed::Box;
use core::cmp::Reverse;
use jig_client::contract as c;
use skein_lib::{Duration, List, Map, Set, Wall};
use temper_client_protocol_wire::v1 as w;

/// The four lists read for Work.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Facts {
    pub goals: List<Goal>,
    pub changes: List<Change>,
    pub chats: List<Chat>,
    pub landed: List<LandedDraft>,
    pub landed_default: List<Landing>,
}

/// A landed draft's identity, card and listing facts (client/pages/work.md, 1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LandedDraft {
    pub forge: u16,
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
    pub path: Box<[u8]>,
    pub at: Wall,
    pub title: Box<[u8]>,
    pub listed: bool,
}
impl LandedDraft {
    #[must_use]
    pub fn row(row: &temper_client_protocol_wire::v1::LandedDraft) -> Self {
        Self {
            forge: row.forge(),
            owner: row.owner().into(),
            name: row.name().into(),
            path: row.path().into(),
            at: Wall::from_nanos(row.at()),
            title: row.title().into(),
            listed: row.ready() && row.goal().is_none(),
        }
    }
}

/// A goal, live or ended, with its last activity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Goal {
    pub task: u64,
    pub live: bool,
    pub activity: Wall,
}

/// A change on its way or landed, and where it came from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change {
    pub task: u64,
    pub at: Where,
    pub belongs: Belongs,
    pub signed: Option<Box<[u8]>>,
}

/// A change's place in its queue or its landing time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Where {
    Queued { place: u32, since: Wall },
    Landed { at: Wall, default_branch: bool },
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
    Landed(u32),
}

/// An item's card, with changes on their way by place, then landed newest first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Card {
    pub item: Item,
    pub changes: List<Row>,
}

/// A card's change and the words kept by its landed row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub task: u64,
    pub signed: Option<Box<[u8]>>,
}

/// A default-branch landing and its task's provenance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Landing {
    pub task: u64,
    pub commit: [u8; 32],
    pub at: Wall,
    pub belongs: Belongs,
    pub signed: Option<Box<[u8]>>,
}

/// A week's goal or independent change at its newest default-branch landing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Landed {
    pub item: Item,
    pub change: u64,
    pub commit: [u8; 32],
    pub at: Wall,
    pub signed: Option<Box<[u8]>>,
}

/// Work's In flight, Drafts and Landed this week sections and its filters' counts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Work {
    pub in_flight: List<Card>,
    pub drafts: List<Card>,
    pub landed: List<Landed>,
    pub goal_branches: u32,
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
        Self {
            goals: List::uncapped(),
            changes: List::uncapped(),
            chats: List::uncapped(),
            landed: List::uncapped(),
            landed_default: List::uncapped(),
        }
    }
}

impl Default for Work {
    fn default() -> Self {
        Self {
            in_flight: List::uncapped(),
            drafts: List::uncapped(),
            landed: List::uncapped(),
            goal_branches: 0,
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
    let mut drafts = List::uncapped();
    let mut goals = Set::with_capacity(facts.goals.len());
    let mut live_goals = Set::with_capacity(facts.goals.len());
    for goal in &facts.goals {
        if !goals.insert(goal.task).expect("one key per goal row") {
            continue;
        }
        if goal.live {
            live_goals.insert(goal.task).expect("no more live goals than goal rows");
            let card = Card { item: Item::Goal(goal.task), changes: belonging(&changes, Belongs::Goal(goal.task)) };
            insert_card(facts, &mut in_flight, ActiveCard { activity: goal.activity, task: goal.task, card });
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
            insert_card(facts, &mut in_flight, ActiveCard { activity: chat.activity, task: chat.task, card });
        }
        if chat.draft {
            let card = Card { item: Item::Draft(chat.task), changes: List::with_capacity(0) };
            insert_card(facts, &mut drafts, ActiveCard { activity: chat.activity, task: chat.task, card });
        }
    }
    let mut landed = Set::with_capacity(facts.landed.len());
    for (index, draft) in facts.landed.iter().enumerate() {
        let key = (draft.forge, draft.owner.clone(), draft.name.clone(), draft.path.clone());
        if !landed.insert(key).expect("one key per landed draft row") {
            continue;
        }
        if draft.listed {
            let index = u32::try_from(index).expect("landed facts fit");
            let card = Card { item: Item::Landed(index), changes: List::with_capacity(0) };
            insert_card(facts, &mut drafts, ActiveCard { activity: draft.at, task: 0, card });
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
                    insert_card(facts, &mut in_flight, ActiveCard { activity: since, task: change.task, card });
                }
            }
            Where::Landed { at: _, default_branch: _ } => {}
        }
    }
    work.goal_branches = goal_branches(&facts.changes, week_start, wall);
    work.landed = landed_this_week(&facts.landed_default, week_start, wall);
    work.filters.drafts = drafts.len();
    for active in in_flight.into_boxed() {
        work.in_flight.push(active.card).expect("one card per live item");
    }
    for active in drafts.into_boxed() {
        work.drafts.push(active.card).expect("one card per draft");
    }
    work
}

fn goal_branches(changes: &List<Change>, start: Wall, wall: Wall) -> u32 {
    let mut tasks = Set::with_capacity(changes.len());
    for change in changes {
        match change.at {
            Where::Landed { at, default_branch: false } if at >= start && at <= wall => match change.belongs {
                Belongs::Goal(_) => {
                    tasks.insert(change.task).expect("one key per goal-branch task");
                }
                Belongs::Chat(_) | Belongs::Nothing => {}
            },
            Where::Landed { .. } | Where::Queued { .. } => {}
        }
    }
    tasks.len()
}

fn landed_this_week(rows: &List<Landing>, start: Wall, wall: Wall) -> List<Landed> {
    let mut recent = Map::with_capacity(rows.len());
    for row in rows {
        if row.at >= start && row.at <= wall {
            recent.insert((Reverse(row.at), row.task), row.clone()).expect("one key per landing");
        }
    }
    let mut tasks = Set::with_capacity(rows.len());
    let mut goals = Set::with_capacity(rows.len());
    let mut landed = List::with_capacity(rows.len());
    for (_, row) in &recent {
        if !tasks.insert(row.task).expect("one key per landing task") {
            continue;
        }
        let item = match row.belongs {
            Belongs::Goal(goal) => {
                if !goals.insert(goal).expect("no more goals than landings") {
                    continue;
                }
                Item::Goal(goal)
            }
            Belongs::Chat(_) | Belongs::Nothing => Item::Change(row.task),
        };
        landed
            .push(Landed { item, change: row.task, commit: row.commit, at: row.at, signed: row.signed.clone() })
            .expect("no more rows than landings");
    }
    landed
}

fn newer(left: Wall, left_task: u64, right: Wall, right_task: u64) -> bool {
    left > right || (left == right && left_task < right_task)
}

fn insert_card(facts: &Facts, cards: &mut List<ActiveCard>, mut card: ActiveCard) {
    for next in cards.iter_mut() {
        if card_before(facts, &card, next) {
            core::mem::swap(next, &mut card);
        }
    }
    cards.push(card).expect("one card per listed item");
}

fn card_before(facts: &Facts, left: &ActiveCard, right: &ActiveCard) -> bool {
    if left.activity != right.activity {
        return left.activity > right.activity;
    }
    match (landed_card(facts, left.card.item), landed_card(facts, right.card.item)) {
        (Some(left), Some(right)) => {
            (left.forge, &left.owner, &left.name, &left.path) < (right.forge, &right.owner, &right.name, &right.path)
        }
        (Some(_), None) => false,
        (None, Some(_)) => true,
        (None, None) => left.task < right.task,
    }
}
fn landed_card(facts: &Facts, item: Item) -> Option<&LandedDraft> {
    match item {
        Item::Landed(index) => facts.landed.get(index),
        Item::Goal(_) | Item::Change(_) | Item::Chat(_) | Item::Draft(_) => None,
    }
}
fn ordered_changes(changes: &List<Change>) -> List<Change> {
    let mut seen = Set::with_capacity(changes.len());
    let mut ordered = List::with_capacity(changes.len());
    for change in changes {
        if !seen.insert(change.task).expect("one key per change row") {
            continue;
        }
        let mut change = change.clone();
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
            Where::Landed { at: _, default_branch: _ } => true,
        },
        Where::Landed { at: left_at, default_branch: _ } => match right.at {
            Where::Queued { place: _, since: _ } => false,
            Where::Landed { at: right_at, default_branch: _ } => newer(left_at, left.task, right_at, right.task),
        },
    }
}

fn belonging(changes: &List<Change>, owner: Belongs) -> List<Row> {
    let mut held = List::with_capacity(changes.len());
    for change in changes {
        if change.belongs == owner {
            held.push(Row { task: change.task, signed: change.signed.clone() })
                .expect("no more owned changes than changes");
        }
    }
    held
}

fn has_queued(changes: &List<Change>, owner: Belongs) -> bool {
    for change in changes {
        if change.belongs == owner {
            match change.at {
                Where::Queued { place: _, since: _ } => return true,
                Where::Landed { at: _, default_branch: _ } => {}
            }
        }
    }
    false
}

/// Kind 7's read Hidden switches override Work's filters (client/pages/README.md).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Overrides {
    pub goals: bool,
    pub changes: bool,
}
#[expect(clippy::derivable_impls, reason = "the strict subset forbids deriving Default")]
impl Default for Overrides {
    fn default() -> Self {
        Self { goals: false, changes: false }
    }
}

/// The nonempty filters that no override hides; the view also keeps the current one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[expect(clippy::struct_excessive_bools, reason = "one switch per filter")]
pub struct Shown {
    pub drafts: bool,
    pub goals: bool,
    pub changes: bool,
    pub chats: bool,
}

/// A filter is drawn while it holds something and no override hides it.
#[must_use]
pub fn shown(filters: &Filters, overrides: Overrides) -> Shown {
    Shown {
        drafts: filters.drafts > 0,
        goals: filters.goals > 0 && !overrides.goals,
        changes: filters.changes > 0 && !overrides.changes,
        chats: filters.chats > 0,
    }
}

/// Neither switch hides a filter until Kind 7 is read.
#[must_use]
pub fn overrides(root: &crate::Domain, project: u32) -> Overrides {
    match root.forge().ways(project) {
        Some(ways) => Overrides { goals: !ways.goals, changes: !ways.changes },
        None => Overrides::default(),
    }
}

/// Name a landing's signers, reader first, leaving words out until people are read.
#[must_use]
pub fn signed(reader: u64, parties: &[u64], people: Option<&c::Project>) -> Option<Box<[u8]>> {
    use super::signing::{count, join};
    let people = people?;
    let mut names: List<Box<[u8]>> = List::uncapped();
    if parties.contains(&reader) {
        names.push(b"you".as_slice().into()).expect("one name per signer");
    }
    let mut others = 0_u32;
    for party in parties {
        if *party == reader {
            continue;
        }
        let mut name = None;
        for member in &people.members {
            if member.party == *party {
                name = Some(if member.name.bytes().is_empty() { member.login.bytes() } else { member.name.bytes() });
                break;
            }
        }
        match name {
            Some(name) => names.push(name.into()).expect("one name per signer"),
            None => others = others.saturating_add(1),
        }
    }
    if others > 0 {
        names
            .push(join(&[&count(others), if others == 1 { b" other" } else { b" others" }]))
            .expect("one item for unlisted parties");
    }
    if names.is_empty() {
        return None;
    }
    let mut words = Box::from(b"signed by ".as_slice());
    for (index, name) in names.iter().enumerate() {
        let separator: &[u8] = if index == 0 {
            b""
        } else if index.checked_add(1).expect("name index fits") == usize::try_from(names.len()).expect("names fit") {
            b" and "
        } else {
            b", "
        };
        words = join(&[&words, separator, name]);
    }
    Some(words)
}

fn row_signed(reader: u64, row: &w::ChangesLanded, people: Option<&c::Project>) -> Option<Box<[u8]>> {
    let mut parties = List::with_capacity(row.signed().len());
    for signer in row.signed() {
        parties.push(signer.party()).expect("one party per sign-off");
    }
    signed(reader, parties.as_slice(), people)
}

/// Gather the four lists only once each has answered (client/pages/README.md, Work).
#[must_use]
#[expect(clippy::too_many_lines, reason = "Work joins its four lists and each landing's signers in one facts read")]
pub fn facts(root: &crate::Domain, project: u32) -> Option<Facts> {
    use super::changes::State;
    let goals = root.pages().goals()?;
    let changes = root.pages().changes()?;
    let chats = root.pages().chats()?;
    let drafts = root.pages().drafts()?;
    if root.pages().project() != Some(project)
        || changes.project != project
        || !goals.known(root.client())
        || !chats.known(root.client())
        || !chats.documents_known()
    {
        return None;
    }
    let panel = match changes.state() {
        State::Loading => return None,
        State::Shown(panel) => Some(panel),
        State::Ended => None,
    };
    let mut facts = Facts::default();
    let people = root.pages.team.document(project);
    let reader = root.pages().me(root.client())?.party.number;
    match drafts.state() {
        super::drafts::State::Loading => return None,
        super::drafts::State::Shown(panel) => {
            for row in panel.drafts() {
                facts.landed.push(LandedDraft::row(row)).expect("one fact per landed draft");
            }
        }
        super::drafts::State::Ended => {}
    }
    for line in &goals.live(root.client())?.rows {
        facts
            .goals
            .push(Goal { task: line.chip.task, live: !super::chats::ended(&line.chip), activity: line.last_activity })
            .expect("one fact per goal row");
    }
    for line in goals.ended() {
        facts
            .goals
            .push(Goal { task: line.chip.task, live: false, activity: line.last_activity })
            .expect("one fact per goal row");
    }
    if let Some(panel) = panel {
        for queue in panel.queues() {
            if let Some(flight) = queue.in_flight() {
                facts
                    .changes
                    .push(Change {
                        signed: None,
                        task: flight.member(),
                        at: Where::Queued { place: 1, since: Wall::from_nanos(flight.ready_since()) },
                        belongs: belongs(root, flight.member()),
                    })
                    .expect("one fact per train member");
            }
            for (index, row) in queue.changes().iter().enumerate() {
                facts
                    .changes
                    .push(Change {
                        task: row.task(),
                        signed: None,
                        at: Where::Queued {
                            place: u32::try_from(index)
                                .expect("wire queue fits")
                                .checked_add(1)
                                .expect("wire queue fits")
                                .checked_add(u32::from(queue.in_flight().is_some()))
                                .expect("wire queue fits"),
                            since: Wall::from_nanos(row.ready_since()),
                        },
                        belongs: belongs(root, row.task()),
                    })
                    .expect("one fact per change row");
            }
        }
        for row in panel.landed() {
            facts
                .changes
                .push(Change {
                    task: row.task(),
                    at: Where::Landed { at: Wall::from_nanos(row.at()), default_branch: row.default_branch() },
                    belongs: belongs(root, row.task()),
                    signed: row_signed(reader, row, people),
                })
                .expect("one fact per change row");
        }
        for row in panel.landed_default() {
            facts
                .landed_default
                .push(Landing {
                    task: row.task(),
                    commit: *row.commit(),
                    at: Wall::from_nanos(row.at()),
                    belongs: belongs(root, row.task()),
                    signed: row_signed(reader, row, people),
                })
                .expect("one fact per default-branch landing");
        }
    }
    for line in &chats.live(root.client())?.rows {
        gather_chat(&mut facts, chats, line);
    }
    if let Some(page) = &chats.page {
        for line in &page.live {
            gather_chat(&mut facts, chats, line);
        }
        match &page.ended {
            jig_client::contract::EndedGroup::Rows { rows, .. } => {
                for line in rows {
                    gather_chat(&mut facts, chats, line);
                }
            }
            jig_client::contract::EndedGroup::None => {}
        }
    }
    Some(facts)
}

fn gather_chat(facts: &mut Facts, chats: &super::chats::Chats, line: &jig_client::contract::TaskLine) {
    for chat in &facts.chats {
        if chat.task == line.chip.task {
            return;
        }
    }
    facts
        .chats
        .push(Chat {
            task: line.chip.task,
            activity: line.last_activity,
            draft: chats.documents.contains_key(&line.chip.task),
        })
        .expect("one fact per chat row");
}

fn belongs(root: &crate::Domain, task: u64) -> Belongs {
    use super::changes::From;
    let Some(page) = root.pages().changes() else { return Belongs::Nothing };
    if let Some(goal) = page.goal(task) {
        return Belongs::Goal(goal);
    }
    match page.from(root.client(), task) {
        Some(From::Chat(task)) => Belongs::Chat(task),
        Some(From::Goal(_)) | None => Belongs::Nothing,
    }
}

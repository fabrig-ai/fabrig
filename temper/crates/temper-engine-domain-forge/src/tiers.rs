//! Tiers follow their configured branches (temper/docs/design/domain/after-landing.md, 2, 3, 4 and 10).
use crate::{Domain, Limits, Name, Request, Stored, Tier, TierBranch, TierFlight, TierLanding, TierRow, What};
use alloc::boxed::Box;
use core::mem::size_of;
use skein_forge::api::{self, Commit, Repository, Write};
use skein_lib::{Env, List, Map, Queue, Time, Token, Wall};
use temper_engine_domain_forge_landing as landing;
use temper_engine_domain_forge_tiers as p;
mod parts;
pub(crate) mod words;
use parts::{
    candidate, dispatch_operation, part_branch, part_inputs, part_number, probe_branch, report_read, reported,
};

#[derive(Debug)]
pub(crate) struct State {
    pub(crate) dirty: bool,
    pub(crate) due: Map<u64, Wall>,
    pub(crate) tip_keys: Map<(Repository, Box<[u8]>), p::Judged>,
    walks: Map<(Repository, Box<[u8]>), Walk>,
    pub(crate) reads: Map<Token, (Repository, Box<[u8]>)>,
    pub(crate) follows: Map<(Repository, Box<[u8]>), crate::runs::Follow>,
}
impl State {
    pub(crate) fn new(l: &Limits) -> Self {
        let count = l.repositories.checked_mul(l.tiers.tiers).expect("checked tier count");
        Self {
            dirty: false,
            due: Map::with_capacity(count),
            tip_keys: Map::with_capacity(count),
            walks: Map::with_capacity(count),
            reads: Map::with_capacity(count.checked_mul(2).expect("tier reads")),
            follows: Map::with_capacity(count),
        }
    }
}
#[derive(Debug)]
struct Walk {
    owner: Token,
    commit: Commit,
    object: Commit,
    page: u32,
    trees: List<landing::Tree>,
    entries: List<landing::Entry>,
    next: Option<Time>,
}

pub(crate) fn configured_all(d: &Domain, repository: Repository) -> &[Tier] {
    let Some(adopted) = d.repositories.get(&repository) else { return &[] };
    if let Some((_, repositories, _)) = d.landing.projects.get(&adopted.project) {
        for row in repositories {
            if row.repository == repository {
                return &row.tiers;
            }
        }
    }
    &[]
}
fn configured(d: &Domain, row: &TierRow) -> Option<Tier> {
    for tier in configured_all(d, row.repository) {
        if tier.name == row.tier {
            return Some(tier.clone());
        }
    }
    None
}
fn followed(d: &Domain, repository: Repository, tier: &Tier) -> Option<Box<[u8]>> {
    match &tier.branch {
        Some(branch) => Some(branch.clone()),
        None => Some(d.repositories.get(&repository)?.settings.default_branch.clone()),
    }
}
fn path_parts(bytes: &[u8]) -> Box<[Box<[u8]>]> {
    let mut parts =
        List::with_capacity(u32::try_from(bytes.len()).expect("bounded branch").checked_add(1).expect("branch parts"));
    let mut from = 0;
    for at in 0..=bytes.len() {
        if at == bytes.len() || bytes.get(at) == Some(&b'/') {
            parts.push(Box::from(bytes.get(from..at).expect("branch part"))).expect("one part per byte");
            from = at.checked_add(1).expect("branch cursor");
        }
    }
    parts.into_boxed()
}
fn name(repository: Repository, branch: &[u8]) -> Name {
    Name { forge: repository.forge, repository: repository.repository, what: What::Branch(path_parts(branch)) }
}
pub(crate) fn watched(d: &Domain) -> List<Name> {
    let mut names = List::with_capacity(d.tiers.capacity());
    for (repository, _) in &d.repositories {
        if d.forge_ended(repository.forge) {
            continue;
        }
        for tier in configured_all(d, *repository) {
            if let Some(branch) = followed(d, *repository, tier) {
                let name = name(*repository, &branch);
                if !names.as_slice().contains(&name) {
                    names.push(name).expect("one watch per tier");
                }
            }
        }
    }
    names
}
pub(crate) fn add_watches(
    d: &Domain,
    env: &Env<Limits>,
    watches: &mut Map<skein_forge_client::Resource, skein_forge_client::Watch>,
) -> bool {
    for name in &watched(d) {
        let Some(resource) = crate::domain::to_client(name, &env.limits) else { return false };
        if !watches.contains_key(&resource)
            && watches.insert(resource.clone(), skein_forge_client::Watch { resource, participating: false }).is_err()
        {
            return false;
        }
    }
    true
}
pub(crate) fn watch(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    d.tier_ops.dirty = false;
    let mut watches = Map::with_capacity(env.limits.client.resources);
    for (_, names) in &d.names {
        for name in names {
            if !d.forge_ended(name.forge)
                && let Some(resource) = crate::domain::to_client(name, &env.limits)
            {
                let participating = crate::domain::participating(d, &resource);
                watches
                    .insert(resource.clone(), skein_forge_client::Watch { resource, participating })
                    .expect("admitted watch set");
            }
        }
    }
    assert!(add_watches(d, env, &mut watches), "configured tier watches fit client resources");
    let mut list = List::with_capacity(watches.len());
    for (_, watch) in &watches {
        list.push(watch.clone()).expect("counted watches");
    }
    crate::domain::child(
        d,
        env,
        skein_forge_client::Event::Keep { owner: Token::new(1_u64 << 60), watches: list.into_boxed() },
        out,
    );
}

/// The tier definition includes the optional branch, so a changed following rule changes its key.
pub(crate) fn definition(tier: &Tier) -> Box<[u8]> {
    let mut length = tier
        .workflow
        .len()
        .checked_add(tier.artifact.len())
        .expect("tier definition")
        .checked_add(8)
        .expect("definition headers");
    for path in &tier.ignored {
        length = length.checked_add(2).expect("path header").checked_add(path.len()).expect("ignored path");
    }
    if let Some(branch) = &tier.branch {
        length = length.checked_add(2).expect("branch header").checked_add(branch.len()).expect("branch definition");
    }
    let mut out = skein_lib::Writer::new(length);
    word(&mut out, tier.workflow.len());
    out.put(&tier.workflow).expect("workflow");
    word(&mut out, tier.ignored.len());
    for path in &tier.ignored {
        word(&mut out, path.len());
        out.put(path).expect("ignored path");
    }
    word(&mut out, tier.artifact.len());
    out.put(&tier.artifact).expect("artifact");
    out.put(&[match tier.format {
        p::Format::Junit => 0,
        p::Format::Lines => 1,
    }])
    .expect("format");
    out.put(&[u8::from(tier.branch.is_some())]).expect("branch presence");
    if let Some(branch) = &tier.branch {
        word(&mut out, branch.len());
        out.put(branch).expect("branch");
    }
    out.finish()
}
fn word(out: &mut skein_lib::Writer, value: usize) {
    out.put(&u16::try_from(value).expect("bounded definition").to_be_bytes()).expect("word");
}

fn empty(repository: Repository, tier: Box<[u8]>) -> TierRow {
    TierRow {
        observations: Box::new([]),
        repository,
        tier,
        record: p::Record {
            sequence: 0,
            judged: None,
            failing: p::Failures::Tests(Box::new([])),
            flakes: Box::new([]),
            run: None,
        },
        task: None,
        requested: false,
        cancelled: false,
        purpose: None,
        pending: Box::new([]),
        baseline: None,
        branch: TierBranch::Absent,
        probe: None,
        flight: None,
        held: None,
        result: None,
        received: false,
    }
}
fn save(row: &TierRow, out: &mut Queue<Request>) {
    out.push(Request::Save { read_afresh: None, release: None, record: Stored::Tier(row.clone()) });
}
fn settings(d: &Domain, repository: Repository) -> p::Settings {
    d.after_landing(d.repositories.get(&repository).expect("adopted tier").project)
        .expect("configured tier settings")
        .verdict
}
fn receipt_key(d: &Domain, row: &TierRow, key: Commit) -> crate::ReceiptKey {
    crate::ReceiptKey {
        project: d.repositories.get(&row.repository).expect("adopted tier").project,
        repository: row.repository,
        check: crate::Check::Tier(row.tier.clone()),
        key,
    }
}
fn cached(receipt: &crate::Receipt) -> p::Receipt {
    if receipt.passed {
        p::Receipt::Passed
    } else {
        p::Receipt::Failed(receipt.failures.clone().unwrap_or(p::Failures::Many))
    }
}

pub(crate) fn start_due(d: &mut Domain, env: &Env<Limits>, repository: Repository, out: &mut Queue<Request>) {
    let mut names = List::with_capacity(env.limits.tiers.tiers);
    for tier in configured_all(d, repository) {
        names.push(tier.clone()).expect("configured tiers");
    }
    for tier in names.into_boxed() {
        let Some(branch) = followed(d, repository, &tier) else { continue };
        let Some(head) = d.heads.get(&name(repository, &branch)) else { continue };
        let tip = head.commit;
        let key = (repository, tier.name.clone());
        let mut row = match d.tiers.remove(&key) {
            Some(row) => row,
            None => empty(repository, tier.name),
        };
        if row.record.judged.is_none() && row.record.run.is_none() && row.baseline.is_none() {
            row.baseline = Some(TierLanding { commit: tip, key: None });
        }
        let mut ready = true;
        for landing in &row.pending {
            if landing.key.is_none() {
                ready = false;
            }
        }
        let target = match row.baseline {
            Some(baseline) => Some(baseline),
            None if ready => row.pending.last().copied(),
            None => None,
        };
        if let Some(target) = target
            && let Some(tip_key) = target.key
            && p::due(&row.record, target.commit)
        {
            let mut range = List::with_capacity(u32::try_from(row.pending.len()).expect("range bound"));
            if row.baseline.take().is_none() {
                for landing in &row.pending {
                    range
                        .push(p::Landing { commit: landing.commit, key: landing.key.expect("keyed range") })
                        .expect("range");
                }
                row.pending = Box::new([]);
            }
            let range = range.into_boxed();
            let next = p::start(&row.record, target.commit, tip_key, range, None, &settings(d, repository));
            let p::Next::Dispatch(run) = next else { unreachable!("a run without a receipt dispatches") };
            row.result = d.landing.receipts.get(&receipt_key(d, &row, tip_key)).cloned();
            row.received = row.result.is_some();
            row.record.run = Some(run);
            row.requested = false;
            row.cancelled = false;
            row.purpose = None;
            row.task = None;
            row.branch = TierBranch::Absent;
            row.probe = None;
            row.flight = None;
            row.held = None;
        }
        if !row.requested
            && let Some(run) = &row.record.run
        {
            row.requested = true;
            save(&row, out);
            out.push(Request::TierRun { repository, tier: row.tier.clone(), number: run.number, tip: run.tip });
        } else {
            save(&row, out);
        }
        d.tiers.insert(key.clone(), row).expect("bounded tiers");
        walk(d, env, &key, out);
    }
}
fn walk(d: &mut Domain, env: &Env<Limits>, key: &(Repository, Box<[u8]>), out: &mut Queue<Request>) {
    if d.tier_ops.walks.contains_key(key) || d.stopping {
        return;
    }
    let row = d.tiers.get(key).expect("tier to key");
    let mut commit = None;
    if let Some(baseline) = row.baseline
        && baseline.key.is_none()
    {
        commit = Some(baseline.commit);
    }
    if commit.is_none() {
        for landing in &row.pending {
            if landing.key.is_none() {
                commit = Some(landing.commit);
                break;
            }
        }
    }
    if let Some(commit) = commit {
        d.sequence = d.sequence.checked_add(1).expect("tier tree read sequence");
        let walk = Walk {
            owner: Token::new(d.sequence | (1_u64 << 59)),
            commit,
            object: commit,
            page: 1,
            trees: List::with_capacity(env.limits.trees_in_flight),
            entries: List::with_capacity(env.limits.key_entries),
            next: None,
        };
        ask_tree(d, env, key.clone(), walk, out);
    }
}

fn ask_tree(d: &mut Domain, env: &Env<Limits>, key: (Repository, Box<[u8]>), walk: Walk, out: &mut Queue<Request>) {
    let read = api::Read::Tree { object: walk.object, page: walk.page };
    let owner = walk.owner;
    let repository = key.0;
    d.tier_ops.reads.insert(owner, key.clone()).expect("walk and follower per tier");
    d.tier_ops.walks.insert(key, walk).expect("one walk per tier");
    crate::domain::child(d, env, skein_forge_client::Event::Read { owner, repository, read }, out);
}
fn tree_answer(
    d: &mut Domain,
    env: &Env<Limits>,
    key: (Repository, Box<[u8]>),
    answer: Result<api::Answer, api::Error>,
    out: &mut Queue<Request>,
) {
    let mut walk = d.tier_ops.walks.remove(&key).expect("tree walk");
    if d.stopping {
        return;
    }
    if answer == Err(api::Error::Busy) {
        walk.next = Some(env.now.saturating_add(env.limits.client.poll));
        d.tier_ops.walks.insert(key, walk).expect("replaces walk");
        return;
    }
    let mut keyed = landing::Keyed::TooLarge;
    if let Ok(api::Answer::Tree { entries, more }) = answer {
        let mut fits = true;
        for entry in entries {
            if walk
                .entries
                .push(landing::Entry {
                    name: entry.name,
                    mode: entry.mode,
                    object: entry.object,
                    kind: match entry.kind {
                        api::TreeEntryKind::Blob => landing::EntryKind::Blob,
                        api::TreeEntryKind::Tree => landing::EntryKind::Tree,
                        api::TreeEntryKind::Commit => landing::EntryKind::Commit,
                    },
                })
                .is_err()
            {
                fits = false;
                break;
            }
        }
        if fits && more {
            if let Some(page) = walk.page.checked_add(1) {
                walk.page = page;
                ask_tree(d, env, key, walk, out);
                return;
            }
            fits = false;
        }
        if fits {
            let entries = core::mem::replace(&mut walk.entries, List::with_capacity(env.limits.key_entries));
            if walk.trees.push(landing::Tree { object: walk.object, entries: entries.into_boxed() }).is_ok() {
                let row = d.tiers.get(&key).expect("walk tier");
                let tier = configured(d, row).expect("configured walk");
                keyed = landing::key(
                    landing::Repository { forge: key.0.forge, repository: key.0.repository },
                    &tier.name,
                    &definition(&tier),
                    &tier.ignored,
                    walk.commit,
                    walk.trees.as_slice(),
                    &landing::KeyLimits { entries: env.limits.key_entries, path_bytes: env.limits.key_path_bytes },
                );
            }
        }
    }
    let resolved = match keyed {
        landing::Keyed::Need(object) if walk.trees.len() < env.limits.trees_in_flight => {
            walk.object = object;
            walk.page = 1;
            ask_tree(d, env, key, walk, out);
            return;
        }
        landing::Keyed::Key(key) => key,
        landing::Keyed::Need(_) | landing::Keyed::TooLarge => walk.commit,
    };
    let Some(row) = d.tiers.get_mut(&key) else { return };
    if let Some(baseline) = &mut row.baseline
        && baseline.commit == walk.commit
    {
        baseline.key = Some(resolved);
    }
    for landing in &mut row.pending {
        if landing.commit == walk.commit {
            landing.key = Some(resolved);
        }
    }
    save(row, out);
    remember_tip(d, &key);
    start_due(d, env, key.0, out);
}

pub(crate) fn observed(
    d: &mut Domain,
    env: &Env<Limits>,
    repository: Repository,
    branch: &[u8],
    before: Option<Commit>,
    after: Commit,
    out: &mut Queue<Request>,
) {
    let mut names = List::with_capacity(env.limits.tiers.tiers);
    for tier in configured_all(d, repository) {
        if followed(d, repository, tier).as_deref() == Some(branch) {
            names.push(tier.name.clone()).expect("configured tiers");
        }
    }
    for tier in names.into_boxed() {
        let key = (repository, tier.clone());
        let mut row = match d.tiers.remove(&key) {
            Some(row) => row,
            None => empty(repository, tier),
        };
        if row.record.judged.is_none() && row.record.run.is_none() && row.baseline.is_none() {
            row.baseline = Some(TierLanding { commit: before.unwrap_or(after), key: None });
        }
        if before.is_some() && before != Some(after) {
            append(d, env, &mut row, after);
        }
        save(&row, out);
        d.tiers.insert(key.clone(), row).expect("bounded tiers");
        remember_tip(d, &key);
    }
    start_due(d, env, repository, out);
}
fn own(d: &Domain, repository: Repository, commit: Commit) -> bool {
    if d.verifications.contains_key(&(repository, commit)) {
        return true;
    }
    if let Some(task) = d.landed.get(&commit)
        && let Some(change) = d.changes.get(task)
    {
        return change.repository == repository;
    }
    false
}
fn append(d: &Domain, env: &Env<Limits>, row: &mut TierRow, commit: Commit) {
    if match row.pending.last() {
        Some(landing) => Some(landing.commit),
        None => None,
    } == Some(commit)
        || match &row.record.run {
            Some(run) => Some(run.tip),
            None => None,
        } == Some(commit)
        || match row.record.judged {
            Some(judged) => Some(judged.tip),
            None => None,
        } == Some(commit)
        || match row.baseline {
            Some(baseline) => Some(baseline.commit),
            None => None,
        } == Some(commit)
    {
        return;
    }
    let bound = env.limits.changes.checked_mul(2).expect("twice changes").checked_add(1).expect("range bound");
    let mut pending = List::with_capacity(bound.checked_add(1).expect("incoming landing"));
    for landing in &row.pending {
        pending.push(*landing).expect("existing range");
    }
    pending.push(TierLanding { commit, key: None }).expect("incoming landing");
    let mut skip = None;
    if pending.len() > bound {
        for (index, pair) in pending.as_slice().windows(2).enumerate() {
            let [earlier, later] = pair else { unreachable!("adjacent pair") };
            if !own(d, row.repository, earlier.commit) && !own(d, row.repository, later.commit) {
                skip = Some(index);
                break;
            }
        }
        assert!(skip.is_some(), "changes bound leaves adjacent outside landings");
    }
    let mut compacted = List::with_capacity(bound);
    for (index, landing) in pending.into_boxed().into_iter().enumerate() {
        if skip != Some(index) {
            compacted.push(landing).expect("compacted range");
        }
    }
    row.pending = compacted.into_boxed();
}
pub(crate) fn task(
    d: &mut Domain,
    repository: Repository,
    tier: Box<[u8]>,
    number: u64,
    task: Option<u64>,
    out: &mut Queue<Request>,
) {
    let Some(row) = d.tiers.get_mut(&(repository, tier)) else { return };
    if row.task.is_none()
        && let Some(run) = &row.record.run
        && run.number == number
    {
        row.task = task;
        row.requested = task.is_some();
        save(row, out);
    }
}
fn branch(tier: &[u8], number: u64) -> Box<[u8]> {
    joined(&[b"temper/tier/", tier, b".", skein_lib::Decimal::of(number).as_bytes()])
}
fn joined(parts: &[&[u8]]) -> Box<[u8]> {
    let mut length = 0_u32;
    for part in parts {
        length = length.checked_add(u32::try_from(part.len()).expect("bounded words")).expect("word length");
    }
    let mut bytes = List::with_capacity(length);
    for part in parts {
        for byte in *part {
            bytes.push(*byte).expect("measured words");
        }
    }
    bytes.into_boxed()
}
fn purpose(d: &Domain, row: &TierRow, operation: u8, part: u16) -> crate::purpose::EffectPurpose {
    let mut tier = None;
    for (at, item) in configured_all(d, row.repository).iter().enumerate() {
        if item.name == row.tier {
            tier = Some(u16::try_from(at).expect("tier index"));
        }
    }
    crate::purpose::EffectPurpose::Tier {
        repository: row.repository,
        tier: tier.expect("configured tier"),
        run: row.record.run.as_ref().expect("live run").number,
        operation,
        part,
    }
}
fn following(d: &Domain, row: &TierRow) -> crate::runs::Follow {
    let run = row.record.run.as_ref().expect("live run");
    let (commit, key, tests) = candidate(run);
    let mut follow = crate::runs::follow(
        row.task.expect("tier task"),
        0,
        purpose(d, row, dispatch_operation(&run.part), part_number(&run.part)).identity().expect("tier purpose"),
        row.repository,
        configured(d, row).expect("configured tier").workflow,
        part_branch(row),
        commit,
        part_inputs(commit, key, tests),
    );
    if let Some(flight) = &row.flight {
        follow.entry = flight.entry;
        follow.start = flight.start;
        follow.since = flight.since;
        follow.waiting = flight.waiting;
        if let Some(id) = flight.run {
            follow.run = Some(api::Run {
                id,
                number: 0,
                workflow: follow.workflow.clone(),
                commit,
                event: api::Event::Dispatch,
                actor: d.writers.get(&row.repository.forge).copied().unwrap_or(0),
                state: api::RunState::Running,
                conclusion: None,
                url: Box::new([]),
                created: flight.start.unwrap_or(Time::ZERO),
                jobs: Box::new([]),
            });
        }
        follow.writes = Box::new([]);
        let mut writes = List::with_capacity(u32::try_from(flight.pending.len()).expect("pending jobs"));
        for attempt in &flight.pending {
            writes.push(Write::Rerun { attempt: *attempt }).expect("pending jobs");
        }
        follow.writes = writes.into_boxed();
        // An empty, unsent flight is a fresh dispatch; a sent one is followed.
        if flight.start.is_none() && !flight.waiting && flight.pending.is_empty() && flight.fault.is_none() {
            follow.writes = Box::new([Write::Dispatch {
                workflow: follow.workflow.clone(),
                branch: follow.branch.clone(),
                inputs: follow.inputs.clone(),
            }]);
        }
        if let Some(report) = &flight.report {
            let run = follow.run.as_mut().expect("failed workflow identity");
            run.number = report.number;
            run.url.clone_from(&report.url);
            run.state = api::RunState::Completed;
            run.conclusion = Some(api::Conclusion::Failure);
            follow.outcome = Some(landing::Verdict::Failed);
        }
        if let Some(fault) = flight.fault {
            follow.outcome = Some(landing::Verdict::Fault(fault));
        }
    }
    follow
}
#[expect(clippy::manual_map, reason = "step code uses explicit options without closures")]
fn remember(row: &mut TierRow, follow: &crate::runs::Follow) {
    let mut pending = List::with_capacity(u32::try_from(follow.writes.len()).expect("pending writes"));
    for write in &follow.writes {
        if let Write::Rerun { attempt } = write {
            pending.push(*attempt).expect("pending jobs");
        }
    }
    row.flight = Some(TierFlight {
        entry: follow.entry,
        start: follow.start,
        since: follow.since,
        waiting: follow.waiting,
        next: match &row.flight {
            Some(flight) => flight.next,
            None => None,
        },
        run: match &follow.run {
            Some(run) => Some(run.id),
            None => None,
        },
        pending: pending.into_boxed(),
        report: match follow.outcome {
            Some(landing::Verdict::Failed) => {
                let run = follow.run.as_ref().expect("failed workflow");
                Some(crate::ReceiptRun { id: run.id, number: run.number, url: run.url.clone() })
            }
            Some(landing::Verdict::Fault(_) | landing::Verdict::Passed | landing::Verdict::Running) | None => None,
        },
        fault: match follow.outcome {
            Some(landing::Verdict::Fault(fault)) => Some(fault),
            Some(landing::Verdict::Failed | landing::Verdict::Passed | landing::Verdict::Running) | None => None,
        },
    });
}
fn fault_words(fault: landing::Fault) -> &'static [u8] {
    match fault {
        landing::Fault::SetUp => b"set-up",
        landing::Fault::Completion => b"completion",
        landing::Fault::NoStep => b"no step",
        landing::Fault::Cancelled => b"cancelled",
        landing::Fault::OtherCommit => b"other commit",
        landing::Fault::Lost => b"lost",
    }
}
fn observed_part(
    d: &mut Domain,
    env: &Env<Limits>,
    row: &mut TierRow,
    follow: &mut crate::runs::Follow,
    out: &mut Queue<Request>,
) {
    let outcome = match follow.outcome {
        Some(landing::Verdict::Passed) => p::Outcome::Passed,
        Some(landing::Verdict::Fault(_)) => p::Outcome::Fault,
        Some(landing::Verdict::Failed | landing::Verdict::Running) | None => return,
    };
    advance(d, env, row, follow, outcome, out);
}
fn advance(
    d: &mut Domain,
    env: &Env<Limits>,
    row: &mut TierRow,
    follow: &mut crate::runs::Follow,
    outcome: p::Outcome,
    out: &mut Queue<Request>,
) {
    words::observe(row, follow);
    let run = row.record.run.as_ref().expect("followed run");
    let observation = observation(env, row, follow, &outcome);
    if run.part == p::Part::Whole
        && let Some(receipt) = &observation
    {
        crate::landing::put_receipt(d, env, receipt_key(d, row, run.key), receipt.clone(), out);
    }
    match p::step(&row.record, run, outcome, &settings(d, row.repository)) {
        p::Next::Dispatch(next) => {
            if next.part == run.part {
                let attempt =
                    u16::try_from(next.faults).expect("configured retry count").checked_add(1).expect("retry attempt");
                row.record.run = Some(next);
                crate::runs::retry(follow, attempt);
                if follow.start.is_none() {
                    follow.since = None;
                }
            } else {
                row.record.run = Some(next);
                row.flight = None;
                *follow = following(d, row);
            }
        }
        p::Next::Held(run) => {
            row.record.run = Some(run);
            row.held = Some(match follow.outcome {
                Some(landing::Verdict::Fault(fault)) => fault,
                Some(landing::Verdict::Failed | landing::Verdict::Passed | landing::Verdict::Running) | None => {
                    landing::Fault::NoStep
                }
            });
        }
        p::Next::Judged(_) => {
            row.result = observation;
            row.received = false;
        }
    }
    remember(row, follow);
    if row.held.is_some()
        && let Some(flight) = &mut row.flight
    {
        flight.next = None;
    }
    save(row, out);
}
fn decision(task: u64, decision: crate::TierDecision, entry: Option<u64>, out: &mut Queue<Request>) {
    out.push(Request::TierDecision { task, decision, projection: None, entry });
}
fn wait(task: u64, out: &mut Queue<Request>) {
    decision(task, crate::TierDecision::Wait { until: None }, None, out);
}
fn held(row: &TierRow, out: &mut Queue<Request>) {
    let run = row.record.run.as_ref().expect("held run");
    let words = joined(&[
        b"Tier ",
        &row.tier,
        b" run ",
        skein_lib::Decimal::of(run.number).as_bytes(),
        b" held after ",
        skein_lib::Decimal::of(u64::from(run.faults)).as_bytes(),
        b" runner faults, last: ",
        fault_words(row.held.unwrap_or(landing::Fault::Lost)),
    ]);
    decision(row.task.expect("tier task"), crate::TierDecision::Held { words }, None, out);
}
fn effect(d: &mut Domain, row: &mut TierRow, entry: u64, write: Write, release: bool, out: &mut Queue<Request>) {
    let operation = match &write {
        Write::CreateBranch { branch: name, .. } => {
            if *name == branch(&row.tier, row.record.run.as_ref().expect("live run").number) { 0 } else { 3 }
        }
        Write::Dispatch { .. } => dispatch_operation(&row.record.run.as_ref().expect("live run").part),
        Write::Rerun { .. } => 2,
        Write::DeleteBranch { .. } => 5,
        Write::CreateIssue { .. }
        | Write::OpenPull { .. }
        | Write::Post { .. }
        | Write::Review { .. }
        | Write::Edit { .. }
        | Write::SetReviewers { .. }
        | Write::Close { .. }
        | Write::Reopen { .. }
        | Write::Merge { .. }
        | Write::Update { .. }
        | Write::Status { .. } => unreachable!("tier run operations"),
    };
    let part = match &write {
        Write::DeleteBranch { branch: name }
            if *name == branch(&row.tier, row.record.run.as_ref().expect("live run").number) =>
        {
            0
        }
        Write::DeleteBranch { .. } => {
            row.probe.as_ref().expect("probe deletion").landing.checked_add(2).expect("probe part")
        }
        Write::CreateBranch { .. }
        | Write::Dispatch { .. }
        | Write::Rerun { .. }
        | Write::CreateIssue { .. }
        | Write::OpenPull { .. }
        | Write::Post { .. }
        | Write::Review { .. }
        | Write::Edit { .. }
        | Write::SetReviewers { .. }
        | Write::Close { .. }
        | Write::Reopen { .. }
        | Write::Merge { .. }
        | Write::Update { .. }
        | Write::Status { .. } => u32::from(part_number(&row.record.run.as_ref().expect("live run").part)),
    };
    row.purpose = Some(purpose(d, row, operation, u16::try_from(part).expect("bounded part")));
    let kept = skein_forge_client::Entry {
        number: entry,
        task: row.task.expect("tier task"),
        repository: row.repository,
        effect: skein_forge_client::Effect { write, condition: skein_forge_client::Condition::None },
        start: None,
        attempt: None,
        failures: 0,
        queued: false,
    };
    save(row, out);
    crate::domain::keep_entry(d, kept, if release { Some(entry) } else { None }, out);
    decision(
        row.task.expect("effect task"),
        crate::TierDecision::Wait { until: None },
        if release { None } else { Some(entry) },
        out,
    );
}
fn finish(d: &mut Domain, env: &Env<Limits>, row: &mut TierRow, out: &mut Queue<Request>) {
    let result = row.result.as_ref().expect("terminal receipt");
    let run = row.record.run.as_ref().expect("live range");
    let next = if row.received {
        p::start(
            &row.record,
            run.tip,
            run.key,
            run.landings.clone(),
            Some(cached(result)),
            &settings(d, row.repository),
        )
    } else {
        p::step(
            &row.record,
            run,
            match cached(result) {
                p::Receipt::Passed => p::Outcome::Passed,
                p::Receipt::Failed(failures) => p::Outcome::Failed(failures),
            },
            &settings(d, row.repository),
        )
    };
    let p::Next::Judged(verdict) = next else { unreachable!("saved final observation judges the committed part") };
    let words = verdict_words(&row.tier, &verdict);
    crate::verification::judged(d, row.repository, &row.tier, &verdict, &run.landings, out);
    row.record = p::judged(
        &row.record,
        &verdict,
        &p::Limits { failing_tests: env.limits.tiers.failing_tests, test_id_bytes: env.limits.tiers.test_id_bytes },
    );
    let task = row.task.take().expect("tier task");
    d.offered_tier(task);
    row.flight = None;
    row.result = None;
    row.held = None;
    row.requested = false;
    row.received = false;
    row.purpose = None;
    row.cancelled = false;
    let projection = Some(core::mem::replace(&mut row.observations, Box::new([])));
    save(row, out);
    out.push(Request::TierDecision { task, decision: crate::TierDecision::Done { words }, entry: None, projection });
}
fn reading(d: &Domain, key: &(Repository, Box<[u8]>)) -> bool {
    for (owner, candidate) in &d.tier_ops.reads {
        if candidate == key
            && match d.tier_ops.walks.get(key) {
                Some(walk) => Some(walk.owner),
                None => None,
            } != Some(*owner)
        {
            return true;
        }
    }
    false
}
fn cleanup_probe(d: &mut Domain, env: &Env<Limits>, row: &mut TierRow, entry: u64, out: &mut Queue<Request>) -> bool {
    let Some(probe) = &row.probe else { return false };
    let run = row.record.run.as_ref().expect("probe run");
    let cleanup = row.result.is_some()
        || row.cancelled
        || match &run.part {
            p::Part::Probe { landing, .. } => *landing != probe.landing,
            p::Part::Whole | p::Part::Rerun { .. } => true,
        };
    if !cleanup {
        return false;
    }
    if probe.branch == TierBranch::Absent {
        row.probe = None;
        save(row, out);
        return false;
    }
    if probe.branch == TierBranch::Made && room(d, env, entry) {
        let write = Write::DeleteBranch { branch: probe_branch(row, probe.landing) };
        row.probe.as_mut().expect("probe").branch = TierBranch::Pending { entry };
        effect(d, row, entry, write, true, out);
    } else {
        wait(row.task.expect("probe task"), out);
    }
    true
}
fn probe_ready(d: &mut Domain, env: &Env<Limits>, row: &mut TierRow, entry: u64, out: &mut Queue<Request>) -> bool {
    let run = row.record.run.as_ref().expect("probe run");
    let p::Part::Probe { landing, .. } = &run.part else { return true };
    if let Some(probe) = &row.probe
        && probe.branch == TierBranch::Absent
    {
        row.probe = None;
    }
    if row.probe.is_none() && room(d, env, entry) {
        let write = Write::CreateBranch { branch: part_branch(row), commit: candidate(run).0 };
        row.probe = Some(crate::TierProbeBranch { landing: *landing, branch: TierBranch::Pending { entry } });
        effect(d, row, entry, write, false, out);
        return false;
    }
    if let Some(probe) = &row.probe
        && probe.branch == TierBranch::Made
    {
        return true;
    }
    wait(row.task.expect("probe task"), out);
    false
}
fn cached_part(d: &Domain, row: &mut TierRow) {
    if !row.received {
        return;
    }
    let run = row.record.run.as_ref().expect("cached run");
    let next = p::start(
        &row.record,
        run.tip,
        run.key,
        run.landings.clone(),
        Some(cached(row.result.as_ref().expect("cached receipt"))),
        &settings(d, row.repository),
    );
    match next {
        p::Next::Dispatch(run) => {
            row.record.run = Some(run);
            row.result = None;
            row.received = false;
            row.flight = None;
        }
        p::Next::Judged(_) => {}
        p::Next::Held(_) => unreachable!("a receipt has no runner fault"),
    }
}
#[expect(clippy::too_many_lines, reason = "one committed tier step chooses its branch, flight or judgement")]
pub(crate) fn step(
    d: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    entry: u64,
    heard: crate::TierHeard,
    out: &mut Queue<Request>,
) {
    let Some(key) = task_key(d, task) else {
        decision(
            task,
            crate::TierDecision::Held { words: Box::from(b"Tier run is unavailable".as_slice()) },
            None,
            out,
        );
        return;
    };
    let mut row = d.tiers.remove(&key).expect("found tier");
    cached_part(d, &mut row);
    if heard.released && row.held.is_some() {
        row.record.run.as_mut().expect("held run").faults = 0;
        row.held = None;
        row.flight = None;
        d.tier_ops.follows.remove(&key);
        save(&row, out);
    }
    if heard.cancelled {
        row.cancelled = true;
        // Cancellation closes the task after its branch is released, with no verdict.
        row.held = None;
    }
    if row.flight.is_some() && !d.tier_ops.follows.contains_key(&key) && row.result.is_none() && row.held.is_none() {
        d.tier_ops.follows.insert(key.clone(), following(d, &row)).expect("one follower per tier");
    }
    if row.result.is_none()
        && row.held.is_none()
        && let Some(mut follow) = d.tier_ops.follows.remove(&key)
    {
        observed_part(d, env, &mut row, &mut follow, out);
        d.tier_ops.follows.insert(key.clone(), follow).expect("bounded follower");
    }
    if row.held.is_some() {
        held(&row, out);
    } else if cleanup_probe(d, env, &mut row, entry, out) {
        // A completed probe releases its branch before the next part starts.
    } else if row.result.is_some() || row.cancelled {
        match row.branch {
            TierBranch::Made => {
                if room(d, env, entry) {
                    let write = Write::DeleteBranch {
                        branch: branch(&row.tier, row.record.run.as_ref().expect("live run").number),
                    };
                    row.branch = TierBranch::Pending { entry };
                    effect(d, &mut row, entry, write, true, out);
                } else {
                    wait(task, out);
                }
            }
            TierBranch::Pending { .. } => wait(task, out),
            TierBranch::Absent => {
                d.tier_ops.follows.remove(&key);
                if row.cancelled {
                    cancelled(d, &mut row, task, out);
                } else {
                    finish(d, env, &mut row, out);
                }
            }
        }
    } else {
        match row.branch {
            TierBranch::Absent if room(d, env, entry) => {
                let run = row.record.run.as_ref().expect("live run");
                let write = Write::CreateBranch { branch: branch(&row.tier, run.number), commit: run.tip };
                row.branch = TierBranch::Pending { entry };
                effect(d, &mut row, entry, write, false, out);
            }
            TierBranch::Made => {
                if !probe_ready(d, env, &mut row, entry, out) {
                    d.tiers.insert(key, row).expect("replaces tier");
                    return;
                }
                let mut follow = match d.tier_ops.follows.remove(&key) {
                    Some(follow) => follow,
                    None => following(d, &row),
                };
                if !follow.waiting && !follow.writes.is_empty() && room(d, env, entry) {
                    let write = follow.write(entry).expect("pending write");
                    let run = row.record.run.as_ref().expect("run");
                    let operation = if let Write::Rerun { .. } = write { 2 } else { dispatch_operation(&run.part) };
                    follow.purpose = purpose(d, &row, operation, part_number(&run.part)).identity().expect("purpose");
                    remember(&mut row, &follow);
                    effect(d, &mut row, entry, write, false, out);
                } else {
                    wait(task, out);
                }
                if !reading(d, &key)
                    && let Some(read) = report_read(d, &row, env).or(follow.read())
                {
                    d.sequence = d.sequence.checked_add(1).expect("tier read sequence");
                    let owner = Token::new(d.sequence | (1_u64 << 59));
                    d.tier_ops.reads.insert(owner, key.clone()).expect("one read per tier");
                    d.tier_ops.follows.insert(key.clone(), follow).expect("bounded follower");
                    d.tiers.insert(key.clone(), row).expect("replaces tier");
                    crate::domain::child(
                        d,
                        env,
                        skein_forge_client::Event::Read { owner, repository: key.0, read },
                        out,
                    );
                    return;
                }
                d.tier_ops.follows.insert(key.clone(), follow).expect("bounded follower");
            }
            TierBranch::Absent | TierBranch::Pending { .. } => wait(task, out),
        }
    }
    let done = row.record.run.is_none();
    d.tiers.insert(key.clone(), row).expect("replaces tier");
    if done {
        start_due(d, env, key.0, out);
    }
}
fn room(d: &Domain, env: &Env<Limits>, entry: u64) -> bool {
    !d.entries.contains_key(&entry) && d.entries.len() < env.limits.entries
}
fn task_key(d: &Domain, task: u64) -> Option<(Repository, Box<[u8]>)> {
    for (key, row) in &d.tiers {
        if row.task == Some(task) {
            return Some(key.clone());
        }
    }
    None
}
#[expect(clippy::match_like_matches_macro, reason = "step code uses explicit matches")]
pub(crate) fn answered(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    answer: Result<api::Answer, api::Error>,
    out: &mut Queue<Request>,
) {
    let Some(key) = d.tier_ops.reads.remove(&owner) else { return };
    if let Some(walk) = d.tier_ops.walks.get(&key)
        && walk.owner == owner
    {
        tree_answer(d, env, key, answer, out);
        return;
    }
    let Some(mut row) = d.tiers.remove(&key) else { return };
    if d.stopping || row.record.run.is_none() {
        d.tiers.insert(key, row).expect("replaces tier");
        return;
    }
    let mut follow = d.tier_ops.follows.remove(&key).expect("read's follower");
    let writer = d.writers.get(&row.repository.forge).copied().unwrap_or(0);
    let listing = if let Ok(api::Answer::Runs { .. }) = &answer { true } else { false };
    let reporting = match &row.flight {
        Some(flight) => flight.report.is_some(),
        None => false,
    };
    let ended = if reporting {
        let outcome = reported(d, env, &row, answer);
        advance(d, env, &mut row, &mut follow, outcome, out);
        true
    } else {
        let ended = follow.answered(answer, writer, env.now, env.limits.client.lifetime).is_some();
        words::observe(&mut row, &follow);
        if row.held.is_none() && row.result.is_none() {
            observed_part(d, env, &mut row, &mut follow, out);
        }
        ended
    };
    remember(&mut row, &follow);
    save(&row, out);
    if ended || (listing && follow.run.is_some()) {
        if let Some(task) = row.task {
            out.push(Request::WakeProcedure { task });
        }
    } else if let Some(flight) = &mut row.flight {
        flight.next = Some(env.now.saturating_add(env.limits.client.poll));
        save(&row, out);
    }
    d.tiers.insert(key.clone(), row).expect("replaces tier");
    d.tier_ops.follows.insert(key, follow).expect("replaces follower");
}
pub(crate) fn hinted(d: &Domain, hint: &api::Hint, out: &mut Queue<Request>) {
    if let api::Change::Commit(commit) = hint.change {
        for (_, follow) in &d.tier_ops.follows {
            if follow.repository == hint.repository && follow.commit == commit {
                out.push(Request::WakeProcedure { task: follow.task });
            }
        }
    }
}
pub(crate) fn deadline(d: &Domain) -> Option<Time> {
    if d.stopping {
        return None;
    }
    let mut next: Option<Time> = None;
    for (_, walk) in &d.tier_ops.walks {
        if let Some(at) = walk.next {
            next = Some(match next {
                Some(old) => old.min(at),
                None => at,
            });
        }
    }
    for (_, row) in &d.tiers {
        if let Some(flight) = &row.flight
            && row.result.is_none()
            && let Some(at) = flight.next
        {
            next = Some(match next {
                Some(old) => old.min(at),
                None => at,
            });
        }
    }
    next
}
pub(crate) fn fire(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let mut tasks = List::with_capacity(d.tier_ops.due.capacity());
    for (task, when) in &d.tier_ops.due {
        if *when <= env.wall {
            tasks.push(*task).expect("one deadline per admitted tier task");
        }
    }
    for task in &tasks {
        d.tier_ops.due.remove(task);
        out.push(Request::WakeProcedure { task: *task });
    }
    let mut due = None;
    for (key, walk) in &d.tier_ops.walks {
        if let Some(at) = walk.next
            && at <= env.now
        {
            due = Some(key.clone());
            break;
        }
    }
    if let Some(key) = due {
        let mut walk = d.tier_ops.walks.remove(&key).expect("due walk");
        walk.next = None;
        ask_tree(d, env, key, walk, out);
        return;
    }
    let mut found = None;
    for (key, row) in &d.tiers {
        if row.result.is_none()
            && let Some(flight) = &row.flight
            && let Some(at) = flight.next
            && at <= env.now
        {
            found = Some(key.clone());
            break;
        }
    }
    if let Some(key) = found {
        let row = d.tiers.get_mut(&key).expect("due tier");
        row.flight.as_mut().expect("due flight").next = None;
        save(row, out);
        if let Some(task) = row.task {
            out.push(Request::WakeProcedure { task });
        }
    }
}
fn branch_write(write: &Write) -> Option<(&[u8], bool)> {
    match write {
        Write::CreateBranch { branch, .. } => Some((branch, false)),
        Write::DeleteBranch { branch } => Some((branch, true)),
        Write::Dispatch { .. }
        | Write::Rerun { .. }
        | Write::CreateIssue { .. }
        | Write::OpenPull { .. }
        | Write::Post { .. }
        | Write::Review { .. }
        | Write::Edit { .. }
        | Write::SetReviewers { .. }
        | Write::Close { .. }
        | Write::Reopen { .. }
        | Write::Merge { .. }
        | Write::Update { .. }
        | Write::Status { .. } => None,
    }
}
pub(crate) fn outcome(d: &mut Domain, entry: u64, outcome: skein_forge_client::Outcome, out: &mut Queue<Request>) {
    use skein_forge_client::Outcome;
    let mut found = None;
    for (key, row) in &d.tiers {
        if row.branch == (TierBranch::Pending { entry })
            || match &row.probe {
                Some(probe) => probe.branch == (TierBranch::Pending { entry }),
                None => false,
            }
            || match &row.flight {
                Some(flight) => Some((flight.entry, flight.waiting)),
                None => None,
            } == Some((entry, true))
        {
            found = Some(key.clone());
            break;
        }
    }
    let Some(key) = found else { return };
    if outcome == Outcome::Queued || outcome == Outcome::Uncertain {
        return;
    }
    let mut row = d.tiers.remove(&key).expect("effect tier");
    let made = match outcome {
        Outcome::Made { .. } => true,
        Outcome::Failed(_)
        | Outcome::Raced { .. }
        | Outcome::Withdrawn
        | Outcome::Held
        | Outcome::Uncertain
        | Outcome::Queued => false,
    };
    let probe_pending = match &row.probe {
        Some(probe) => probe.branch == (TierBranch::Pending { entry }),
        None => false,
    };
    if row.branch == (TierBranch::Pending { entry }) || probe_pending {
        let (named, deleting) =
            branch_write(&d.entries.get(&entry).expect("branch entry").entry.effect.write).expect("branch operation");
        if probe_pending {
            if made && deleting {
                row.probe = None;
            } else {
                row.probe.as_mut().expect("probe").branch =
                    if made || deleting { TierBranch::Made } else { TierBranch::Absent };
            }
        } else {
            row.branch = if made == deleting { TierBranch::Absent } else { TierBranch::Made };
        }
        if made && deleting && d.heads.remove(&name(row.repository, named)).is_some() {
            out.push(Request::Erase { key: crate::Key::BranchHead(name(row.repository, named)) });
        }
        if !made {
            row.held = Some(landing::Fault::Lost);
        }
    } else {
        let mut follow = match d.tier_ops.follows.remove(&key) {
            Some(follow) => follow,
            None => following(d, &row),
        };
        follow.waiting = false;
        if let Some(kept) = d.entries.get(&entry) {
            if let Write::Dispatch { .. } = kept.entry.effect.write
                && let Some(start) = kept.entry.start
            {
                follow.start = Some(start.at);
            }
            if let Some(attempt) = kept.entry.attempt {
                follow.since = Some(attempt.sent);
            }
        }
        if !made {
            follow.outcome = Some(landing::Verdict::Fault(landing::Fault::Lost));
        }
        remember(&mut row, &follow);
        d.tier_ops.follows.insert(key.clone(), follow).expect("bounded follower");
    }
    save(&row, out);
    if let Some(task) = row.task {
        out.push(Request::WakeProcedure { task });
    }
    d.tiers.insert(key, row).expect("replaces tier");
}
pub(crate) fn veto(d: &mut Domain, task: u64, entry: u64, out: &mut Queue<Request>) {
    let Some(key) = task_key(d, task) else { return };
    let Some(kept) = d.entries.get(&entry) else { return };
    if kept.entry.task != task {
        return;
    }
    let deleting = match &kept.entry.effect.write {
        Write::DeleteBranch { .. } => true,
        Write::CreateBranch { .. }
        | Write::Dispatch { .. }
        | Write::Rerun { .. }
        | Write::CreateIssue { .. }
        | Write::OpenPull { .. }
        | Write::Post { .. }
        | Write::Review { .. }
        | Write::Edit { .. }
        | Write::SetReviewers { .. }
        | Write::Close { .. }
        | Write::Reopen { .. }
        | Write::Merge { .. }
        | Write::Update { .. }
        | Write::Status { .. } => false,
    };
    let row = d.tiers.get_mut(&key).expect("task tier");
    if let Some(probe) = &mut row.probe
        && probe.branch == (TierBranch::Pending { entry })
    {
        probe.branch = if deleting { TierBranch::Made } else { TierBranch::Absent };
    }
    if row.branch == (TierBranch::Pending { entry }) {
        row.branch = if deleting { TierBranch::Made } else { TierBranch::Absent };
    }
    if let Some(flight) = &mut row.flight
        && flight.entry == entry
    {
        flight.waiting = false;
        flight.fault = Some(landing::Fault::Lost);
    }
    save(row, out);
    d.tier_ops.follows.remove(&key);
    d.entries.remove(&entry);
    out.push(Request::Erase { key: crate::Key::Entry(entry) });
}
pub(crate) fn restart(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    d.tier_ops.due = Map::with_capacity(d.tier_ops.due.capacity());
    let mut keys = List::with_capacity(d.tiers.capacity());
    for (key, _) in &d.tiers {
        keys.push(key.clone()).expect("bounded tiers");
    }
    for key in keys.into_boxed() {
        remember_tip(d, &key);
        let row = d.tiers.get(&key).expect("restored tier");
        if row.flight.is_some() && row.task.is_some() && row.result.is_none() && row.held.is_none() {
            d.tier_ops.follows.insert(key.clone(), following(d, row)).expect("bounded followers");
        }
        if let Some(task) = row.task {
            out.push(Request::WakeProcedure { task });
        }
        walk(d, env, &key, out);
    }
    let mut repositories = List::with_capacity(d.repositories.len());
    for (repository, _) in &d.repositories {
        repositories.push(*repository).expect("adopted repositories");
    }
    for repository in repositories.into_boxed() {
        start_due(d, env, repository, out);
    }
    watch(d, env, out);
}
impl Domain {
    /// The top owns each tier task's deadline (temper/docs/design/domain/after-landing.md, 10).
    pub fn wait_tier(&mut self, env: &Env<Limits>, task: u64, until: Option<Wall>) {
        let until = match until {
            Some(until) => until,
            None => Wall::from_nanos(env.wall.as_nanos().saturating_add(env.limits.client.poll.as_nanos())),
        };
        self.tier_ops.due.insert(task, until).expect("one deadline per admitted tier task");
    }
    /// The tier task was stepped: its deadline, if any, is spent.
    pub fn offered_tier(&mut self, task: u64) {
        self.tier_ops.due.remove(&task);
    }

    #[must_use]
    #[expect(clippy::manual_find, reason = "step code searches explicitly without callbacks")]
    pub fn tier_task(&self, task: u64) -> Option<&TierRow> {
        for (_, row) in &self.tiers {
            if row.task == Some(task) {
                return Some(row);
            }
        }
        None
    }
    #[must_use]
    pub fn describe_tier_effect(&self, task: u64, entry: u64) -> Option<(Repository, What, crate::AgentEffect)> {
        let row = self.tier_task(task)?;
        let run = row.record.run.as_ref()?;
        let (commit, key, tests) = candidate(run);
        let kept = &self.entries.get(&entry)?.entry;
        if kept.task != task || kept.repository != row.repository {
            return None;
        }
        let (kind, what) = match &kept.effect.write {
            Write::CreateBranch { branch: name, commit }
                if row.branch == (TierBranch::Pending { entry })
                    && *name == branch(&row.tier, run.number)
                    && *commit == run.tip =>
            {
                (9, What::Branch(path_parts(name)))
            }
            Write::CreateBranch { branch: name, commit: head }
                if row.probe.as_ref()?.branch == (TierBranch::Pending { entry })
                    && *name == part_branch(row)
                    && *head == commit =>
            {
                (9, What::Branch(path_parts(name)))
            }
            Write::Dispatch { workflow, branch: name, inputs }
                if row.flight.as_ref()?.entry == entry
                    && row.flight.as_ref()?.waiting
                    && *workflow == configured(self, row)?.workflow
                    && *name == part_branch(row)
                    && *inputs == part_inputs(commit, key, tests) =>
            {
                (16, What::Repository)
            }
            Write::Rerun { attempt }
                if row.flight.as_ref()?.entry == entry && row.flight.as_ref()?.waiting && attempt.head == commit =>
            {
                (17, What::Repository)
            }
            Write::CreateBranch { .. }
            | Write::Dispatch { .. }
            | Write::Rerun { .. }
            | Write::CreateIssue { .. }
            | Write::OpenPull { .. }
            | Write::Post { .. }
            | Write::Review { .. }
            | Write::Edit { .. }
            | Write::SetReviewers { .. }
            | Write::Close { .. }
            | Write::Reopen { .. }
            | Write::Merge { .. }
            | Write::Update { .. }
            | Write::Status { .. }
            | Write::DeleteBranch { .. } => return None,
        };
        let operation = match kind {
            9 => {
                if kept.effect.write == (Write::CreateBranch { branch: branch(&row.tier, run.number), commit: run.tip })
                {
                    0
                } else {
                    3
                }
            }
            16 => dispatch_operation(&run.part),
            17 => 2,
            _ => return None,
        };
        if row.purpose != Some(purpose(self, row, operation, part_number(&run.part))) {
            return None;
        }
        let (form, recovery) = crate::effect_shape(kind)?;
        let state = match &kept.effect.write {
            Write::CreateBranch { commit, .. } => *commit,
            Write::Dispatch { .. } | Write::Rerun { .. } => commit,
            Write::CreateIssue { .. }
            | Write::OpenPull { .. }
            | Write::Post { .. }
            | Write::Review { .. }
            | Write::Edit { .. }
            | Write::SetReviewers { .. }
            | Write::Close { .. }
            | Write::Reopen { .. }
            | Write::Merge { .. }
            | Write::Update { .. }
            | Write::Status { .. }
            | Write::DeleteBranch { .. } => return None,
        };
        Some((row.repository, what, crate::AgentEffect { effect: kept.effect.clone(), kind, state, form, recovery }))
    }
}
pub(crate) fn tests_bytes(tests: &[Box<[u8]>]) -> Option<u64> {
    let mut held = u64::try_from(tests.len()).ok()?.checked_mul(u64::try_from(size_of::<Box<[u8]>>()).ok()?)?;
    for test in tests {
        held = held.checked_add(u64::try_from(test.len()).ok()?)?;
    }
    Some(held)
}
pub(crate) fn failures_bytes(failures: &p::Failures) -> Option<u64> {
    match failures {
        p::Failures::Many => Some(0),
        p::Failures::Tests(tests) => tests_bytes(tests),
    }
}
pub(crate) fn blame_bytes(blame: &p::Blame) -> Option<u64> {
    match blame {
        p::Blame::Cleared => Some(0),
        p::Blame::Owns(f) | p::Blame::Suspect(f) => failures_bytes(f),
    }
}
pub(crate) fn row_bytes(row: &TierRow) -> Option<u64> {
    let mut held = u64::try_from(row.tier.len())
        .ok()?
        .checked_add(u64::try_from(row.observations.len()).ok()?)?
        .checked_add(failures_bytes(&row.record.failing)?)?
        .checked_add(
            u64::try_from(row.record.flakes.len()).ok()?.checked_mul(u64::try_from(size_of::<p::Flake>()).ok()?)?,
        )?;
    for flake in &row.record.flakes {
        held = held.checked_add(u64::try_from(flake.test.len()).ok()?)?;
    }
    held = held.checked_add(
        u64::try_from(row.pending.len()).ok()?.checked_mul(u64::try_from(size_of::<TierLanding>()).ok()?)?,
    )?;
    if let Some(flight) = &row.flight {
        if let Some(report) = &flight.report {
            held = held.checked_add(u64::try_from(report.url.len()).ok()?)?;
        }
        held = held.checked_add(
            u64::try_from(flight.pending.len()).ok()?.checked_mul(u64::try_from(size_of::<api::JobAttempt>()).ok()?)?,
        )?;
    }
    if let Some(result) = &row.result {
        held = held.checked_add(u64::try_from(result.run.url.len()).ok()?)?;
        if let Some(failures) = &result.failures {
            held = held.checked_add(failures_bytes(failures)?)?;
        }
    }
    if let Some(run) = &row.record.run {
        held = held.checked_add(
            u64::try_from(run.landings.len()).ok()?.checked_mul(u64::try_from(size_of::<p::Landing>()).ok()?)?,
        )?;
        match &run.part {
            p::Part::Whole => {}
            p::Part::Rerun { failed } => held = held.checked_add(tests_bytes(failed)?)?,
            p::Part::Probe { tests, finding, .. } => {
                match tests {
                    p::Selection::All => {}
                    p::Selection::Tests(tests) => held = held.checked_add(tests_bytes(tests)?)?,
                }
                held = held
                    .checked_add(failures_bytes(&finding.failing)?)?
                    .checked_add(tests_bytes(&finding.flakes)?)?
                    .checked_add(
                        u64::try_from(finding.blamed.len())
                            .ok()?
                            .checked_mul(u64::try_from(size_of::<p::Blame>()).ok()?)?,
                    )?;
                for blame in &finding.blamed {
                    held = held.checked_add(blame_bytes(blame)?)?;
                }
            }
        }
    }
    Some(held)
}

#[expect(clippy::manual_map, reason = "step code uses explicit options without closures")]
fn remember_tip(d: &mut Domain, key: &(Repository, Box<[u8]>)) {
    let row = d.tiers.get(key).expect("tier tip");
    let tip = match row.pending.last().or(row.baseline.as_ref()) {
        Some(tip) => match tip.key {
            Some(key) => Some(p::Judged { tip: tip.commit, key }),
            None => None,
        },
        None => match &row.record.run {
            Some(run) => Some(p::Judged { tip: run.tip, key: run.key }),
            None => row.record.judged,
        },
    };
    match tip {
        Some(tip) => {
            d.tier_ops.tip_keys.insert(key.clone(), tip).expect("one key per tier");
        }
        None => {
            d.tier_ops.tip_keys.remove(key);
        }
    }
}

fn observation(
    env: &Env<Limits>,
    row: &TierRow,
    follow: &crate::runs::Follow,
    outcome: &p::Outcome,
) -> Option<crate::Receipt> {
    match outcome {
        p::Outcome::Fault => None,
        p::Outcome::Passed | p::Outcome::Failed(_) => {
            let workflow = follow.run.as_ref().expect("completed workflow");
            let failures = match outcome {
                p::Outcome::Passed => p::Failures::Tests(Box::new([])),
                p::Outcome::Failed(failures) => failures.clone(),
                p::Outcome::Fault => unreachable!("completed observation"),
            };
            Some(crate::Receipt {
                passed: *outcome == p::Outcome::Passed,
                failures: Some(failures),
                candidate: candidate(row.record.run.as_ref().expect("followed run")).0,
                ended: env.wall,
                run: crate::ReceiptRun { id: workflow.id, number: workflow.number, url: workflow.url.clone() },
            })
        }
    }
}

fn verdict_words(tier: &[u8], verdict: &p::Verdict) -> Box<[u8]> {
    let count = u64::try_from(verdict.landings.len()).expect("bounded range");
    match &verdict.failing {
        p::Failures::Tests(tests) if tests.is_empty() => joined(&[
            b"Tier ",
            tier,
            b" run ",
            skein_lib::Decimal::of(verdict.number).as_bytes(),
            b": green, cleared ",
            skein_lib::Decimal::of(count).as_bytes(),
            b" landings",
        ]),
        failing @ (p::Failures::Tests(_) | p::Failures::Many) => {
            let failures = match failing {
                p::Failures::Tests(tests) => {
                    skein_lib::Decimal::of(u64::try_from(tests.len()).expect("bounded tests")).as_bytes().into()
                }
                p::Failures::Many => Box::from(b"many".as_slice()),
            };
            let mut blamed = 0_u64;
            for blame in &verdict.landings {
                if *blame != p::Blame::Cleared {
                    blamed = blamed.checked_add(1).expect("bounded range");
                }
            }
            joined(&[
                b"Tier ",
                tier,
                b" run ",
                skein_lib::Decimal::of(verdict.number).as_bytes(),
                b": red, ",
                &failures,
                b" tests failing, blamed on ",
                skein_lib::Decimal::of(blamed).as_bytes(),
                b" landings",
            ])
        }
    }
}

fn cancelled(d: &mut Domain, row: &mut TierRow, task: u64, out: &mut Queue<Request>) {
    row.record.run = None;
    row.task = None;
    d.offered_tier(task);
    row.requested = false;
    row.flight = None;
    row.result = None;
    row.purpose = None;
    row.received = false;
    let projection = Some(core::mem::replace(&mut row.observations, Box::new([])));
    save(row, out);
    out.push(Request::TierDecision {
        task,
        decision: crate::TierDecision::Done { words: joined(&[b"Tier ", &row.tier, b" cancelled"]) },
        entry: None,
        projection,
    });
}

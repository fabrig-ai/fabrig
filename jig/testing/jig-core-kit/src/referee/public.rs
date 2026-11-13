//! Public reading's independent boundary rules (jig/docs/design/domain/people/public-reading.md, 6).
//!
//! The model consumes submitted writes and boundary messages, never the public index.
//! Its requester paths and bounded prefixes determine which tasks an answer may name.
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{ReplyTo, Token};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Broken {
    PublicCommit,
    PublicRecord,
    Watermark,
    Private,
    Membership,
    Fields,
}
#[derive(Clone, Debug)]
struct Admission {
    read: core::PublicRead,
    tasks: BTreeSet<u64>,
    blocked: bool,
}
#[derive(Clone, Debug, Default)]
pub struct Referee {
    rows: BTreeMap<root::Key, root::Record>,
    pending: BTreeMap<Token, Admission>,
    retained: BTreeSet<u64>,
    frozen: BTreeMap<u32, Vec<(u64, BTreeSet<u64>)>>,
    public_step: bool,
    pub checked: u64,
}
fn token(reply: &mut ReplyTo) -> Token {
    let token = std::mem::replace(reply, ReplyTo::new(Token::new(0))).into_token();
    *reply = ReplyTo::new(token);
    token
}
impl Referee {
    /// Observe a root input and the backlog before that input changes anything.
    pub fn input(&mut self, event: &mut root::Event, backlog: u64, limits: root::Limits) {
        self.public_step = false;
        if let root::Event::StartBegin { .. } = event {
            self.rows.clear();
            self.pending.clear();
            self.retained.clear();
            self.frozen.clear();
        }
        if let root::Event::StartRecord(row) = event {
            self.rows.insert(row.key(), row.clone());
        }
        if let root::Event::Core(core::Event::PublicRead { reply, read }) = event {
            self.public_step = true;
            let pending = Admission {
                read: *read,
                tasks: self.members(read.project, limits.core.public),
                blocked: backlog >= u64::from(limits.public_watermark)
                    || self.pending.values().filter(|read| !read.blocked).count()
                        >= usize::try_from(limits.public_reads).expect("read limit"),
            };
            assert!(self.pending.insert(token(reply), pending).is_none(), "distinct public replies");
        }
    }

    /// # Errors
    /// Reports a write by public reading, or a projection breaking admission or membership.
    pub fn output(&mut self, output: &mut root::Request, limits: root::Limits) -> Result<(), Broken> {
        if let root::Request::Commit { writes, .. } = output {
            if self.public_step {
                if writes
                    .iter()
                    .any(|write| matches!(write, root::Write::Save(root::Record::Core(core::Record::People(_)))))
                {
                    return Err(Broken::PublicRecord);
                }
                return Err(Broken::PublicCommit);
            }
            let previous = self.clone();
            for write in writes.iter() {
                match write {
                    root::Write::Save(row) => {
                        self.rows.insert(row.key(), row.clone());
                    }
                    root::Write::Erase(key) => {
                        self.rows.remove(key);
                    }
                    root::Write::DropTranscript { .. } | root::Write::CopySettingsRevision(_) | root::Write::Wires => {}
                }
            }
            self.endings(&previous, writes, limits.core.public);
        }
        if let root::Request::Deliver(root::Delivery::Core(core::Held::PublicRead { reply, answer })) = output {
            let pending =
                self.pending.remove(&token(reply)).expect("public terminal was admitted or refused at its entrance");
            if pending.blocked && *answer != core::PublicAnswer::Busy {
                return Err(Broken::Watermark);
            }
            if let core::PublicAnswer::Document { document, .. } = answer {
                self.document(&pending, document, limits.core.public)?;
            }
            self.checked += 1;
        }
        Ok(())
    }

    fn endings(&mut self, previous: &Self, writes: &skein_lib::Queue<root::Write>, limits: core::PublicIndexLimits) {
        let rows = previous.tasks();
        for write in writes {
            if let root::Write::Save(root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(task)))) = write {
                if !previous.project_public(task.project)
                    || rows.get(&task.number).is_none_or(|row| row.ended_at.is_some())
                {
                    continue;
                }
                let Some(path) = Self::path(&rows, task.number) else { continue };
                let members = previous.members(Some(u64::from(task.project)), limits);
                if path.len() == 1 {
                    let mut frozen: BTreeSet<_> = members
                        .into_iter()
                        .filter(|n| Self::path(&rows, *n).is_some_and(|p| p.first() == Some(&task.number)))
                        .collect();
                    frozen.insert(task.number);
                    let ended = self.frozen.entry(task.project).or_default();
                    ended.insert(0, (task.number, frozen));
                    ended.truncate(usize::from(limits.ended));
                } else if members.contains(&task.number) {
                    self.retained.insert(task.number);
                }
            }
        }
        self.frozen.retain(|project, _| {
            self.rows.contains_key(&root::Key::Core(core::Key::People(people::Key::Public(*project))))
        });
        let tasks = self.tasks();
        self.retained = self
            .retained
            .iter()
            .copied()
            .filter(|n| tasks.get(n).is_some_and(|row| self.project_public(row.project)))
            .collect();
    }

    fn project_public(&self, project: u32) -> bool {
        self.rows.contains_key(&root::Key::Core(core::Key::People(people::Key::Public(project))))
            && !self.rows.contains_key(&root::Key::Core(core::Key::People(people::Key::Deletion(project))))
    }
    fn tasks(&self) -> BTreeMap<u64, &tasks::TaskRecord> {
        self.rows
            .values()
            .filter_map(|row| {
                if let root::Record::Core(core::Record::Tasks(tasks::Stored::Live(task) | tasks::Stored::Ended(task))) =
                    row
                {
                    Some((task.number, task.as_ref()))
                } else {
                    None
                }
            })
            .collect()
    }
    fn path(rows: &BTreeMap<u64, &tasks::TaskRecord>, task: u64) -> Option<Vec<u64>> {
        let mut path = Vec::new();
        let mut next = Some(task);
        let mut goal = None;
        let mut excluded = false;
        while let Some(number) = next {
            if path.contains(&number) {
                return None;
            }
            let row = rows.get(&number)?;
            path.push(number);
            excluded |= row.flags.conversation || row.recurring.is_some();
            if row.tracked.is_some() {
                if excluded {
                    return None;
                }
                goal = Some(path.len());
            }
            next = if let tasks::Party::Task(parent) = row.requester { Some(parent) } else { None };
        }
        path.truncate(goal?);
        path.reverse();
        Some(path)
    }
    fn members(&self, project: Option<u64>, limits: core::PublicIndexLimits) -> BTreeSet<u64> {
        let Some(project) = project.and_then(|n| u32::try_from(n).ok()) else { return BTreeSet::new() };
        if !self.project_public(project) {
            return BTreeSet::new();
        }
        let rows = self.tasks();
        let paths: BTreeMap<_, _> = rows
            .iter()
            .filter(|(n, row)| row.project == project && (row.ended_at.is_none() || self.retained.contains(n)))
            .filter_map(|(&n, _)| Self::path(&rows, n).map(|path| (n, path)))
            .collect();
        let mut goals: Vec<_> =
            paths.iter().filter(|(n, path)| path.len() == 1 && rows[*n].ended_at.is_none()).map(|(&n, _)| n).collect();
        goals.sort_by_key(|n| (rows[n].tracked, *n));
        goals.truncate(usize::from(limits.goals));
        let mut public = BTreeSet::new();
        for goal in goals {
            public.insert(goal);
            let mut tree: Vec<_> = paths
                .iter()
                .filter(|(_, path)| {
                    path.first() == Some(&goal)
                        && path.len() > 1
                        && path.len().saturating_sub(1) <= usize::from(limits.depth)
                })
                .collect();
            tree.sort_by_key(|(_, path)| *path);
            public.extend(tree.into_iter().take(usize::from(limits.lines)).map(|(&n, _)| n));
        }
        if let Some(ended) = self.frozen.get(&project) {
            for (_, frozen) in ended {
                public.extend(frozen);
            }
        }
        public
    }

    fn document(
        &self,
        admission: &Admission,
        document: &core::PublicDocument,
        limits: core::PublicIndexLimits,
    ) -> Result<(), Broken> {
        let public = self.members(admission.read.project, limits);
        if let Some(project) = admission.read.project
            && !u32::try_from(project).is_ok_and(|p| self.project_public(p))
        {
            return Err(Broken::Private);
        }
        let mut lines = Vec::new();
        let mut references = Vec::new();
        match document {
            core::PublicDocument::Projects(projects) => {
                for project in &projects.projects {
                    if !self.project_public(project.project) {
                        return Err(Broken::Private);
                    }
                    self.name(project.project, &project.name)?;
                }
            }
            core::PublicDocument::Board(board) => {
                if Some(u64::from(board.project)) != admission.read.project {
                    return Err(Broken::Fields);
                }
                self.name(board.project, &board.name)?;
                for goal in board.live.iter().chain(board.ended.iter()) {
                    lines.push(&goal.line);
                    lines.extend(goal.tree.iter());
                }
            }
            core::PublicDocument::Card(card) => {
                lines.push(&card.line);
                lines.extend(card.delegates.iter());
                references.extend(card.lineage.iter().copied());
            }
            core::PublicDocument::Application(application) => {
                references.extend(application.tasks.iter().copied());
            }
        }
        if let Some(task) = admission.read.task {
            references.push(task);
        }
        let rows = self.tasks();
        for line in lines {
            references.push(line.task);
            if let Some(parent) = line.parent {
                references.push(parent);
            }
            let Some(row) = rows.get(&line.task) else { return Err(Broken::Membership) };
            let first = row.spec.words.split(|b| *b == b'\n').next().expect("first line");
            if line.title.bytes.as_ref() != first.get(..line.title.bytes.len()).ok_or(Broken::Fields)?
                || line.title.bytes.len() > 256
                || line.title.cut != (first.len() > line.title.bytes.len())
            {
                return Err(Broken::Fields);
            }
            let path = Self::path(&rows, line.task).ok_or(Broken::Membership)?;
            let parent = path.iter().rev().nth(1).copied();
            if line.parent != parent
                || line.priority != row.tracked
                || line.goal != row.tracked.is_some()
                || line.spent != row.tracked.map(|_| row.numbers.spent.saturating_add(row.numbers.spent_below))
                || line.made != row.created_at.as_secs()
                || line.changed != row.changed.as_secs()
                || line.active != row.run_since.map(skein_lib::Wall::as_secs)
                || line.ended != row.ended_at.map(skein_lib::Wall::as_secs)
                || line.executor
                    != match row.executor {
                        tasks::Executor::Agent { .. } => core::PublicExecutor::Agent,
                        tasks::Executor::Procedure { .. } => core::PublicExecutor::Procedure,
                        tasks::Executor::Person(_) => core::PublicExecutor::Person,
                        tasks::Executor::Job { .. } => core::PublicExecutor::Job,
                    }
            {
                return Err(Broken::Fields);
            }
            let mut counts = core::PublicCounts { working: 0, waiting: 0, held: 0, closing: 0, ended: 0 };
            for number in &public {
                let candidate = rows.get(number).ok_or(Broken::Membership)?;
                if candidate.requester != tasks::Party::Task(line.task) {
                    continue;
                }
                match candidate.phase {
                    tasks::Phase::Waiting => counts.waiting += 1,
                    tasks::Phase::Held { .. } => counts.held += 1,
                    tasks::Phase::Closing(_) => counts.closing += 1,
                    tasks::Phase::Ended(_) => counts.ended += 1,
                    tasks::Phase::Active(_) => counts.working += 1,
                }
            }
            if counts != line.delegates {
                return Err(Broken::Fields);
            }
        }
        if references.iter().any(|n| !admission.tasks.contains(n) || !public.contains(n)) {
            return Err(Broken::Membership);
        }
        Ok(())
    }
    fn name(&self, project: u32, name: &str) -> Result<(), Broken> {
        if let Some(root::Record::Core(core::Record::People(people::Stored::Policy { name: expected, .. }))) =
            self.rows.get(&root::Key::Core(core::Key::People(people::Key::Policy(project))))
            && expected.as_ref() == name
        {
            return Ok(());
        }
        Err(Broken::Fields)
    }
}

impl crate::world::World {
    pub fn public_rule_input(&mut self, event: &mut root::Event) {
        let limits = self.env().limits;
        if limits.public_reads == 0 && !matches!(event, root::Event::Core(core::Event::PublicRead { .. })) {
            return;
        }
        let backlog = self.domain.journal_mut().backlog();
        self.public_referee.input(event, backlog, limits);
    }
    pub fn public_rule_output(&mut self, output: &mut root::Request) {
        let limits = self.env().limits;
        if limits.public_reads == 0 && self.public_referee.pending.is_empty() {
            return;
        }
        self.public_referee.output(output, limits).expect("public referee");
    }
}

/// Checked between every iteration, including stories with public reading off.
pub fn check(world: &crate::world::World) {
    assert!(
        world.public_referee.pending.values().filter(|read| !read.blocked).count()
            <= usize::try_from(world.env().limits.public_reads).expect("public read bound"),
        "public reads have separate bounded room"
    );
}

/// Feed independent planted breaches through the same rule used by the world.
#[must_use]
pub fn planted_breaches(
    world: &crate::world::World,
    read: core::PublicRead,
    document: &core::PublicDocument,
) -> Vec<Broken> {
    let limits = world.env().limits;
    let original = world.public_referee.clone();
    let admission = Admission { read, tasks: original.members(read.project, limits.core.public), blocked: false };
    assert_eq!(original.document(&admission, document, limits.core.public), Ok(()), "unplanted control");
    let mut caught = Vec::new();
    for record in [false, true] {
        let mut rule = original.clone();
        rule.public_step = true;
        let mut writes = skein_lib::Queue::uncapped();
        if record {
            writes.push(root::Write::Save(
                original
                    .rows
                    .values()
                    .find(|row| matches!(row, root::Record::Core(core::Record::People(people::Stored::Person { .. }))))
                    .expect("person fixture")
                    .clone(),
            ));
        }
        caught.push(
            rule.output(&mut root::Request::Commit { number: 999, writes }, limits).expect_err("public cannot write"),
        );
    }
    let mut rule = original.clone();
    let mut blocked = admission.clone();
    blocked.blocked = true;
    rule.pending.insert(Token::new(999), blocked);
    caught.push(
        rule.output(
            &mut root::Request::Deliver(root::Delivery::Core(core::Held::PublicRead {
                reply: ReplyTo::new(Token::new(999)),
                answer: core::PublicAnswer::Document { decided: skein_lib::Time::ZERO, document: document.clone() },
            })),
            limits,
        )
        .expect_err("no admission at watermark"),
    );
    let mut unknown_at_decision = admission.clone();
    unknown_at_decision.tasks.clear();
    caught.push(
        original
            .document(&unknown_at_decision, document, limits.core.public)
            .expect_err("index membership at decision"),
    );
    for change in [0, 1, 2, 3] {
        let mut rule = original.clone();
        if change == 0 {
            rule.rows.remove(&root::Key::Core(core::Key::People(people::Key::Public(1))));
        } else if change == 3 {
            let deletion = root::Record::Core(core::Record::People(people::Stored::Deletion {
                project: 1,
                step: people::DeletionStep::Tasks,
                grace: None,
            }));
            rule.rows.insert(deletion.key(), deletion);
        } else {
            for row in rule.rows.values_mut() {
                if let root::Record::Core(core::Record::Tasks(tasks::Stored::Live(task))) = row {
                    if change == 1 && Some(task.number) == read.task {
                        task.requester = tasks::Party::Person(1);
                    }
                    if change == 2 {
                        task.tracked = None;
                    }
                }
            }
        }
        caught.push(rule.document(&admission, document, limits.core.public).expect_err("revocation precedes answer"));
    }
    caught.push(
        original
            .document(&admission, document, core::PublicIndexLimits { lines: 0, ..limits.core.public })
            .expect_err("task beyond index bound"),
    );
    let mut foreign = document.clone();
    let core::PublicDocument::Card(card) = &mut foreign else { panic!("card control") };
    card.line.task = u64::MAX;
    caught.push(original.document(&admission, &foreign, limits.core.public).expect_err("foreign task"));
    let mut text = document.clone();
    let core::PublicDocument::Card(card) = &mut text else { panic!("card control") };
    card.line.title.bytes = b"hidden-delegate-spec".as_slice().into();
    caught.push(original.document(&admission, &text, limits.core.public).expect_err("non-title text"));
    caught
}

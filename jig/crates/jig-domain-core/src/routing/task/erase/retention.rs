//! Timer passes use deletion's one walk (protocol/store/retention.md, 4–6).
use super::{Core, Env, Erasing, Key, Limits, Mode, Phase, Queue, Request, Walk, Write, people, reads, tasks};
use alloc::boxed::Box;
use skein_lib::{Duration, List, Wall};
use tasks::{RetentionRow, RetentionStage as Stage};

#[derive(Hash, Debug)]
pub(super) struct Pass {
    stage: Stage,
    rows: Queue<RetentionRow>,
    pending: Option<Pending>,
}
#[derive(Hash, Debug)]
enum Pending {
    Owner(RetentionRow),
    Reading { row: RetentionRow, record: Box<tasks::TaskRecord> },
    Proposal { row: RetentionRow, proposal: Box<tasks::PersonProposal> },
    Remains { row: RetentionRow, keys: List<Key> },
}
impl Pass {
    pub(super) fn new() -> Self {
        Self { stage: Stage::Turns, rows: Queue::uncapped(), pending: None }
    }
}

fn horizon(limits: &Limits, stage: Stage) -> Duration {
    match stage {
        Stage::Turns => limits.retention.turns,
        Stage::Record | Stage::Proposal => limits.retention.records,
        Stage::Summary => limits.retention.summaries,
        Stage::Forgotten => Duration::ZERO,
    }
}
fn following(stage: Stage) -> Option<Stage> {
    match stage {
        Stage::Turns => Some(Stage::Record),
        Stage::Record => Some(Stage::Summary),
        Stage::Summary => Some(Stage::Forgotten),
        Stage::Forgotten => Some(Stage::Proposal),
        Stage::Proposal => None,
    }
}

pub(super) fn next(core: &mut Core, env: &Env<Limits>, out: &mut Queue<Request>, mut walk: Walk) {
    if let Some(row) = walk.retention.rows.pop() {
        if row.stage == Stage::Turns {
            if core.tasks.stub(row.number).is_some() {
                defer(out, row, env.wall);
            } else {
                out.push(Request::Write(Write::DropTranscript { task: row.number }));
                move_row(out, row, Some(Stage::Record));
            }
            // One task per decision, with the next read behind this commit.
            keys(core, env, out, walk, Box::new([]));
        } else {
            let key = match row.stage {
                Stage::Record => tasks::Key::Ended(row.number),
                Stage::Summary | Stage::Forgotten => tasks::Key::Summary(row.number),
                Stage::Proposal => tasks::Key::DecidedProposal(row.number),
                Stage::Turns => unreachable!("turns read no owner"),
            };
            walk.retention.pending = Some(Pending::Owner(row));
            keys(core, env, out, walk, Box::new([Key::Tasks(key)]));
        }
        return;
    }
    // Before the first horizon, a row at the epoch is not yet due.
    for _ in 0_u8..5 {
        let stage = walk.retention.stage;
        let horizon = horizon(&env.limits, stage).as_nanos();
        if env.wall.as_nanos() >= horizon {
            let through = Wall::from_nanos(env.wall.as_nanos().saturating_sub(horizon));
            core.erasures.walk = Some(walk);
            super::ask(core, env, out, reads::Scope::Retention { stage, through }, None, env.limits.documents.rows);
            return;
        }
        let Some(stage) = following(stage) else {
            super::finish(core, env, out, walk.select);
            return;
        };
        walk.retention.stage = stage;
    }
}
fn keys(core: &mut Core, env: &Env<Limits>, out: &mut Queue<Request>, walk: Walk, keys: Box<[Key]>) {
    core.erasures.walk = Some(walk);
    super::ask(core, env, out, reads::Scope::ErasureKeys { keys }, None, 1);
}

#[expect(clippy::too_many_lines, reason = "the loaded stage chooses one bounded continuation")]
pub(super) fn loaded(
    core: &mut Core,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
    mut walk: Walk,
    rows: Box<[reads::Row]>,
    range: reads::Range,
    after: Option<reads::Position>,
) {
    let Some(pending) = walk.retention.pending.take() else {
        if rows.is_empty() {
            if let reads::Scope::Retention { .. } = range.scope {
                let Some(stage) = following(walk.retention.stage) else {
                    super::finish(core, env, out, walk.select);
                    return;
                };
                walk.retention.stage = stage;
            }
            next(core, env, out, walk);
        } else {
            for row in rows {
                let crate::Record::Tasks(tasks::Stored::Retention(row)) = row.value else {
                    unreachable!("retention range contains retention rows")
                };
                walk.retention.rows.push(row);
            }
            next(core, env, out, walk);
        }
        return;
    };
    if let Pending::Remains { row, mut keys } = pending {
        for value in rows {
            keys.push(value.value.key()).expect("one decision and one update per proposal");
        }
        if after.is_some() {
            walk.retention.pending = Some(Pending::Remains { row, keys });
            core.erasures.walk = Some(walk);
            reads::erase_load(core, reads::Range { after, ..range }, out);
        } else {
            for key in keys.into_boxed() {
                out.push(Request::Write(Write::Erase(key)));
            }
            move_row(out, row, None);
            next(core, env, out, walk);
        }
        return;
    }
    let value = match rows.into_iter().next() {
        Some(row) => Some(row.value),
        None => None,
    };
    match pending {
        Pending::Owner(row) => match value {
            Some(crate::Record::Tasks(tasks::Stored::Ended(record))) => {
                let live_requester = match record.requester {
                    tasks::Party::Task(task) => core.tasks.task(task).is_some(),
                    tasks::Party::Person(_) | tasks::Party::Deployment { .. } => false,
                };
                if core.tasks.stub(row.number).is_some() || live_requester {
                    defer(out, row, env.wall);
                    next(core, env, out, walk);
                } else if let tasks::Party::Person(party) = record.requester {
                    walk.retention.pending = Some(Pending::Reading { row, record });
                    keys(
                        core,
                        env,
                        out,
                        walk,
                        Box::new([Key::People(people::Key::EndedRead { task: row.number, party })]),
                    );
                } else {
                    erase_record(core, env, out, walk, row, &record);
                }
            }
            Some(crate::Record::Tasks(tasks::Stored::Summary(summary))) => {
                if row.stage == Stage::Summary {
                    super::erase_ended_indexes(&summary, out);
                    out.push(Request::Write(Write::Erase(Key::Tasks(tasks::Key::Summary(row.number)))));
                    move_row(out, row, None);
                    next(core, env, out, walk);
                } else {
                    walk.task = Some(Erasing { record: summary, mode: Mode::Forgotten(row), phase: Phase::Rows });
                    super::own_rows(core, env, out, walk, None);
                }
            }
            Some(crate::Record::Tasks(tasks::Stored::DecidedProposal(proposal))) => {
                let key = Key::Core(crate::CoreKey::ProposalDecision {
                    proposer: people::Proposer::Party(proposal.proposer),
                    proposal: row.number,
                });
                walk.retention.pending = Some(Pending::Proposal { row, proposal });
                keys(core, env, out, walk, Box::new([key]));
            }
            None => {
                if row.stage == Stage::Proposal {
                    walk.retention.pending = Some(Pending::Remains { row, keys: List::with_capacity(2) });
                    core.erasures.walk = Some(walk);
                    super::ask(
                        core,
                        env,
                        out,
                        reads::Scope::ProposalRemains { proposal: row.number },
                        None,
                        env.limits.documents.rows,
                    );
                } else {
                    move_row(out, row, None);
                    next(core, env, out, walk);
                }
            }
            Some(
                crate::Record::Tasks(_)
                | crate::Record::Core(_)
                | crate::Record::People(_)
                | crate::Record::Notes(_)
                | crate::Record::Accounts(_),
            ) => unreachable!("retention owner kind"),
        },
        Pending::Remains { .. } => unreachable!("proposal remains continued above"),
        Pending::Reading { row, record } => {
            let turns = match value {
                Some(crate::Record::People(people::Stored::EndedRead { through, .. })) => Some(through),
                None => Some(0),
                Some(
                    crate::Record::Tasks(_)
                    | crate::Record::Core(_)
                    | crate::Record::People(_)
                    | crate::Record::Notes(_)
                    | crate::Record::Accounts(_),
                ) => unreachable!("ended read position"),
            };
            let tasks::Party::Person(party) = record.requester else { unreachable!("party read test") };
            // A requester no longer in the roster counts as read (retention.md, 6.2).
            let read = match core.people.results_seen(party) {
                None => true,
                Some(results) => reads::forget::forgets(
                    &owner(&record, party),
                    reads::forget::Reading { results, turns, named: core.tasks.stub(row.number).is_some() },
                )
                .is_some(),
            };
            if read {
                erase_record(core, env, out, walk, row, &record);
            } else {
                defer(out, row, env.wall);
                next(core, env, out, walk);
            }
        }
        Pending::Proposal { row, proposal } => {
            let at = match value {
                Some(crate::Record::Core(crate::CoreRecord::ProposalDecision(decision))) => {
                    out.push(Request::Write(Write::Erase(Key::Core(crate::CoreKey::ProposalDecision {
                        proposer: decision.proposer,
                        proposal: decision.proposal,
                    }))));
                    decision.at
                }
                None => row.since,
                Some(
                    crate::Record::Tasks(_)
                    | crate::Record::Core(_)
                    | crate::Record::People(_)
                    | crate::Record::Notes(_)
                    | crate::Record::Accounts(_),
                ) => unreachable!("proposal decision"),
            };
            let update = tasks::Update {
                party: proposal.proposer,
                at,
                task: proposal.first(),
                object: tasks::UpdateObject::Proposal { proposal: tasks::Newest::of(row.number) },
            };
            out.push(Request::Write(Write::Erase(Key::Tasks(update.key()))));
            out.push(Request::Write(Write::Erase(Key::Tasks(tasks::Key::DecidedProposal(row.number)))));
            move_row(out, row, None);
            next(core, env, out, walk);
        }
    }
}

fn owner(record: &tasks::TaskRecord, party: u64) -> reads::forget::Owner {
    reads::forget::Owner {
        task: record.number,
        party,
        project: record.project,
        label: record.label,
        ended_at: record.ended_at.expect("ended record"),
        result_position: record.result_position,
        listed: record.flags.listed,
        last_reply: record.flags.conversation.then_some(record.last_reply),
    }
}
fn erase_record(
    core: &mut Core,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
    mut walk: Walk,
    row: RetentionRow,
    record: &tasks::TaskRecord,
) {
    let whole = record.flags.retention == tasks::Retention::ForgottenOnceRead
        && match record.requester {
            tasks::Party::Person(_) => true,
            tasks::Party::Task(_) | tasks::Party::Deployment { .. } => false,
        };
    walk.task = Some(Erasing {
        record: Box::new(tasks::TaskSummary::of(record)),
        mode: Mode::Record { row, whole },
        phase: Phase::Rows,
    });
    super::own_rows(core, env, out, walk, None);
}
fn defer(out: &mut Queue<Request>, row: RetentionRow, now: Wall) {
    move_row(out, row, None);
    out.push(Request::Write(Write::Save(crate::Record::Tasks(tasks::Stored::Retention(RetentionRow {
        since: now,
        ..row
    })))));
}
pub(super) fn move_row(out: &mut Queue<Request>, row: RetentionRow, next: Option<Stage>) {
    out.push(Request::Write(Write::Erase(Key::Tasks(row.key()))));
    if let Some(stage) = next {
        out.push(Request::Write(Write::Save(crate::Record::Tasks(tasks::Stored::Retention(RetentionRow {
            stage,
            ..row
        })))));
    }
}

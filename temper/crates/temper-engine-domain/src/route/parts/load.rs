//! Archive and joined-query continuations (temper/docs/design/client/forge/parts.md, 4).
use super::{
    Box, CoreRange, Decision, Delivery, Domain, Ended, Env, Flight, Id, Key, Limits, List, MadePart, Making, Map,
    MilestoneTotals, NoteLoad, PartFailure, Phase, Range, Read, Record, TaskLine, Token, core, d, delegates, drive,
    emit, loads, notes, push, tasks,
};
use ::core::cmp::Reverse;

pub(crate) fn failed(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, waiter: Token, why: PartFailure) {
    let Some(Read::Parts(mut flight)) = crate::archive::take_read(domain, waiter) else { return };
    flight.phase = Phase::Making;
    push(&mut flight, MadePart::Failed(why));
    drive(domain, env, decision, Some(waiter), flight);
}

pub(crate) fn loaded(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    waiter: Token,
    rows: Box<[Record]>,
    next: Option<Key>,
    cut: Option<loads::Cut>,
) {
    let Some(Read::Parts(mut flight)) = crate::archive::take_read(domain, waiter) else { return };
    let phase = ::core::mem::replace(&mut flight.phase, Phase::Making);
    let result = if cut.is_some() { Err(PartFailure::Unreadable) } else { ordinary(domain, phase, rows, next) };
    continue_read(domain, env, decision, waiter, flight, result);
}
fn continue_read(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    waiter: Token,
    mut flight: Box<Flight>,
    result: Result<Option<(Phase, Range)>, PartFailure>,
) {
    match result {
        Ok(Some((phase, range))) => {
            flight.phase = phase;
            *domain.archive_reads.get_mut(Id::from_token(waiter)).expect("part slot") = Some(Read::Parts(flight));
            emit(decision, &env.limits, Delivery::Load { waiter, range, after: None });
        }
        Ok(None) => drive(domain, env, decision, Some(waiter), flight),
        Err(why) => {
            push(&mut flight, MadePart::Failed(why));
            drive(domain, env, decision, Some(waiter), flight);
        }
    }
}

fn ordinary(
    domain: &mut Domain,
    phase: Phase,
    rows: Box<[Record]>,
    next: Option<Key>,
) -> Result<Option<(Phase, Range)>, PartFailure> {
    match phase {
        Phase::Index(mut read) => {
            let mut last = read.after;
            for row in rows {
                let Record::Core(core::Record::Notes(notes::Record::Line(line))) = row else {
                    return Err(PartFailure::Unreadable);
                };
                if line.scope != (notes::Scope::Goal { project: read.project, goal: read.task })
                    || Some(line.name) <= last
                    || !notes::valid_line(&line, &domain.limits.notes)
                {
                    return Err(PartFailure::Unreadable);
                }
                last = Some(line.name);
                if read.kept.document.is_none() && !line.description.starts_with(b"work:") {
                    read.kept.document = Some(line.description.clone());
                }
                if line.description.starts_with(b"report: ") {
                    read.names.push(line.name);
                }
            }
            if next.is_some() && last == read.after {
                return Err(PartFailure::Unreadable);
            }
            read.after = last;
            read.next = next.is_some();
            Ok(notes_next(domain, read))
        }
        Phase::Note(mut read, name) => {
            if next.is_some() || rows.len() != 1 {
                return Err(PartFailure::Unreadable);
            }
            let Some(Record::Core(core::Record::Notes(notes::Record::Entry(entry)))) = rows.into_iter().next() else {
                return Err(PartFailure::Unreadable);
            };
            if entry.name != name
                || entry.scope != (notes::Scope::Goal { project: read.project, goal: read.task })
                || !notes::valid_entry(&entry, &domain.limits.notes)
            {
                return Err(PartFailure::Unreadable);
            }
            if let Some(headline) = entry.description.strip_prefix(b"report: ") {
                let key = (entry.changed, entry.name);
                if read.best.is_none() || Some(key) > read.best {
                    let end = skein_lib::bytes::find(headline, b"\n").unwrap_or(headline.len());
                    let headline = headline.get(..end).expect("headline first line");
                    let headline = headline.strip_suffix(b"\r").unwrap_or(headline);
                    read.kept.report = Some((headline.into(), entry.changed));
                    read.best = Some(key);
                }
            }
            Ok(notes_next(domain, read))
        }
        Phase::Milestone { project, task } => {
            if next.is_some() || rows.len() != 1 {
                return Err(PartFailure::Unreadable);
            }
            let Some(Record::Core(core::Record::Tasks(tasks::Stored::Ended(row)))) = rows.into_iter().next() else {
                return Err(PartFailure::Unreadable);
            };
            if row.project != project || row.number != task || !tasks::gathering::executor(row.executor) {
                return Err(PartFailure::Unreadable);
            }
            let totals = MilestoneTotals {
                task,
                done: row.ended_delegates.done,
                failed: row.ended_delegates.failed,
                closed: row.ended_delegates.cancelled,
                live: 0,
                ended: row.ended_at,
                last_landing: None,
                spent: row.numbers.spent.checked_add(row.numbers.spent_below).expect("task accounting"),
                reds: 0,
            };
            domain.parts.entry(project, task).ended = Some(Ended {
                count: totals.done.saturating_add(totals.failed).saturating_add(totals.closed),
                loaded: totals.done == 0 && totals.failed == 0 && totals.closed == 0,
                whole: match row.concluded {
                    Some(concluded) => Some(concluded.at),
                    None => None,
                },
                totals: Some(totals),
                lines: List::with_capacity(16),
            });
            Ok(None)
        }
        Phase::Making | Phase::Delegates { .. } => Err(PartFailure::Unreadable),
    }
}
fn notes_next(domain: &mut Domain, mut read: NoteLoad) -> Option<(Phase, Range)> {
    if read.epoch != domain.parts.epoch {
        return None;
    }
    if let Some(name) = read.names.pop() {
        return Some((Phase::Note(read, name), Range::Core(CoreRange::Notes(notes::Range::Entry { name }))));
    }
    if read.next {
        let range = Range::Core(CoreRange::Notes(notes::Range::Lines {
            scope: notes::Scope::Goal { project: read.project, goal: read.task },
            after: read.after,
        }));
        return Some((Phase::Index(read), range));
    }
    domain.parts.entry(read.project, read.task).notes = Some(read.kept);
    None
}

pub(crate) fn joined(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    waiter: Token,
    result: core::reads::Loaded,
) {
    let Some(Read::Parts(mut flight)) = crate::archive::take_read(domain, waiter) else { return };
    let Phase::Delegates { project, task, mut ended, page } = ::core::mem::replace(&mut flight.phase, Phase::Making)
    else {
        unreachable!("joined part load")
    };
    let result = match result {
        core::reads::Loaded::Page { rows, next } => {
            let mut ordered = Map::uncapped();
            let mut valid = true;
            for row in rows {
                let line = joined_line(domain, env, flight.ask.party, project, task, &row);
                match line {
                    Ok(line) => {
                        ordered.insert((Reverse(line.ended_at), line.chip.number), line).expect("ended page");
                    }
                    Err(_) => {
                        valid = false;
                        break;
                    }
                }
            }
            if valid {
                for line in &ended.lines {
                    ordered.insert((Reverse(line.ended_at), line.chip.number), line.clone()).expect("ended prefix");
                }
                ended.lines.clear();
                for (_, line) in &ordered {
                    if ended.lines.len() == 16 {
                        break;
                    }
                    ended.lines.push(line.clone()).expect("ended prefix");
                }
                if let Some(after) = next
                    && page < 3
                {
                    let Making::Joined(phase, range) = delegates(
                        env,
                        flight.ask.party,
                        project,
                        task,
                        ended,
                        page.checked_add(1).expect("four pages"),
                        Some(after),
                    ) else {
                        unreachable!("joined request")
                    };
                    flight.phase = phase;
                    *domain.archive_reads.get_mut(Id::from_token(waiter)).expect("part slot") =
                        Some(Read::Parts(flight));
                    emit(decision, &env.limits, Delivery::PartLoad { waiter, range });
                    return;
                }
                ended.loaded = true;
                domain.parts.entry(project, task).ended = Some(ended);
                Ok(None)
            } else {
                Err(PartFailure::Unreadable)
            }
        }
        core::reads::Loaded::Busy => Err(PartFailure::Busy),
        core::reads::Loaded::Gone | core::reads::Loaded::TooLinked { .. } | core::reads::Loaded::TurnProblem { .. } => {
            Err(PartFailure::Unreadable)
        }
    };
    continue_read(domain, env, decision, waiter, flight, result);
}

fn joined_line(
    domain: &Domain,
    env: &Env<Limits>,
    party: u64,
    project: u32,
    task: u64,
    row: &core::reads::Row,
) -> Result<TaskLine, d::Missing> {
    let limits = crate::limits::environment_core(env).limits;
    let owner = if let core::Record::Tasks(tasks::Stored::Ended(owner)) = &row.value {
        Some(owner.as_ref())
    } else {
        row.owner.as_deref()
    };
    match owner {
        Some(owner)
            if owner.project == project && owner.requester == tasks::Party::Task(task) && owner.ended_at.is_some() =>
        {
            d::task_line(domain.engine.core(), &limits, party, owner, row.ended_read)
        }
        Some(_) => Err(d::Missing::Unknown),
        None => match &row.summary {
            Some(summary) if summary.project == project && summary.requester == tasks::Party::Task(task) => {
                d::task_line_from_summary(domain.engine.core(), &limits, party, summary)
            }
            Some(_) | None => Err(d::Missing::Unknown),
        },
    }
}

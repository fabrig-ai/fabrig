//! Deleted repositories' forge rows (temper/docs/design/protocol/store.md, 4).
use super::refusal;
use crate::{Bounds, keys, records};
use alloc::boxed::Box;
use jig_protocol_store::{JobAnswer, JobPage, JobRefusal, JobRow, JobWrite, Store};
use skein_forge::api::Repository;
use skein_lib::{Decimal, List, Writer};
use temper_engine_domain_forge_vocabulary::{ApplicationOutcome, ForgeState, Key, Name, RecordKind, Stored, Topic};

/// Stored facts, read afresh while each job page is out.
#[derive(Debug)]
pub struct Present {
    pub repositories: Box<[Repository]>,
    pub forges: Box<[(u16, bool)]>,
    pub last: u16,
}
impl Default for Present {
    fn default() -> Self {
        Self { repositories: Box::new([]), forges: Box::new([]), last: 0 }
    }
}
impl Present {
    fn missing(&self, repository: Repository, kind: RecordKind) -> bool {
        if self.repositories.contains(&repository) {
            return false;
        }
        let mut ended = repository.forge != 0 && repository.forge <= self.last;
        for (forge, stored_ended) in &self.forges {
            if *forge == repository.forge {
                ended = *stored_ended;
                break;
            }
        }
        !(kind.historical_repository() && ended)
    }
}

/// Read families 2, 25 and 26 through the open's read edge, not the configuration.
pub fn read_present(component: &mut Store, bounds: &Bounds) -> Result<Present, JobRefusal> {
    let mut repositories = List::with_capacity(bounds.forge().repositories);
    let mut forges = List::with_capacity(bounds.forge().forges);
    let mut last = 0;
    for (family, end, limit) in [(2, 3, bounds.forge().repositories), (25, 26, bounds.forge().forges), (26, 27, 1)] {
        let mut after = None;
        let mut done = false;
        for _ in 0..=limit {
            let Ok(page) = component.patch_page(
                skein_kv::Range { start: Box::new([family]), end: Some(Box::new([end])) },
                after.as_ref(),
                1,
            ) else {
                return Err(refusal(None, "cannot read deleted repository job facts"));
            };
            for row in page.rows {
                let name = match family {
                    2 => "Repository",
                    25 => "Forge",
                    26 => "ForgeNumber",
                    _ => unreachable!("fact families"),
                };
                if family == 2 {
                    let Some(Key::Repository(repository)) = keys::decode(&row.key, bounds.forge()) else {
                        return Err(unreadable(&row.key, name));
                    };
                    if repositories.push(repository).is_err() {
                        return Err(unreadable(&row.key, name));
                    }
                } else {
                    let Ok(stored) = records::row(&row.key, &row.value, bounds) else {
                        return Err(unreadable(&row.key, name));
                    };
                    if family == 25 {
                        let Stored::Forge(forge) = stored else {
                            return Err(unreadable(&row.key, name));
                        };
                        let ended = match forge.state {
                            ForgeState::Kept => false,
                            ForgeState::Ended { .. } => true,
                        };
                        if forges.push((forge.number, ended)).is_err() {
                            return Err(unreadable(&row.key, name));
                        }
                    } else {
                        let Stored::ForgeNumber { last: number } = stored else {
                            return Err(unreadable(&row.key, name));
                        };
                        last = number;
                    }
                }
            }
            after = page.next;
            if after.is_none() {
                done = true;
                break;
            }
        }
        if !done {
            return Err(refusal(None, "deleted repository job facts exceed their bound"));
        }
    }
    Ok(Present { repositories: repositories.into_boxed(), forges: forges.into_boxed(), last })
}

pub(super) fn page(page: &JobPage, present: &Present, bounds: &Bounds) -> Result<JobAnswer, JobRefusal> {
    let mut writes = List::uncapped();
    let mut bytes = 0_u64;
    let mut through = None;
    let mut counts = [0_u64; 11];
    for row in &page.rows {
        let judged = judge(row, present, bounds)?;
        if let Some(write) = judged.write {
            let cost = bytes.checked_add(write.bytes());
            let fits = match (cost, writes.len().checked_add(1)) {
                (Some(cost), Some(count)) => cost <= page.room.bytes && count <= page.room.writes,
                (None, _) | (_, None) => false,
            };
            if !fits {
                if through.is_none() {
                    return Err(refusal(Some(row.key.clone()), "row exceeds the deleted repository job edge"));
                }
                return Ok(JobAnswer { writes: writes.into_boxed(), through, counts: Box::new(counts) });
            }
            bytes = cost.expect("cost checked");
            writes.push(write).expect("uncapped job writes");
            let total = counts.first_mut().expect("total count");
            *total = total.saturating_add(judged.amount);
            let family = counts.get_mut(judged.count).expect("declared family count");
            *family = family.saturating_add(judged.amount);
        }
        through = Some(row.key.clone());
    }
    Ok(JobAnswer { writes: writes.into_boxed(), through: None, counts: Box::new(counts) })
}

fn repository(name: &Name) -> Repository {
    Repository { forge: name.forge, repository: name.repository }
}

struct Judged {
    write: Option<JobWrite>,
    count: usize,
    amount: u64,
}
impl Judged {
    const fn keep() -> Self {
        Self { write: None, count: 0, amount: 0 }
    }
    const fn write(write: JobWrite, count: usize, amount: u64) -> Self {
        Self { write: Some(write), count, amount }
    }
}

#[expect(clippy::disallowed_methods, reason = "job refusals join ASCII literals and decimal digits into text")]
fn words(parts: &[&[u8]]) -> Box<str> {
    let mut len = 0_usize;
    for part in parts {
        len = len.checked_add(part.len()).expect("bounded job refusal");
    }
    let mut writer = Writer::new(len);
    for part in parts {
        writer.put(part).expect("measured job refusal");
    }
    Box::from(core::str::from_utf8(&writer.finish()).expect("job refusal is ASCII"))
}
fn unreadable(key: &[u8], family: &str) -> JobRefusal {
    JobRefusal { key: Some(Box::from(key)), why: words(&[b"unreadable ", family.as_bytes(), b" row"]) }
}

#[expect(clippy::too_many_lines, reason = "the job judges the walked families in one exhaustive record match")]
fn judge(row: &JobRow, present: &Present, bounds: &Bounds) -> Result<Judged, JobRefusal> {
    let family = match row.key.first() {
        Some(1) => "ProposedEffect",
        Some(3) => "Hold",
        Some(4) => "Names",
        Some(5) => "Subscription",
        Some(6) => "BranchHead",
        Some(7) => "PullState",
        Some(8) => "Ci",
        Some(10) => "Entry",
        Some(11) => "Client",
        Some(13) => "Issue",
        Some(14) => "Release",
        Some(15) => "Application",
        Some(23) => "Landing",
        Some(24) => "Receipt",
        Some(28) => "Tier",
        Some(29) => "Verification",
        Some(30) => "LandingLog",
        Some(42) => "Follow",
        Some(47) => "ChangeCold",
        _ => return Err(refusal(Some(row.key.clone()), "unreadable job key")),
    };
    // Row decoding checks that each key field agrees with the record's.
    let Ok(stored) = records::row(&row.key, &row.value, bounds) else {
        return Err(unreadable(&row.key, family));
    };
    if let Stored::Names { task, resources } = stored {
        let mut kept = List::uncapped();
        let mut removed = 0_u64;
        for name in resources {
            if present.missing(repository(&name), RecordKind::Names) {
                removed = removed.saturating_add(1);
            } else {
                kept.push(name).expect("uncapped names");
            }
        }
        if removed == 0 {
            return Ok(Judged::keep());
        }
        let write = if kept.is_empty() {
            JobWrite::Erase { key: row.key.clone() }
        } else {
            JobWrite::Put {
                key: row.key.clone(),
                value: records::encode(&Stored::Names { task, resources: kept.into_boxed() }),
            }
        };
        return Ok(Judged::write(write, 10, removed));
    }
    let mut repositories = List::uncapped();
    let mut dropped = None;
    let kind = match stored {
        Stored::BranchHead(value) => {
            repositories.push(repository(&value.name)).expect("uncapped");
            dropped = Some(1);
            RecordKind::BranchHead
        }
        Stored::PullState(value) => {
            repositories.push(repository(&value.name)).expect("uncapped");
            dropped = Some(2);
            RecordKind::PullState
        }
        Stored::Ci(value) => {
            repositories.push(value.repository).expect("uncapped");
            dropped = Some(3);
            RecordKind::Ci
        }
        Stored::Client(value) => {
            match value {
                skein_forge_client::Stored::Live(value) => {
                    repositories.push(value.watch.resource.repository).expect("uncapped");
                    dropped = Some(4);
                }
                skein_forge_client::Stored::Repository(value) => {
                    repositories.push(value.repository).expect("uncapped");
                }
                skein_forge_client::Stored::Writer(_) => {}
            }
            RecordKind::Client
        }
        Stored::Receipt { key, .. } => {
            repositories.push(key.repository).expect("uncapped");
            dropped = Some(5);
            RecordKind::Receipt
        }
        Stored::Tier(value) => {
            repositories.push(value.repository).expect("uncapped");
            dropped = Some(6);
            RecordKind::Tier
        }
        Stored::Verification(value) => {
            repositories.push(value.repository).expect("uncapped");
            dropped = Some(7);
            RecordKind::Verification
        }
        Stored::LandingLog(value) => {
            repositories.push(value.repository).expect("uncapped");
            dropped = Some(8);
            RecordKind::LandingLog
        }
        Stored::Application { outcome, .. } => {
            match outcome {
                ApplicationOutcome::Adopted(value) => {
                    repositories.push(value.repository).expect("uncapped");
                    dropped = Some(9);
                }
                ApplicationOutcome::CoreRefused(_)
                | ApplicationOutcome::ForgeRefused(_)
                | ApplicationOutcome::MakeKept
                | ApplicationOutcome::Follow(_)
                | ApplicationOutcome::MakeRefused(_) => {}
            }
            RecordKind::Application
        }
        Stored::ProposedEffect(value) => {
            repositories.push(value.entry.repository).expect("uncapped");
            RecordKind::ProposedEffect
        }
        Stored::Hold(value) => {
            repositories.push(repository(&value.name)).expect("uncapped");
            RecordKind::Hold
        }
        Stored::Subscription(value) => {
            match value.topic {
                Topic::Landings { repository, .. }
                | Topic::Ci { repository, .. }
                | Topic::Pull { repository, .. }
                | Topic::Participation { repository, .. } => {
                    repositories.push(repository).expect("uncapped");
                }
                Topic::Findings { .. } => {}
            }
            RecordKind::Subscription
        }
        Stored::Entry { entry, .. } => {
            repositories.push(entry.repository).expect("uncapped");
            RecordKind::Entry
        }
        Stored::Issue(value) => {
            repositories.push(value.repository).expect("uncapped");
            RecordKind::Issue
        }
        Stored::Release(value) => {
            if let Some((_, Some(name))) = value.pending {
                repositories.push(repository(&name)).expect("uncapped");
            }
            RecordKind::Release
        }
        Stored::Landing(value) => {
            for member in &value.landing.members {
                repositories
                    .push(Repository { forge: member.repository.forge, repository: member.repository.repository })
                    .expect("uncapped");
            }
            RecordKind::Landing
        }
        Stored::Follow(value) => {
            repositories.push(value.source).expect("uncapped");
            RecordKind::Follow
        }
        Stored::ChangeCold(value) => {
            repositories.push(value.repository).expect("uncapped");
            RecordKind::Change
        }
        Stored::Names { .. }
        | Stored::Call { .. }
        | Stored::Repository(_)
        | Stored::Landed { .. }
        | Stored::Closer(_)
        | Stored::Change(_)
        | Stored::ChangeHot(_)
        | Stored::Writer(_)
        | Stored::Binding { .. }
        | Stored::Setting(_)
        | Stored::RefusedSetting(_)
        | Stored::Flight { .. }
        | Stored::Forge(_)
        | Stored::ForgeNumber { .. }
        | Stored::Playground(_)
        | Stored::Making(_)
        | Stored::Dismissed { .. }
        | Stored::ProjectFacts(_)
        | Stored::Looked(_)
        | Stored::Away(_)
        | Stored::Decided(_)
        | Stored::Device(_)
        | Stored::Finding(_)
        | Stored::LandedDraft(_)
        | Stored::Red(_)
        | Stored::RedNumber { .. }
        | Stored::RedPass(_)
        | Stored::HistoryProject(_)
        | Stored::HistoryEvent(_)
        | Stored::HistoryChange(_)
        | Stored::HistoryLanding(_)
        | Stored::HistoryLast(_) => return Err(unreadable(&row.key, family)),
    };
    for repository in repositories.into_boxed() {
        if present.missing(repository, kind) {
            return match dropped {
                Some(count) => Ok(Judged::write(JobWrite::Erase { key: row.key.clone() }, count, 1)),
                None => Err(JobRefusal {
                    key: Some(row.key.clone()),
                    why: words(&[
                        family.as_bytes(),
                        b" names deleted repository ",
                        Decimal::of(u64::from(repository.forge)).as_bytes(),
                        b"/",
                        Decimal::of(u64::from(repository.repository)).as_bytes(),
                    ]),
                }),
            };
        }
    }
    Ok(Judged::keep())
}

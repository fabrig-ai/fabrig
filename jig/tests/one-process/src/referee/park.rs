//! Pairs the delegate's park on the agent pipe with its wake in the journal
//! (jig/docs/design/domain/testing-cells.md, 2, 4–5).
use super::agent::Frame;
use crate::{
    fixtures::{
        self,
        park::{CHILD_RESULT, NOTICE},
    },
    world::Proc,
};
use jig_conformance::referee::Observed;
use jig_core_kit::{
    cells::{self, Pairing, Park},
    parks::PARKS_ON_NOTHING,
};
use jig_domain_core::{self as core, CoreRecord};
use jig_domain_tasks::{Active, End, MessageKind, Party, Phase, Stored, TaskResult, Why};
use jig_test_domain as root;
use skein_fake_llm_domain::api::{Caching, Part, Query, Role};
use skein_fake_peers::llm::Observation;
use skein_lib::Time;
use skein_world::faults::{FaultLedger, Kind, Struck};
use std::collections::BTreeMap;

#[derive(Default)]
pub(super) struct Driver {
    pairing: Pairing,
    ledger: FaultLedger,
    child: Option<u64>,
    runs: BTreeMap<u64, u64>,
    child_identity: Option<[u8; 16]>,
    last_wait: Option<Box<[u8]>>,
    parked: Option<Park>,
    wake: Option<u64>,
    resumed: Option<(u64, usize)>,
}

type Rows = BTreeMap<root::Key, root::Record>;
// The native chat script opens the deployment's first task.
const CHAT: u64 = 1;

impl Driver {
    pub(super) fn overdue(&self, rows: &Rows) -> String {
        let terminals: Vec<_> = rows
            .values()
            .filter_map(|row| match row {
                root::Record::Core(core::Record::Core(CoreRecord::Terminal(terminal))) => {
                    Some((terminal.task, terminal.attempt, &terminal.end))
                }
                _ => None,
            })
            .collect();
        format!(
            "park story timed out: delegate {:?}, started {:?}, parked {:?}, wake {:?}, resumed {:?}, terminals {terminals:?}",
            self.child, self.runs, self.parked, self.wake, self.resumed
        )
    }

    pub(super) fn durable(&mut self, now: Time, rows: &Rows) -> Result<(), String> {
        if self.wake.is_some() {
            return Ok(());
        }
        let Some(child) = rows.values().find_map(|row| match row {
            root::Record::Core(core::Record::Tasks(Stored::Live(row))) if row.requester == Party::Task(CHAT) => {
                Some(row)
            }
            _ => None,
        }) else {
            return Ok(());
        };
        self.child = Some(child.number);
        let Why::Message { message } = child.due_reason else { return Ok(()) };
        let Some(notice) =
            child.inbox.iter().find(|word| word.number == message && word.kind == MessageKind::NothingAwaited)
        else {
            return Ok(());
        };
        let park = self.parked.ok_or("NothingAwaited woke a child without a parked agent answer")?;
        if (child.number, child.attempt) != (park.task, park.attempt)
            || !matches!(child.phase, Phase::Active(Active::Due | Active::Preparing))
            || notice.words.as_ref() != NOTICE
            || !notice.eligible
            || !matches!(rows.get(&root::Key::Core(core::Key::Core(core::CoreKey::Terminal {
                task: park.task, attempt: park.attempt,
            }))), Some(root::Record::Core(core::Record::Core(CoreRecord::Terminal(terminal))))
                if terminal.end == End::Parked)
        {
            return Err(format!("NothingAwaited did not wake {park:?} from its committed parked terminal"));
        }
        self.fell(now, park, PARKS_ON_NOTHING)?;
        self.wake = Some(message);
        Ok(())
    }

    pub(super) fn agent(
        &mut self,
        now: Time,
        frame: &Frame,
        rows: &Rows,
        provider_seen: usize,
    ) -> Result<Option<Observed>, String> {
        Ok(match frame {
            Frame::Start { attempt, identity, resumed, budget } => {
                let proof = rows
                    .values()
                    .find_map(|row| match row {
                        root::Record::Core(core::Record::Core(CoreRecord::RunProof(proof)))
                            if proof.attempt == *attempt =>
                        {
                            Some(proof)
                        }
                        _ => None,
                    })
                    .ok_or_else(|| format!("agent start {attempt} has no committed run proof"))?;
                let task = proof.task;
                self.runs.insert(*attempt, task);
                if Some(task) == self.child {
                    let identity = identity.ok_or("delegate Start has no conversation identity")?;
                    if *resumed {
                        let park = self.parked.ok_or("delegate resumed before it parked")?;
                        if self.child_identity != Some(identity)
                            || proof.conversation != park.attempt
                            || self.resumed.is_some()
                            || self.wake != proof.offered
                            || self.wake.is_none()
                        {
                            return Err(format!("delegate resumed without {park:?}'s committed wake and conversation"));
                        }
                        // Simulated time may not advance between requests. The provider's
                        // journal position splits the two attempts of this same conversation.
                        self.resumed = Some((*attempt, provider_seen));
                    } else if self.child_identity.replace(identity).is_some() || proof.offered.is_some() {
                        return Err(String::from("delegate must start once with no queued messages"));
                    }
                } else if *identity != Some(fixtures::RUN_ID) {
                    return Err(String::from("chat did not keep its conversation identity"));
                }
                Some(Observed::Assigned {
                    task,
                    attempt: *attempt,
                    conversation: proof.conversation,
                    resumed: *resumed,
                    budget: *budget,
                    settings: 1,
                })
            }
            Frame::Turn { attempt, turn } => {
                let task = *self.runs.get(attempt).ok_or("agent turn has no Start")?;
                if Some(task) == self.child && self.parked.is_none() {
                    self.last_wait = withheld(turn)?;
                }
                Some(Observed::Completion { task, attempt: *attempt, cumulative: turn.spent })
            }
            Frame::Parked { attempt } => {
                let task = *self.runs.get(attempt).ok_or("parked agent has no Start")?;
                if Some(task) == self.child {
                    let park = Park { task, attempt: *attempt, number: 1 };
                    if self.parked.is_some() || self.last_wait.is_none() {
                        return Err(format!("{park:?} was not the delegate's first park with its wait withheld"));
                    }
                    self.fell(now, park, smith_faults::parks::WITHHELD_WAIT)?;
                    self.parked = Some(park);
                }
                None
            }
        })
    }

    fn fell(&mut self, now: Time, park: Park, kind: Kind) -> Result<(), String> {
        let entry =
            self.ledger.inject(kind, now, Struck { request: Some(format!("task {}", park.task)), ..Struck::default() });
        self.ledger.stop(entry, now)?;
        self.pairing.fell(park, kind, now, &mut self.ledger)?;
        Ok(())
    }

    pub(super) fn reached(&self, procs: &[Proc], rows: &Rows) -> Result<bool, String> {
        if self.child.is_none() && finished(rows, CHAT, None, fixtures::ANSWER) {
            let answers: Vec<_> = observations(procs)?
                .iter()
                .flat_map(|seen| match seen {
                    Observation::Query { query, .. } => query.messages.as_ref(),
                    _ => &[],
                })
                .flat_map(|message| &message.parts)
                .filter_map(|part| match part {
                    Part::ToolOutput { output, .. } => Some(String::from_utf8_lossy(output)),
                    _ => None,
                })
                .collect();
            return Err(format!("chat finished without delegating: {answers:?}"));
        }
        let Some((attempt, before_resume)) = self.resumed else { return Ok(false) };
        let park = self.parked.ok_or("resumption without a park")?;
        if !finished(rows, park.task, Some(attempt), CHILD_RESULT) || !finished(rows, CHAT, None, fixtures::ANSWER) {
            return Ok(false);
        }
        let mut cells = self.ledger.entries().iter().filter(|entry| entry.kind.family == cells::FAMILY);
        let cell = cells.next().ok_or("no cell for the delegate's park")?;
        if cell.kind != cells::WITHHELD_WAIT_PARKS_ON_NOTHING
            || cell.struck.request.as_deref() != Some(&format!("task {}", park.task))
            || cells.next().is_some()
        {
            return Err(format!("{park:?}: expected exactly one withheld-wait/nothing-awaited cell"));
        }
        let id = self.last_wait.as_deref().ok_or("parked wait's call ID")?;
        let queries = observations(procs)?.iter().enumerate().filter_map(|(index, seen)| match seen {
            Observation::Query { query, .. } => Some((index, query)),
            _ => None,
        });
        let scope = Caching::Scope(self.child_identity.ok_or("delegate conversation identity")?);
        let mut resumed = false;
        for (index, query) in queries {
            let mut outputs = wait_outputs(query, id);
            if index >= before_resume && query.caching == scope && !resumed {
                if !matches!(outputs.next(), Some((false, words)) if skein_lib::bytes::find(words, NOTICE).is_some())
                    || outputs.next().is_some()
                    || notice_count(query) != 1
                {
                    return Err(format!(
                        "{park:?}: the resumed attempt's first request did not answer the wait once with NothingAwaited"
                    ));
                }
                resumed = true;
            } else if outputs.next().is_some() {
                return Err(format!("{park:?}: wait answered outside the delegate's first resumed request"));
            }
        }
        if !resumed {
            return Err(format!("{park:?}: no request for the resumed delegate"));
        }
        Ok(true)
    }
}

pub(super) fn observations(procs: &[Proc]) -> Result<&[Observation], String> {
    procs
        .iter()
        .find_map(|proc| match proc {
            Proc::Peer(peer) => Some(peer.observations()),
            _ => None,
        })
        .ok_or_else(|| String::from("park story's provider"))
}

fn finished(rows: &Rows, task: u64, attempt: Option<u64>, report: &[u8]) -> bool {
    rows.values().any(|row| matches!(row,
        root::Record::Core(core::Record::Core(CoreRecord::Terminal(terminal)))
        if terminal.task == task && attempt.is_none_or(|attempt| terminal.attempt == attempt)
            && matches!(&terminal.end, End::Finished { result: TaskResult::Report { words }, .. } if words.as_ref() == report)))
}

fn withheld(turn: &smith_channel::Turn) -> Result<Option<Box<[u8]>>, String> {
    use smith_domain::session::llm::Block;
    let agent = jig_channel_kit::fixtures::agent();
    let turn = smith_protocol_channel::decode_turn(&turn.body, turn.number, &agent.limits.transcript, &agent.endpoints)
        .map_err(|why| format!("delegate turn bytes: {why:?}"))?;
    let mut blocks = turn.messages.iter().flat_map(|message| &message.content);
    let Some(call) = blocks.clone().filter_map(Block::tool_call).find(|call| call.name.as_ref() == b"wait") else {
        return Ok(None);
    };
    Ok((!blocks.any(|block| matches!(block, Block::ToolResult { id, .. } if *id == call.id))).then(|| call.id.clone()))
}

fn wait_outputs<'a>(query: &'a Query, wait: &'a [u8]) -> impl Iterator<Item = (bool, &'a [u8])> {
    query.messages.iter().filter(|message| message.role == Role::User).flat_map(|message| &message.parts).filter_map(
        move |part| match part {
            Part::ToolOutput { id, is_error, output } if id.as_ref() == wait => Some((*is_error, output.as_ref())),
            _ => None,
        },
    )
}

fn notice_count(query: &Query) -> usize {
    query
        .messages
        .iter()
        .flat_map(|message| &message.parts)
        .filter_map(|part| match part {
            Part::Text { text } | Part::ToolOutput { output: text, .. } => Some(text),
            _ => None,
        })
        .map(|text| text.windows(NOTICE.len()).filter(|part| *part == NOTICE).count())
        .sum()
}

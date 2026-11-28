//! The worker's actual agent pipe bytes, decoded independently of its root.
//! Agent ACKs attest hub retention, not engine durability (domain/hosts/hub.md, 4).
use jig_conformance::referee::Observed;
use skein_io::kernel::{Complete, Done, Op};
use skein_lib::Reader;

pub(super) enum Frame {
    Start { attempt: u64, identity: Option<[u8; 16]>, resumed: bool, budget: u64 },
    Turn { attempt: u64, turn: smith_channel::Turn },
    Parked { attempt: u64 },
}
use smith_channel::Record;
use std::collections::BTreeMap;

#[derive(Default)]
pub(crate) struct Wire {
    pipes: BTreeMap<i32, (bool, Vec<u8>)>,
    attempt: u64,
    pub events: Vec<Observed>,
    pub(super) park_frames: Option<Vec<Frame>>,
    pub answers: Vec<smith_channel::RunResult>,
    pub mode: Option<smith_charter::Mode>,
}
impl Wire {
    pub(crate) fn for_park() -> Self {
        Self { park_frames: Some(Vec::new()), ..Self::default() }
    }

    pub fn complete(&mut self, complete: &Complete) {
        match (&complete.kind, &complete.result) {
            (Op::Spawn { spawn }, Ok(Done::Spawned { .. })) => {
                for pipe in spawn.pipes.iter().filter(|pipe| pipe.child < 2) {
                    self.pipes.insert(pipe.parent.expect("real agent pipe").raw(), (pipe.child == 0, Vec::new()));
                }
            }
            (Op::Write { fd, bytes, from, .. } | Op::PipeWrite { fd, bytes, from }, Ok(Done::Count(count))) => {
                self.bytes(fd.raw(), &bytes[*from as usize..(*from + *count) as usize]);
            }
            (Op::Read { fd, buf, .. } | Op::PipeRead { fd, buf }, Ok(Done::Count(count))) => {
                self.bytes(fd.raw(), &buf[..*count as usize]);
            }
            (Op::Close { fd }, _) => {
                self.pipes.remove(&fd.raw());
            }
            _ => {}
        }
    }
    fn bytes(&mut self, fd: i32, bytes: &[u8]) {
        let Some((down, buffer)) = self.pipes.get_mut(&fd) else { return };
        buffer.extend_from_slice(bytes);
        let mut frames = Vec::new();
        while buffer.len() >= 8 {
            let kind = u16::from_be_bytes(buffer[..2].try_into().expect("frame kind"));
            assert_eq!(&buffer[2..4], &[0, 0], "agent frame flags");
            let length = u32::from_be_bytes(buffer[4..8].try_into().expect("frame length")) as usize;
            if buffer.len() < 8 + length {
                break;
            }
            frames.push((*down, kind, buffer[8..8 + length].to_vec()));
            buffer.drain(..8 + length);
        }
        for (down, kind, body) in frames {
            self.frame(down, kind, &body);
        }
    }
    fn frame(&mut self, down: bool, kind: u16, bytes: &[u8]) {
        let limits = crate::fixtures::agent_limits();
        if down && kind == 0x0100 {
            let start = smith_channel::Start::decode(&limits, &mut Reader::new(bytes)).expect("agent Start bytes");
            let charter = smith_charter::Charter::decode(&smith_charter::CEILINGS, &mut Reader::new(&start.charter))
                .expect("agent charter bytes");
            self.mode = Some(charter.mode.clone());
            self.attempt = start.activation;
            if let Some(frames) = &mut self.park_frames {
                frames.push(Frame::Start {
                    attempt: start.activation,
                    identity: start.identity,
                    resumed: !start.transcript.is_empty(),
                    budget: charter.budget.spend,
                });
                return;
            }
            assert_eq!(start.identity, Some(crate::fixtures::RUN_ID), "logical chat identity");
            self.events.push(Observed::Assigned {
                task: 1,
                attempt: self.attempt,
                conversation: 1,
                resumed: !start.transcript.is_empty(),
                budget: charter.budget.spend,
                // These byte-level scenarios use fixed first-start settings.
                settings: 1,
            });
        } else if !down && kind == 0x0110 {
            let answer = smith_channel::Answer::decode(&limits, &mut Reader::new(bytes)).expect("agent Answer bytes");
            if let Some(frames) = &mut self.park_frames
                && answer.result == smith_channel::RunResult::Parked
            {
                frames.push(Frame::Parked { attempt: self.attempt });
            }
            self.answers.push(answer.result);
        } else if !down && kind == 0x0109 {
            let turn = smith_channel::Turn::decode(&limits, &mut Reader::new(bytes)).expect("agent Turn bytes");
            if let Some(frames) = &mut self.park_frames {
                frames.push(Frame::Turn { attempt: self.attempt, turn });
            } else {
                self.events.push(Observed::Completion { task: 1, attempt: self.attempt, cumulative: turn.spent });
            }
        }
    }
}

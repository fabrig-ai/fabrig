//! Host handoffs and opaque settled answers (domain/engine/executors.md, 2–3;
//! domain/hosts/README.md, 2). Protocol decoding and rendering remain with the root.

pub use jig_domain_core_vocabulary::runs::conversation::*;

use crate::{CallKey, CallRecord, Core, CoreRecord, Held, Limits, Now, Record, Request, Write};
use alloc::boxed::Box;
use skein_lib::{Queue, ReplyTo};

impl Core {
    /// The root wraps this whole core decision, including settled host evidence.
    #[must_use]
    pub fn call_record(&self, key: CallKey) -> Option<CallRecord> {
        Some(CallRecord {
            key,
            part: self.call_parts.get(&key)?.clone(),
            settled: self.call_settled.get(&key).cloned(),
        })
    }
    /// Settled answers after the last committed turn, in their commit order.
    #[must_use]
    pub fn settled_calls(&self, task: u64, attempt: u64) -> Box<[Settled]> {
        let mut rows = skein_lib::List::with_capacity(self.call_settled.len());
        let mut after = 0;
        for _ in 0..self.call_settled.len() {
            let mut next: Option<(CallKey, &SettledCall)> = None;
            for (key, call) in &self.call_settled {
                if key.task == task
                    && key.attempt < attempt
                    && call.serial > after
                    && match next {
                        Some((_, old)) => call.serial < old.serial,
                        None => true,
                    }
                {
                    next = Some((*key, call));
                }
            }
            let Some((key, call)) = next else { break };
            after = call.serial;
            rows.push(Settled { key, call: call.clone() }).expect("settled keys bound their answers");
        }
        rows.into_boxed()
    }
}

pub(crate) fn settle(
    core: &mut Core,
    limits: &Limits,
    to: ReplyTo,
    key: CallKey,
    name: Box<[u8]>,
    mut call: SettledCall,
) -> Queue<Request> {
    let mut out = Queue::uncapped();
    if !call.valid(limits) || !core.call_parts.contains_key(&key) {
        out.push(Request::Now(Box::new(Now::SettledCallRefused { to, key, name, tool: call.tool })));
    } else {
        let settled = match core.call_settled.get(&key) {
            Some(old) => old.clone(),
            None => {
                let Some(serial) = crate::fresh(&mut core.counters, crate::Family::Call) else {
                    out.push(Request::Now(Box::new(Now::SettledCallRefused { to, key, name, tool: call.tool })));
                    return out;
                };
                call.serial = serial;
                assert!(core.call_settled.insert(key, call.clone()).is_ok(), "settled calls fit their retained names");
                out.push(Request::Write(Write::Save(Record::Core(CoreRecord::Call(
                    core.call_record(key).expect("settled named call"),
                )))));
                call
            }
        };
        out.push(Request::Held(Box::new(Held::SettledCall { to, key, call: settled })));
    }
    out
}

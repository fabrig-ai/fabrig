//! Root output routing (domain/host.md, section 9.4).
//! Retains each slot's turn and call ledgers and emits parent notices or lower
//! requests. No codec, policy interpretation or credential values are known.
use crate::domain::{Domain, Relay, Slot};
use crate::{Limits, Lower, Output, stop};
use alloc::boxed::Box;
use skein_lib::{Env, Id, Queue};
use smith_domain as smith;
use smith_host_domain::{self as host, parent};

#[expect(clippy::too_many_lines, reason = "every root request has one typed parent or lower route")]
pub(crate) fn route(agent: &mut Domain, env: &Env<Limits>, id: Id<Slot>, out: &mut Queue<Output>) {
    for _ in 0..agent.lower.len() {
        let Some(request) = agent.lower.pop() else {
            break;
        };
        let slot = agent.slots.get_mut(id).expect("root output belongs to its slot");
        let client = slot.client;
        match request {
            smith::Request::Admitted { host_run, run } => {
                assert_eq!(slot.logical_run, host_run, "root output preserves its scoped identity and single right");
                slot.run = Some(run);
                out.push(Output::Parent(parent::Request::Admitted { client }));
            }
            smith::Request::MessageRefused { host_run, name, reason } => {
                assert_eq!(slot.logical_run, host_run, "root output preserves its scoped identity and single right");
                if slot.inbox.refused(name) {
                    out.push(Output::Parent(parent::Request::MessageRefused { client, name, reason }));
                }
            }
            smith::Request::Turn { host_run, number, spent, read, turn, opens_window, position: _ } => {
                assert_eq!(slot.logical_run, host_run, "root output preserves its scoped identity and single right");
                slot.inbox.fence(read);
                let body = host::TurnValue::new(turn, u64::MAX).expect("root concrete turns have checked ownership");
                assert!(
                    slot.turns.insert(number).expect("root respects the acknowledgement window"),
                    "root output preserves its scoped identity and single right"
                );
                out.push(Output::Parent(parent::Request::Turn {
                    client,
                    turn: host::Turn { number, spent: spent.units, read, opens_window, body },
                }));
            }
            smith::Request::Waiting { host_run, read } => {
                assert_eq!(slot.logical_run, host_run, "root output preserves its scoped identity and single right");
                slot.inbox.fence(read);
                out.push(Output::Parent(parent::Request::Waiting { client, read }));
            }
            smith::Request::Answer { to, answer, read, detail } => {
                assert_eq!(to.into_token(), client, "root output preserves its scoped identity and single right");
                slot.inbox.fence(read);
                out.push(Output::Parent(parent::Request::Answered {
                    client,
                    answer: crate::translate::answer(answer, read, env.now),
                    detail,
                }));
                if let Some(names) = slot.inbox.report() {
                    out.push(Output::Parent(parent::Request::Unread { client, names }));
                }
                stop::finish(agent, env.now, id, out);
            }
            smith::Request::Complete {
                owner,
                name,
                grant,
                prompt,
                deadlines,
                max_completion_bytes,
                max_completion_blocks,
                max_failure_bytes,
                decoded_call_bytes,
            } => {
                assert!(
                    slot.completions.insert(owner, false).expect("root bounds live completions").is_none(),
                    "root output preserves its scoped identity and single right"
                );
                out.push(Output::Lower {
                    agent: id.token(),
                    request: Lower::Complete {
                        owner,
                        name,
                        grant,
                        prompt,
                        deadlines,
                        max_completion_bytes,
                        max_completion_blocks,
                        max_failure_bytes,
                        decoded_call_bytes,
                    },
                });
            }
            smith::Request::Yield { .. } => {}
            smith::Request::Cancel { owner } => match slot.completions.get_mut(&owner) {
                Some(cancelled) => {
                    if !*cancelled {
                        *cancelled = true;
                        out.push(Output::Lower { agent: id.token(), request: Lower::Cancel { owner } });
                    }
                }
                None => unreachable!("cancel retains a live provider right"),
            },
            smith::Request::HostCall { host_run, relay, name, tool, effect, input, deadline } => {
                assert_eq!(slot.logical_run, host_run, "root output preserves its scoped identity and single right");
                let call = crate::workspace::call(slot);
                assert!(
                    slot.relays
                        .insert(call, Relay { name: crate::workspace::Call::Host(relay), withdrawn: false })
                        .expect("root bounds its host rights")
                        .is_none(),
                    "root output preserves its scoped identity and single right"
                );
                out.push(Output::Parent(parent::Request::Called {
                    client,
                    logical_run: host_run,
                    call,
                    name: host::CallName {
                        activation: name.activation,
                        completion: name.completion,
                        position: name.position,
                    },
                    deadline,
                    ask: host::Ask::Host { tool, effect, body: Box::from(input.bytes()) },
                }));
            }
            smith::Request::WithdrawHost { relay } => {
                let mut found = None;
                for (call, pending) in &slot.relays {
                    match pending.name {
                        crate::workspace::Call::Host(name) => {
                            if name == relay {
                                found = Some(*call);
                                break;
                            }
                        }
                        crate::workspace::Call::Delivery(_) => {}
                    }
                }
                if let Some(call) = found {
                    let pending = slot.relays.get_mut(&call).expect("retained relay name");
                    if !pending.withdrawn {
                        pending.withdrawn = true;
                        out.push(Output::Parent(parent::Request::Withdrawn { client, call }));
                    }
                }
            }
            smith::Request::Unauthorized { grant } => out.push(Output::Parent(parent::Request::Unauthorized {
                client,
                account: grant.account,
                generation: grant.generation,
            })),
            smith::Request::Limited { account, reset } => {
                out.push(Output::Parent(parent::Request::Limited { client, account, reset }));
            }
            smith::Request::Refused { account, code, reason } => {
                out.push(Output::Parent(parent::Request::Refused { client, account, code, reason }));
            }
            smith::Request::Long { host_run, operation, until } => {
                assert_eq!(slot.logical_run, host_run, "root workspace requests preserve logical scope");
                out.push(Output::Parent(parent::Request::Long {
                    client,
                    operation,
                    span: until.saturating_since(env.now),
                }));
            }
            smith::Request::LongDone { host_run } => {
                assert_eq!(slot.logical_run, host_run, "root workspace requests preserve logical scope");
                out.push(Output::Parent(parent::Request::LongDone { client }));
            }
            smith::Request::Deliver { host_run, owner, change, name, deadline } => {
                assert_eq!(slot.logical_run, host_run, "root workspace requests preserve logical scope");
                let call = crate::workspace::call(slot);
                assert!(
                    slot.relays
                        .insert(call, Relay { name: crate::workspace::Call::Delivery(owner), withdrawn: false })
                        .expect("bounded parent rights")
                        .is_none(),
                    "each delivery owns one unique parent call"
                );
                out.push(Output::Parent(parent::Request::Called {
                    client,
                    logical_run: host_run,
                    call,
                    name: host::CallName {
                        activation: name.activation,
                        completion: name.completion,
                        position: name.position,
                    },
                    deadline,
                    ask: host::Ask::Deliver {
                        fields: host::Fields::new(change, u64::MAX).expect("root bounds checked fields"),
                    },
                }));
            }
            smith::Request::Io { owner, op, deadline } => {
                crate::workspace::io(slot, id.token(), owner, op, deadline, out);
            }
            smith::Request::Job { owner, act } => crate::workspace::job(slot, id.token(), owner, act, out),
            smith::Request::CancelIo { owner } => crate::workspace::cancel_io(slot, id.token(), owner, out),
            smith::Request::Read { owner, at, max, deadline } => {
                crate::workspace::read(slot, id.token(), owner, at, max, deadline, out);
            }
            smith::Request::Probe { owner, at, deadline } => {
                crate::workspace::probe(slot, id.token(), owner, at, deadline, out);
            }
            smith::Request::Check { owner, program, deadline, tail } => {
                crate::workspace::check(slot, id.token(), owner, program, deadline, tail, out);
            }
            smith::Request::Abort { owner } => crate::workspace::abort(slot, id.token(), owner, out),
        }
    }
    assert!(agent.lower.is_empty(), "each root's requests are routed to its own slot before the next entrance");
}

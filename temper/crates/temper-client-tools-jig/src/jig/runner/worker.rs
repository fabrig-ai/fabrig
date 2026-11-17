//! One worker setting per act (client/ctl/workers.md, 2).
use super::{Out, Runner, acts, reads::Reading};
use crate::jig::{
    Limits,
    call::{Call, Value, number, value},
    render, rows,
};
use alloc::boxed::Box;
use jig_client::{Client, Readout, contract as c};
use skein_lib::{Env, List, Queue};

/// Read the revision first: a settings change during either later step must be stale.
pub(super) fn needs_deployment(reading: &Reading) -> bool {
    if !(rows::WORKER_ADD_AT..=rows::WORKER_REMOVE_AT).contains(&reading.called.call.tool)
        || reading.pages.contains_key(&1)
    {
        return false;
    }
    let Some(Readout::Jig(c::ReadResult::Value(c::ReadValue::Settings(_)))) = reading.pages.get(&2) else {
        return false;
    };
    true
}

pub(super) fn resolved(
    runner: &mut Runner,
    client: &Client,
    env: &Env<Limits>,
    reading: Reading,
    out: &mut Queue<Out>,
) {
    for (local, page) in &reading.pages {
        match (local, page) {
            (1, Readout::Jig(c::ReadResult::Value(c::ReadValue::Deployment(_))))
            | (2, Readout::Jig(c::ReadResult::Value(c::ReadValue::Settings(_)))) => {}
            (_, Readout::Jig(c::ReadResult::Value(_))) => {
                let printed =
                    render::failure(b"tools' bug; unexpected document; report it as a finding", reading.called.json, 1);
                super::finish(reading.called, printed, out);
                return;
            }
            (_, Readout::Jig(_) | Readout::Limit | Readout::NotServed { .. } | Readout::Family { .. }) => {
                let printed = render::read(page, client, runner.me(), reading.called.json, &env.limits, reading.parts);
                super::finish(reading.called, printed, out);
                return;
            }
        }
    }
    let Some(Readout::Jig(c::ReadResult::Value(c::ReadValue::Deployment(deployment)))) = reading.pages.get(&1) else {
        unreachable!("worker deployment read")
    };
    let Some(Readout::Jig(c::ReadResult::Value(c::ReadValue::Settings(settings)))) = reading.pages.get(&2) else {
        unreachable!("worker settings read")
    };
    let request = request(&reading.called.call, deployment, settings.newest);
    acts::prepared(runner, client, env, reading.called, request, out);
}

fn request(call: &Call, deployment: &c::Deployment, against: u32) -> acts::Prepared {
    let Some(Value::Text(name)) = value(call, 0) else { unreachable!("worker name argument") };
    let picked = c::worker_number(deployment.worker_settings.as_slice(), name);
    let number = match (call.tool, picked) {
        (rows::WORKER_ADD_AT, c::WorkerNumber::Live(_)) => return Err(local_refusal(name, true)),
        (rows::WORKER_ADD_AT, c::WorkerNumber::Withdrawn(number) | c::WorkerNumber::Free(number))
        | (rows::WORKER_PLACES_AT | rows::WORKER_REMOVE_AT, c::WorkerNumber::Live(number)) => number,
        (rows::WORKER_PLACES_AT | rows::WORKER_REMOVE_AT, c::WorkerNumber::Withdrawn(_) | c::WorkerNumber::Free(_)) => {
            return Err(local_refusal(name, false));
        }
        _ => unreachable!("worker act"),
    };
    let edit = match call.tool {
        rows::WORKER_ADD_AT => {
            let Some(Value::Sha256(digest)) = value(call, 2) else { unreachable!("worker SHA-256 argument") };
            c::SettingEdit::Set(c::SettingSet {
                value: c::SettingValue::Worker(Box::new(c::WorkerSetting {
                    number,
                    name: name.clone(),
                    places: places(call),
                    digest: Some(*digest),
                    withdrawn: false,
                })),
            })
        }
        rows::WORKER_PLACES_AT => {
            let mut previous = None;
            for worker in &deployment.worker_settings {
                if worker.number == number {
                    previous = Some(worker);
                    break;
                }
            }
            let worker = previous.expect("worker_number selected a live row");
            c::SettingEdit::Set(c::SettingSet {
                value: c::SettingValue::Worker(Box::new(c::WorkerSetting {
                    number,
                    name: worker.name.clone(),
                    places: places(call),
                    digest: worker.digest,
                    withdrawn: false,
                })),
            })
        }
        rows::WORKER_REMOVE_AT => c::SettingEdit::Removed,
        _ => unreachable!("worker act"),
    };
    let mut items = List::with_capacity(1);
    items
        .push(c::SettingItem { setting: c::Setting::Worker(c::SettingNumber { number }), edit })
        .expect("one worker item");
    Ok((
        acts::Request::Ask(c::Ask::ChangeSettings(c::ChangeSettings { against, items, running: c::Running::Stays })),
        None,
    ))
}
fn places(call: &Call) -> u32 {
    u32::try_from(number(call, 1).expect("worker places argument")).expect("places fit u32")
}
fn local_refusal(name: &[u8], exists: bool) -> acts::Failed {
    let mut measure = render::Text::measure();
    refusal_text(name, exists, &mut measure);
    let mut out = render::Text::writing_exact(measure);
    refusal_text(name, exists, &mut out);
    acts::Failed::RefusedDetail(out.finish())
}
fn refusal_text(name: &[u8], exists: bool, out: &mut render::Text) {
    out.put(if exists { b"worker " } else { b"no worker " });
    out.put(name);
    if exists {
        out.put(b" exists; `worker places` changes its places");
    }
}

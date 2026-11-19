//! A gone change's last events (temper/docs/design/domain/landing-history.md, 5.2).
use super::{Watching, handle};
use crate::{
    jig::{
        self,
        render::{self, Text},
        runner::Out,
    },
    landing::history,
    root::Limits,
};
use alloc::boxed::Box;
use jig_client::{Client, Event, Readable, Readout, contract as c};
use skein_json::writer::{Encoder, Limits as JsonLimits};
use skein_lib::{Duration, Env, Queue, Time};
use temper_client_protocol_wire::v1 as w;

#[derive(Debug)]
pub(super) struct Reading {
    pages: history::ChangeHistory,
    read: Readable,
    until: Time,
}
pub(super) fn start(watch: &mut Watching, env: &Env<Limits>, out: &mut Queue<Out>) {
    let (pages, read) = history::read_change_history(watch.project.expect("resolved project"), watch.task);
    watch.history = Some(Reading { pages, read, until: env.now.saturating_add(env.limits.tools.wait) });
    send(watch, out);
}
pub(super) fn send(watch: &mut Watching, out: &mut Queue<Out>) {
    let handle = handle(watch);
    watch.read = super::Reading::Pending(handle);
    let read = watch.history.as_ref().expect("history in progress").read.clone();
    out.push(Out::Client { event: Event::Read { handle, read } });
}
pub(super) fn answer(
    watch: &mut Watching,
    page: &Readout,
    client: &Client,
    env: &Env<Limits>,
    out: &mut Queue<Out>,
) -> Option<jig::Printed> {
    let reading = watch.history.as_mut().expect("history in progress");
    let reply = history::decode(page, &env.limits);
    if reply == Some(w::HistoryReply::Busy) || *page == Readout::Jig(c::ReadResult::None(c::ReadNone::Busy)) {
        if env.now < reading.until {
            watch.read = super::Reading::Retry(env.now.saturating_add(Duration::from_secs(1)).min(reading.until));
            return None;
        }
        return Some(render::failure(b"busy", watch.called.json, 1));
    }
    match reply {
        Some(w::HistoryReply::Page(page)) => {
            let more = page.after().is_some();
            if let Some(read) = reading.pages.accept(page) {
                reading.read = read;
                send(watch, out);
                None
            } else {
                Some(print(watch, more, &env.limits))
            }
        }
        Some(w::HistoryReply::Gone(gone)) => Some(reading.pages.gone(&gone, watch.called.json)),
        Some(w::HistoryReply::Busy) => unreachable!("busy read retried"),
        None => Some(render::read(
            page,
            client,
            jig_client::Handle { asker: crate::root::RUNNER.asker(), number: 0 },
            watch.called.json,
            &env.limits.tools,
            &crate::root::Parts,
        )),
    }
}
fn command(watch: &Watching, out: &mut Text) {
    out.put(b"temper ctl landing history --project ");
    out.number(u64::from(watch.project.expect("resolved project")));
    out.put(b" --change ");
    out.number(watch.task);
}
fn text(watch: &Watching, more: bool, out: &mut Text) {
    let events = watch.history.as_ref().expect("history read").pages.events();
    out.put(b"change ");
    out.number(watch.task);
    if events.is_empty() && !more {
        out.put(b": its record and its history have gone\n");
        return;
    }
    out.put(b": its record has gone; its last events:\n");
    for event in events.iter().skip(usize::try_from(events.len().saturating_sub(8)).expect("bounded history")) {
        history::event_line(event, out);
    }
    if more {
        out.put(b"more events remain\n");
    }
    out.put(b"all of them: ");
    command(watch, out);
    out.put(b"\n");
}
fn json(watch: &Watching, more: bool, out: &mut Encoder) {
    let events = watch.history.as_ref().expect("history read").pages.events();
    out.object_start();
    out.key(b"task");
    out.unsigned(watch.task);
    out.key(b"record");
    out.string(b"gone");
    if events.is_empty() && !more {
        out.key(b"history");
        out.string(b"gone");
    } else {
        out.key(b"events");
        out.array_start();
        for event in events.iter().skip(usize::try_from(events.len().saturating_sub(8)).expect("bounded history")) {
            history::event_json(event, out);
        }
        out.array_end();
        out.key(b"read");
        let mut measure = Text::measure();
        command(watch, &mut measure);
        let mut read = Text::writing_exact(measure);
        command(watch, &mut read);
        out.string(&read.finish());
    }
    out.object_end();
}
fn print(watch: &Watching, more: bool, limits: &Limits) -> jig::Printed {
    let stdout = if watch.called.json {
        let bounds = JsonLimits { depth: 12, length: u32::try_from(limits.tools.result_bytes).expect("result bound") };
        let mut measure = Encoder::measure(&bounds);
        json(watch, more, &mut measure);
        let Ok(size) = measure.measured() else { return render::failure(b"result limit", true, 1) };
        let mut out = Encoder::write(size, &bounds);
        json(watch, more, &mut out);
        out.finish()
    } else {
        let mut measure = Text::measure();
        text(watch, more, &mut measure);
        if measure.structural() > usize::try_from(limits.tools.text).expect("text bound") {
            return render::failure(b"result limit", false, 1);
        }
        let mut out = Text::writing_exact(measure);
        text(watch, more, &mut out);
        out.finish()
    };
    let gone = watch.history.as_ref().expect("history read").pages.events().is_empty() && !more;
    jig::Printed { file: jig::FileEnd::Keep, status: if gone { 2 } else { 0 }, stdout, stderr: Box::default() }
}

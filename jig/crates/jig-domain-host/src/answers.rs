//! Answers retained on their run slots until acknowledged (domain/hosts/hub.md, 2).
use crate::domain::Hub;
use crate::{Bounds, Hello, Hosting, LinkUp, Phase, Request, RunAnswer};
use alloc::boxed::Box;
use skein_lib::{Duration, List, Queue, Token};

#[derive(Debug)]
pub(crate) enum Kept {
    Run(RunAnswer),
    Job(crate::JobEnding),
}

pub(crate) fn kept(domain: &Hub) -> u32 {
    domain.answers.len()
}

fn retained(domain: &Hub, index: u32) -> Option<&crate::hosted::Hosted> {
    for (at, (_, id)) in domain.answers.iter().enumerate() {
        if u32::try_from(at).expect("bounded answer slots") == index {
            return domain.hosted.get(*id);
        }
    }
    None
}

pub(crate) fn at(domain: &Hub, index: u32) -> Option<LinkUp> {
    let run = retained(domain, index)?;
    Some(match run.answer.as_ref().expect("indexed answer") {
        Kept::Run(answer) => LinkUp::Answer { run: run.run, attempt: run.attempt, answer: answer.clone() },
        Kept::Job(answer) => LinkUp::JobAnswer { run: run.run, attempt: run.attempt, answer: answer.clone() },
    })
}

pub(crate) fn guides(run: &crate::hosted::Hosted) -> Option<LinkUp> {
    let bytes = run.guides.as_ref()?.clone();
    Some(LinkUp::Guides { run: run.run, attempt: run.attempt, bytes })
}

pub(crate) fn guides_at(domain: &Hub, index: u32) -> Option<LinkUp> {
    guides(retained(domain, index)?)
}

pub(crate) fn acknowledge(domain: &mut Hub, run: Token, attempt: Token) {
    if let Some(id) = domain.answers.remove(&(run, attempt)) {
        let entry = domain.hosted.get_mut(id).expect("retained answer");
        entry.answer = None;
        entry.guides = None;
        domain.hosted.retire(id);
    }
}

pub(crate) fn opened(
    domain: &Hub,
    slots: u32,
    stop_bound: Duration,
    workstreams: Box<[u64]>,
    bounds: Bounds,
    out: &mut Queue<Request>,
) {
    let mut hosting = List::with_capacity(domain.hosted.len());
    for (_, id) in &domain.names {
        let entry = crate::hosted::hosting(domain, id.token()).expect("named run");
        hosting.push(entry).expect("run slots bound the hello");
    }
    for (_, id) in &domain.answers {
        let run = domain.hosted.get(*id).expect("retained answer slot");
        if run.answer.is_some() {
            hosting
                .push(Hosting { run: run.run, attempt: run.attempt, phase: Phase::Answered })
                .expect("answer retains its slot");
        }
    }
    out.push(Request::Hello {
        hello: Hello {
            // The worker link sets process freshness when it sends the hello.
            fresh: false,
            reads_usage: true,
            slots,
            stop_bound,
            workstreams,
            hosting: hosting.into_boxed(),
            bounds,
            jobs: domain.job_kinds.clone(),
        },
    });
}

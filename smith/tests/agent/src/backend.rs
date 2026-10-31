//! Provider faces driven by the agent world's single loop (domain/run.md, section 13).
use crate::{Settings, translate};
use skein_fake_llm_domain as provider;
use skein_lib::{ReplyTo, Time, Token, Wall};
use skein_world::domain::{Ledger, Stage};
use smith_domain::{Event, Request, llm, tools};

/// A provider face under the shared agent loop (testing-strategy.md, section 2).
/// Implementations own lower progression and return only boundary observations.
pub trait Backend: core::fmt::Debug {
    /// Admit the root's complete request without reconstructing its contract.
    fn start(&mut self, request: Request, now: Time, wall: skein_lib::Wall) -> Vec<provider::api::Query>;
    /// Advance lower work once under the world's clocks.
    fn advance(&mut self, now: Time, wall: skein_lib::Wall, finished: bool) -> Progress;
    /// Return true when the backend owes the cancellation terminal itself.
    fn cancel(&mut self, owner: Token, now: Time, wall: skein_lib::Wall) -> bool;
    /// Whether another round can make progress without advancing time.
    fn has_events(&self) -> bool;
    /// Whether all provider calls and physical resources have settled.
    fn is_settled(&self) -> bool;
    /// The next lower deadline.
    fn next_deadline(&self) -> Option<Time>;
    /// Verify all lower callback rights have been consumed.
    fn assert_settled(&self);
}

/// Boundary observations from one lower progression round.
#[derive(Debug, Default)]
pub struct Progress {
    /// Queries independently seen by the fake provider.
    pub queries: Vec<provider::api::Query>,
    /// One terminal per admitted completion owner.
    pub terminals: Vec<(Token, Event)>,
    /// More lower work is ready under the same clocks.
    pub immediate: bool,
    /// Typed neighbour replies traverse the fixture network schedule.
    pub delay_terminals: bool,
}

/// The domain-only scripted provider, with finite failure injection.
#[derive(Debug)]
pub struct Typed {
    pub(crate) provider: provider::Domain,
    stage: Stage<provider::Config, provider::Event, provider::Request>,
    pub(crate) failure: Option<(llm::Failure, llm::Evidence)>,
    pub(crate) failure_report: llm::Report,
    pub(crate) failure_count: Option<u32>,
    calls: Ledger<Token, (tools::Grants, Box<[llm::Served]>, llm::Asker)>,
}

impl Typed {
    pub(crate) fn new(settings: &Settings) -> Self {
        Self {
            provider: crate::scripted_provider(&settings.provider, settings.seed ^ 0x25),
            stage: Stage::uncapped(settings.provider),
            failure: None,
            failure_report: llm::Report::none(),
            failure_count: None,
            calls: Ledger::new("fake provider call"),
        }
    }
}
impl Backend for Typed {
    fn start(&mut self, request: Request, now: Time, wall: Wall) -> Vec<provider::api::Query> {
        let Request::Complete { owner, prompt, .. } = request else { panic!("backend accepts Complete") };
        self.stage.tick(now);
        self.stage.env.wall = wall;
        self.calls.open(owner, (prompt.tools, prompt.served.clone(), prompt.asker));
        let query = translate::query(prompt);
        self.stage.push(provider::Event::Call { reply_to: ReplyTo::new(owner), call: owner, query: query.clone() });
        vec![query]
    }

    fn advance(&mut self, now: Time, wall: Wall, _finished: bool) -> Progress {
        self.stage.tick(now);
        self.stage.env.wall = wall;
        while let Some(event) = self.stage.next_event() {
            if matches!(&event, provider::Event::Call { .. })
                && let Some(remaining) = self.failure_count.as_mut()
            {
                self.stage.env.limits.overloaded = if *remaining > 0 { 1000 } else { 0 };
                *remaining = remaining.saturating_sub(1);
            }
            provider::step(&mut self.provider, &self.stage.env, event, &mut self.stage.out);
        }
        while self.provider.is_due(now) {
            provider::fire(&mut self.provider, &self.stage.env, &mut self.stage.out);
        }
        let mut progress = Progress { delay_terminals: true, ..Progress::default() };
        while let Some(request) = self.stage.out.pop() {
            let provider::Request::Reply { to, result } = request else { panic!("typed provider returns a terminal") };
            let owner = to.into_token();
            let (grants, served, asker) = self.calls.end(owner);
            let event = match result {
                Ok(answer) => {
                    Event::Completed { owner, completion: translate::completion(answer, grants, &served, asker) }
                }
                Err(error) => {
                    let (failure, evidence) =
                        self.failure.unwrap_or((translate::failure(error.error), llm::Evidence::Unknown));
                    Event::Failed {
                        owner,
                        failure,
                        evidence,
                        detail: Box::default(),
                        report: self.failure_report.clone(),
                    }
                }
            };
            progress.terminals.push((owner, event));
        }
        self.provider.reclaim();
        progress
    }
    fn cancel(&mut self, _owner: Token, _now: Time, _wall: Wall) -> bool {
        false
    }
    fn has_events(&self) -> bool {
        self.stage.has_events()
    }
    fn is_settled(&self) -> bool {
        self.provider.calls() == 0
    }
    fn next_deadline(&self) -> Option<Time> {
        self.provider.next_deadline()
    }
    fn assert_settled(&self) {
        self.calls.assert_settled();
    }
}

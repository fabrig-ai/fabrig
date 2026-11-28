//! Connector judgements live only through the decision's hand-offs
//! (jig/docs/design/domain/engine/vocabulary.md, 2.5).

use crate::{Ask, Request};
use jig_domain_contract::{
    self as contract, effects, lifecycle, parts, procedures, projections, requests, resources, settings, topics,
};
use skein_lib::{List, Queue, Token};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum Subject {
    Holdings(Token),
    DelegateHoldings(Token),
    ProcedureHoldings { task: u64, step: u64 },
    Describe(Token),
    Verdict(Token, effects::Judge),
    Public(Token),
    SettingsJudge(Token),
    SettingsRestore(Token),
}

#[derive(Hash, Debug)]
pub(crate) struct Pending {
    asks: List<(u16, Subject)>,
}

impl Pending {
    pub(crate) fn new(routes: u32) -> Self {
        Self { asks: List::with_capacity(routes) }
    }

    pub(crate) fn asked(&mut self, connector: u16, subject: Subject) {
        self.asks.push((connector, subject)).expect("connector judgements fit decision routes");
    }

    /// Record the judgements in the batch that leaves the core, not asks held
    /// for a later phase of the start. Public reads register at construction,
    /// where they still have the token used to issue the one-use `ReplyTo`.
    pub(crate) fn issued(&mut self, out: &Queue<Request>) {
        for request in out {
            if let Request::Ask { connector, ask: Ask::Connector(ask) } = request
                && let Some(subject) = subject(ask)
            {
                self.asked(*connector, subject);
            }
        }
    }

    pub(crate) fn answered(&mut self, connector: u16, subject: Subject) {
        for (index, &(asked, pending)) in self.asks.iter().enumerate() {
            if subject == pending && connector == asked {
                let _answered = self.asks.swap_remove(u32::try_from(index).expect("bounded decision routes"));
                return;
            }
        }
    }

    pub(crate) fn outstanding(&self) -> Option<(u16, Subject)> {
        self.asks.as_slice().first().copied()
    }
}

fn subject(ask: &contract::Ask) -> Option<Subject> {
    match ask {
        contract::Ask::Resources(ask) => match ask {
            resources::Ask::Holdings { request, .. } => Some(Subject::Holdings(*request)),
            resources::Ask::DelegateHoldings { request, .. } => Some(Subject::DelegateHoldings(*request)),
            resources::Ask::ProcedureHoldings { task, step, .. } => {
                Some(Subject::ProcedureHoldings { task: *task, step: *step })
            }
            resources::Ask::Unname { .. }
            | resources::Ask::Hold { .. }
            | resources::Ask::Lost { .. }
            | resources::Ask::RepairRefused { .. }
            | resources::Ask::DelegateRefused { .. } => None,
        },
        contract::Ask::Effects(ask) => match ask {
            effects::Ask::Describe { owner } | effects::Ask::DescribeProposal { owner, .. } => {
                Some(Subject::Describe(*owner))
            }
            effects::Ask::Judge { owner, judge, .. } => Some(Subject::Verdict(*owner, *judge)),
            effects::Ask::Keep { .. }
            | effects::Ask::KeepProposal { .. }
            | effects::Ask::Drop { .. }
            | effects::Ask::DropProposal { .. }
            | effects::Ask::Resolve { .. }
            | effects::Ask::Close { .. }
            | effects::Ask::Release { .. } => None,
        },
        contract::Ask::Settings(ask) => match ask {
            settings::Ask::Judge { owner, .. } => Some(Subject::SettingsJudge(*owner)),
            settings::Ask::Restore { owner, .. } => Some(Subject::SettingsRestore(*owner)),
            settings::Ask::Keep { .. }
            | settings::Ask::KeepProposal { .. }
            | settings::Ask::DropProposal { .. }
            | settings::Ask::ForgetReplaced { .. }
            | settings::Ask::SaveDefaults { .. } => None,
        },
        contract::Ask::Parts(
            parts::Ask::Document { .. }
            | parts::Ask::Lines(_)
            | parts::Ask::DropDocument { .. }
            | parts::Ask::Read { .. }
            | parts::Ask::Public { .. }
            | parts::Ask::Plan(_)
            | parts::Ask::Abandon { .. }
            | parts::Ask::Gather { .. }
            | parts::Ask::Cut { .. }
            | parts::Ask::Drop { .. },
        )
        | contract::Ask::Requests(
            requests::Ask::Decide { .. } | requests::Ask::Drop { .. } | requests::Ask::PartAnswered { .. },
        )
        | contract::Ask::Lifecycle(
            lifecycle::Ask::Restore
            | lifecycle::Ask::CatchUp
            | lifecycle::Ask::Adopt { .. }
            | lifecycle::Ask::ReleaseProject { .. },
        )
        | contract::Ask::Topics(
            topics::Ask::Subscribed { .. }
            | topics::Ask::Unsubscribed { .. }
            | topics::Ask::Refused { .. }
            | topics::Ask::Ended { .. },
        )
        | contract::Ask::Procedures(
            procedures::Ask::Start { .. }
            | procedures::Ask::StepMade { .. }
            | procedures::Ask::StepRefused { .. }
            | procedures::Ask::WaitsRefused { .. },
        )
        | contract::Ask::Projections(projections::Ask::Project { .. } | projections::Ask::Forget { .. }) => None,
    }
}

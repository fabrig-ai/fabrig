//! Presentation of committed owner results (client/domain/contract/README.md, 3;
//! client/domain/view.md, 7). Reads supplied facts only: no request, current
//! object, name lookup or explanation is used to fill a missing fact.
use crate::words::Words;
use crate::words::outcome_words as owner_outcome_words;
use crate::words::outcome_words::OutcomePhrase as P;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role, Run};

use jig_client::contract::{
    DocumentOutcome, EscalationChoice, ProposalChoice, ProposalKey, Proposer, Revision, SeenThrough,
};
use skein_lib::Wall;

// Region + named status + title + confirmation, then a reason and whole name per host.
pub(crate) const MAX_NODES: u32 = 4 + 128 * 2;

fn header(value: &DocumentOutcome) -> P {
    match value {
        DocumentOutcome::Created { .. } => P::Created,
        DocumentOutcome::Compound { .. } => P::Compound,
        DocumentOutcome::MarkedSeen { .. } => P::MarkedSeen,
        DocumentOutcome::NoteEdited { .. } => P::NoteEdited,
        DocumentOutcome::ServiceMade { .. } => P::ServiceMade,
        DocumentOutcome::Adopted { .. } => P::Adopted,
        DocumentOutcome::Left { .. } => P::Left,
        DocumentOutcome::Stopped { .. } => P::Stopped,
        DocumentOutcome::Cancelled { .. } => P::Cancelled,
        DocumentOutcome::Released { .. } | DocumentOutcome::Lifted { .. } => P::Released,
        DocumentOutcome::PersonTaken { .. } => P::PersonTaken,
        DocumentOutcome::PersonHandedBack { .. } => P::PersonHandedBack,
        DocumentOutcome::PersonAnswered { .. } => P::PersonAnswered,
        DocumentOutcome::Moved { .. } => P::Moved,
        DocumentOutcome::ProposalDecided { .. } => P::ProposalDecided,
        DocumentOutcome::Said { .. } => P::Said,
        DocumentOutcome::QuestionAnswered { .. } => P::QuestionAnswered,
        DocumentOutcome::Amended { .. } => P::Amended,
        DocumentOutcome::Proposed { .. } => P::Proposed,
        DocumentOutcome::Withdrawn { .. } => P::Withdrawn,
        DocumentOutcome::RolesSet { .. } => P::RolesSet,
        DocumentOutcome::ProjectRemoved { .. } => P::ProjectRemoved,
        DocumentOutcome::VisibilitySet { .. } | DocumentOutcome::Renamed { .. } => P::Renamed,
        DocumentOutcome::ProjectMade { .. } => P::ProjectMade,
        DocumentOutcome::PolicyChanged { .. } => P::PolicyChanged,
        DocumentOutcome::AccountProgramDeclared { .. } => P::AccountProgramDeclared,
        DocumentOutcome::AccountKeyKept { .. } => P::AccountKeyKept,
        DocumentOutcome::AccountMovedIn { renewed, .. } => {
            if *renewed {
                P::AccountRenewed
            } else {
                P::AccountMovedIn
            }
        }
        DocumentOutcome::AccountFinishing { .. } => P::AccountFinishing,
        DocumentOutcome::AccountChanged => P::AccountChanged,
        DocumentOutcome::AccountRemoved { .. } => P::AccountRemoved,
        DocumentOutcome::AccountSignInStarted { .. } => P::AccountSignInStarted,
        DocumentOutcome::AccountSignedOut { .. } => P::AccountSignedOut,
        DocumentOutcome::PoolSet { .. } => P::PoolSet,
        DocumentOutcome::EscalationDecided { .. } => P::EscalationDecided,
    }
}
fn text(b: &mut Builder, words: &Words, phrase: P, holes: &[Hole]) {
    let saying = owner_outcome_words::say(words, phrase, holes);
    b.text(Run::Plain, &saying);
}
fn number(b: &mut Builder, words: &Words, phrase: P, value: u64) {
    text(b, words, phrase, &[Hole::Number(value)]);
}
fn seen(b: &mut Builder, value: &SeenThrough, words: &Words) {
    match value {
        SeenThrough::Results { through } => number(b, words, P::ResultsThrough, *through),
        SeenThrough::Conversation { task: chat, through } => {
            number(b, words, P::Chat, *chat);
            number(b, words, P::RepliesThrough, *through);
        }
    }
}
fn proposal(b: &mut Builder, key: &ProposalKey, by: u64, at: Wall, choice: ProposalChoice, words: &Words) {
    match key.proposer {
        Proposer::Task(task) => number(b, words, P::TaskProposer, task),
        Proposer::Party(party) => number(b, words, P::PartyProposer, party),
    }
    number(b, words, P::ProposalNumber, key.number);
    number(b, words, P::By, by);
    text(b, words, P::At, &[Hole::Wall(words.adjusted_wall(at))]);
    let phrase = match choice {
        ProposalChoice::Accepted => P::Accepted,
        ProposalChoice::Rejected => P::Rejected,
        ProposalChoice::Passed => P::Passed,
        ProposalChoice::Withdrawn => P::Withdrawn,
        ProposalChoice::Stale => P::Stale,
    };
    text(b, words, phrase, &[]);
}
fn escalation(
    b: &mut Builder,
    task: u64,
    revision: Revision,
    by: u64,
    at: Wall,
    choice: EscalationChoice,
    words: &Words,
) {
    number(b, words, P::Task, task);
    number(b, words, P::Revision, revision.value);
    number(b, words, P::By, by);
    text(b, words, P::At, &[Hole::Wall(words.adjusted_wall(at))]);
    let phrase = match choice {
        EscalationChoice::Released => P::ReleaseChoice,
        EscalationChoice::Rejected => P::Rejected,
        EscalationChoice::Passed => P::Passed,
    };
    text(b, words, phrase, &[]);
}
fn details(b: &mut Builder, value: &DocumentOutcome, words: &Words) {
    match value {
        DocumentOutcome::AccountKeyKept { .. }
        | DocumentOutcome::AccountChanged
        | DocumentOutcome::AccountSignInStarted { .. }
        | DocumentOutcome::Lifted { .. } => {}
        DocumentOutcome::AccountFinishing { source } => number(b, words, P::SourceNumber, u64::from(*source)),
        DocumentOutcome::AccountMovedIn { account, .. } => number(b, words, P::AccountNumber, u64::from(*account)),
        DocumentOutcome::AccountProgramDeclared { source, name } => {
            number(b, words, P::SourceNumber, u64::from(*source));
            b.text(Run::Plain, name);
        }
        DocumentOutcome::AccountRemoved { confirmed, failed_hosts }
        | DocumentOutcome::AccountSignedOut { confirmed, failed_hosts } => {
            let status = if *confirmed { P::SignOutConfirmed } else { P::SignOutUnconfirmed };
            text(b, words, status, &[]);
            for host in &**failed_hosts {
                let why = match host.why {
                    jig_client::contract::AccountHostWhy::NotInContact => P::HostNotInContact,
                    jig_client::contract::AccountHostWhy::Lost => P::HostLost,
                    jig_client::contract::AccountHostWhy::Failed => P::HostFailed,
                };
                text(b, words, why, &[]);
                b.text(Run::Plain, &host.name);
            }
        }
        DocumentOutcome::Created { first, last, given } => created(b, *first, *last, *given, words),
        DocumentOutcome::MarkedSeen { seen: position } => seen(b, position, words),
        DocumentOutcome::NoteEdited { name } => number(b, words, P::Note, *name),
        DocumentOutcome::ServiceMade { person } => number(b, words, P::Person, *person),
        DocumentOutcome::Adopted { project }
        | DocumentOutcome::VisibilitySet { project, .. }
        | DocumentOutcome::ProjectRemoved { project }
        | DocumentOutcome::Renamed { project }
        | DocumentOutcome::ProjectMade { project }
        | DocumentOutcome::RolesSet { project }
        | DocumentOutcome::PolicyChanged { project } => number(b, words, P::Project, u64::from(*project)),
        DocumentOutcome::Left { task }
        | DocumentOutcome::Stopped { task }
        | DocumentOutcome::Cancelled { task }
        | DocumentOutcome::Released { task }
        | DocumentOutcome::PersonTaken { task }
        | DocumentOutcome::PersonHandedBack { task }
        | DocumentOutcome::PersonAnswered { task }
        | DocumentOutcome::Moved { task } => number(b, words, P::Task, *task),
        DocumentOutcome::ProposalDecided { proposal: key, by, at, choice, repeated: _ } => {
            proposal(b, key, *by, *at, *choice, words);
        }
        DocumentOutcome::Compound { project, operations, first_message } => {
            number(b, words, P::Project, u64::from(*project));
            number(b, words, P::Operations, u64::from(*operations));
            number(b, words, P::Message, *first_message);
        }
        DocumentOutcome::Said { task, message } => {
            number(b, words, P::Task, *task);
            number(b, words, P::Message, *message);
        }
        DocumentOutcome::QuestionAnswered { task, question, message } => {
            number(b, words, P::Task, *task);
            number(b, words, P::Question, *question);
            number(b, words, P::Message, *message);
        }
        DocumentOutcome::Amended { task } => {
            number(b, words, P::Task, *task);
        }
        DocumentOutcome::Proposed { proposal, .. } | DocumentOutcome::Withdrawn { proposal } => {
            number(b, words, P::ProposalNumber, *proposal);
        }
        DocumentOutcome::PoolSet { project, person } => {
            number(b, words, P::Project, u64::from(*project));
            number(b, words, P::Person, *person);
        }
        DocumentOutcome::EscalationDecided { task, revision, by, at, choice } => {
            escalation(b, *task, *revision, *by, *at, *choice, words);
        }
    }
}
/// Build one bounded fragment using only the committed value and selected Words.
pub fn render(b: &mut Builder, value: &DocumentOutcome, words: &Words) {
    b.open(Role::Region);
    let title = owner_outcome_words::say(words, header(value), &[]);
    b.name(&title);
    b.open(Role::Status);
    b.name(&title);
    b.text(Run::Plain, &title);
    b.close();
    details(b, value, words);
    b.close();
}

fn created(b: &mut Builder, first: u64, last: u64, given: Option<jig_client::contract::CreatedGiven>, words: &Words) {
    number(b, words, P::FirstTask, first);
    number(b, words, P::LastTask, last);
    if let Some(given) = given {
        number(b, words, P::GivenTask, given.task);
        number(b, words, P::GivenAmount, given.amount);
    }
}

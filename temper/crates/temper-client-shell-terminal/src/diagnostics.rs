//! Bounded, payload-free state for an overdue real-loop snapshot.
use jig_client::{contract as c, thread::Reading};
use skein_lib::List;

#[derive(Debug)]
pub struct Pending {
    pub loaded: bool,
    pub dirty: bool,
    pub shell: jig_client_shell_native::diagnostics::Pending,
    pub admission: jig_client::Admission,
    pub chat: Option<Chat>,
    pub view_revision: Option<u64>,
    pub builds: u64,
    pub patches: u64,
    pub address: temper_client_domain::Address,
    pub model: Option<Model>,
}
/// Model setup metadata, without the pasted value, digest or request name.
#[derive(Debug)]
pub struct Model {
    pub holder: c::AskAccountHolder,
    pub has_key: bool,
    pub broken: Option<temper_client_domain::pages::accounts::Broken>,
    pub request: &'static str,
    pub refusal: Option<jig_client::words::Word>,
}

pub(crate) fn model(page: &temper_client_domain::pages::connect_model::ConnectModel) -> Model {
    let mut refusal = None;
    let request = match page.request.pending.as_ref().and_then(|pending| pending.outcome.as_ref()) {
        None => if page.request.pending.is_some() { "pending" } else { "none" },
        Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(answer))) => match answer {
            c::Answer::Refused { refusal: why, .. } => {
                refusal = Some(jig_client::words::Word::refusal(why));
                "refused"
            }
            c::Answer::Done(outcome) => {
                if let c::Outcome::Document(c::DocumentOutcome::AccountKeyKept { .. }) = outcome {
                    "account key kept"
                } else {
                    "done"
                }
            }
        },
        Some(jig_client::Outcome::Answered(jig_client::Answered::Family(_))) => "family answer",
        Some(jig_client::Outcome::NotKept) => "not kept",
        Some(jig_client::Outcome::Unknown) => "unknown",
        Some(jig_client::Outcome::Limit) => "limit",
        Some(jig_client::Outcome::Unconfirmed) => "unconfirmed",
        Some(jig_client::Outcome::Held) => "held",
        Some(jig_client::Outcome::Unsent) => "unsent",
        Some(jig_client::Outcome::Full { .. }) => "full",
        Some(jig_client::Outcome::Lost) => "lost",
    };
    Model { holder: page.holder, has_key: page.request.has_key(), broken: page.request.broken, request, refusal }
}
#[derive(Debug)]
pub struct Chat {
    pub document_state: Option<jig_client::State>,
    pub activity: Option<u64>,
    pub committed: u64,
    pub loaded: u64,
    pub reading: Option<Reading>,
    pub last_read_none: Option<c::ReadNone>,
    pub retained: List<Row>,
    pub staged: u32,
}
#[derive(Debug)]
pub struct Row {
    pub position: u64,
    pub committed_turn: Option<c::TurnName>,
}

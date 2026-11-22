//! Adoption refusal words (temper/docs/design/domain/forge/adoption.md, 5).
use alloc::boxed::Box;
use jig_domain_people as people;

/// Render the connector's adoption refusal without changing its reason or bound.
#[must_use]
pub fn adoption_refusal_words(why: people::Refusal) -> Box<[u8]> {
    let words = match why {
        people::Refusal::LiveTasks { .. } => unreachable!("only an owner's RemoveProject refuses live tasks"),
        people::Refusal::NotOffered => b"the forge connector is not granted adopt (engine.acts)".as_slice(),
        people::Refusal::Role { held: None } => b"adopt has no configured first owner",
        people::Refusal::Role { held: Some(_) } => b"adopt lacks the required role",
        people::Refusal::Unknown => b"adopt project no longer exists",
        people::Refusal::Ended { .. } => b"adopt project has ended",
        people::Refusal::Authority { .. } => b"adopt lacks resource access or authority",
        people::Refusal::Limit { .. } => b"adopt exceeds a configured limit",
        people::Refusal::KeyConflict => b"adopt key names another act",
        people::Refusal::Judged { why, .. } => {
            return Box::from(why.get(..why.len().min(256)).expect("bounded adoption refusal"));
        }
        people::Refusal::Forgotten { .. } => b"adopt revision was forgotten",
        people::Refusal::Stale { .. } => b"adopt revision is stale",
        people::Refusal::Full { .. } => b"adopt has no room",
        people::Refusal::NameTaken => b"adopt name is taken",
        people::Refusal::NotWellFormed { .. } => b"adopt is not well formed",
        people::Refusal::Moved { .. } => b"adopt subject has moved",
        people::Refusal::Taken { .. } => b"adopt subject is taken",
        people::Refusal::NotWaiting => b"adopt subject is not waiting",
        people::Refusal::NoFurther => b"adopt has no further holder",
        people::Refusal::Provider => b"adopt provider is not allowed",
        people::Refusal::NoEscalationHolder => b"adopt has no escalation holder",
        people::Refusal::Llm { .. } => b"adopt model is not allowed",
        people::Refusal::Account { .. }
        | people::Refusal::NoTerminal
        | people::Refusal::LoginUnderWay
        | people::Refusal::NotLending
        | people::Refusal::AccountMove { .. }
        | people::Refusal::ProgramNameReserved => b"adopt account is not allowed",
        people::Refusal::SignInRedirect { .. } => b"adopt sign-in redirect was refused",
        people::Refusal::Circle { .. } => b"adopt has a dependency circle",
        people::Refusal::Dependency { .. } => b"adopt prerequisite failed",
        people::Refusal::Unlinked { .. } => b"worker is unlinked",
        people::Refusal::Concluded { .. } => b"adopt gathering is concluded",
        people::Refusal::Kept { .. } => b"adopt waits for prerequisites",
    };
    words.into()
}

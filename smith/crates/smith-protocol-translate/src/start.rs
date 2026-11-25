//! Start context before domain admission (smith/docs/design/protocol/channel.md, sections 3 and 5).
use crate::Error;
use crate::credentials::renewable;
use crate::generated::channel as mapped;
use alloc::boxed::Box;
use skein_lib::List;
use smith_channel as channel;
use smith_domain::run;

/// One directory whose root path the agent service attaches to io.
#[derive(Debug)]
pub struct Mount {
    pub name: Box<[u8]>,
    pub path: Box<[u8]>,
    pub writable: bool,
    pub git: bool,
    pub conflicts: Box<[Box<[u8]>]>,
    pub results: Option<Box<[u8]>>,
}

/// One grant name and validity; the agent channel keeps its credential value.
#[derive(Debug)]
pub struct Grant {
    pub name: smith_domain::GrantName,
    pub valid: skein_lib::Duration,
    /// A key is never renewed by its host (smith/docs/design/protocol/channel.md, section 6).
    pub renewable: bool,
}

/// Start context after channel and charter decoding, before io root attachment.
#[derive(Debug)]
pub struct DecodedStart {
    pub live: Option<channel::LiveAsk>,
    /// Opaque host identity preserved through framing and charter decoding.
    pub identity: Option<run::Identity>,
    pub messages: Box<[run::Message]>,
    pub activation: u64,
    pub charter: run::Charter,
    pub mounts: Option<Box<[Mount]>>,
    pub transcript: Option<smith_domain_session::record::Transcript>,
    pub tell_parts: bool,
    pub parts: Box<[smith_domain_session::record::Resumed]>,
    pub answered: Box<[smith_domain::AnsweredCall]>,
    pub grants: Box<[Grant]>,
    pub window: smith_domain::Window,
}

/// Move decoded channel fields into bounded start context, keeping values below the domain.
pub fn start_context(
    start: channel::Start,
    charter: run::Charter,
    transcript: Option<smith_domain_session::record::Transcript>,
) -> Result<DecodedStart, Error> {
    let parts = start;
    let mounts = match parts.workspace {
        Some(workspace) => {
            let mut mounts = List::with_capacity(workspace.directories.len());
            for directory in workspace.directories.into_boxed() {
                let mount = mapped::directory_to_domain(directory);
                if mounts.push(mount).is_err() {
                    return Err(Error::Capacity);
                }
            }
            Some(mounts.into_boxed())
        }
        None => None,
    };
    let mut grants = List::with_capacity(parts.grants.len());
    for grant in &parts.grants {
        let item = Grant {
            name: smith_domain::GrantName { account: grant.account, generation: grant.generation },
            valid: grant.valid,
            renewable: renewable(&grant.value),
        };
        if grants.push(item).is_err() {
            return Err(Error::Capacity);
        }
    }
    let mut messages = List::with_capacity(parts.messages.len());
    for message in parts.messages.into_boxed() {
        if messages.push(mapped::message_to_domain(message)).is_err() {
            return Err(Error::Capacity);
        }
    }
    Ok(DecodedStart {
        live: parts.live,
        identity: match parts.identity {
            Some(bytes) => Some(run::Identity::new(bytes)),
            None => None,
        },
        messages: messages.into_boxed(),
        activation: parts.activation,
        charter,
        mounts,
        transcript,
        tell_parts: parts.tell_parts,
        parts: Box::default(),
        answered: saved_answers(parts.answered)?,
        grants: grants.into_boxed(),
        window: mapped::window_to_domain(parts.window),
    })
}

/// Move post-transcript host decisions into the domain's settled vocabulary.
fn saved_answers(source: List<channel::AnsweredCall>) -> Result<Box<[smith_domain::AnsweredCall]>, Error> {
    let mut answered = List::with_capacity(source.len());
    for call in source.into_boxed() {
        let item = mapped::answered_call_to_domain(call).or(Err(Error::InvalidSavedAnswer))?;
        answered.push(item).or(Err(Error::InvalidSavedAnswer))?;
    }
    Ok(answered.into_boxed())
}

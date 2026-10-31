//! Start context before domain admission (protocol/channel.md, sections 3 and 5).
use crate::Error;
use crate::credentials::renewable;
use crate::delivery::saved_delivery;
use alloc::boxed::Box;
use skein_lib::{List, Token};
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
}

/// One grant name and validity; the agent channel keeps its credential value.
#[derive(Debug)]
pub struct Grant {
    pub name: smith_domain::GrantName,
    pub valid: skein_lib::Duration,
    /// A key is never renewed by its host (protocol/channel.md, section 6).
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
    pub answered: Box<[smith_domain::AnsweredCall]>,
    pub grants: Box<[Grant]>,
    pub window: smith_domain::Window,
}

/// Move decoded channel fields into bounded start context, keeping values below the domain.
#[expect(clippy::manual_map, reason = "strict subset keeps optional boundary conversion as an explicit match")]
pub fn start_context(
    start: channel::Start,
    charter: run::Charter,
    transcript: Option<smith_domain_session::record::Transcript>,
) -> Result<DecodedStart, Error> {
    let parts = start.into_parts();
    let mounts = match parts.workspace {
        Some(workspace) => {
            let mut mounts = List::with_capacity(workspace.directories().len());
            for directory in workspace.directories() {
                let mount = Mount {
                    name: Box::from(directory.name()),
                    path: Box::from(directory.path()),
                    writable: directory.writable(),
                    git: directory.git(),
                    conflicts: directory.conflicts().to_boxed(),
                };
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
            name: smith_domain::GrantName { account: grant.account(), generation: grant.generation() },
            valid: grant.valid(),
            renewable: renewable(grant.value()),
        };
        if grants.push(item).is_err() {
            return Err(Error::Capacity);
        }
    }
    let mut messages = List::with_capacity(parts.messages.len());
    for message in parts.messages.into_boxed() {
        let parts = message.into_parts();
        if messages
            .push(run::Message {
                finish_requested: parts.finish_requested,
                name: Token::new(parts.name),
                label: parts.label,
                text: parts.text,
            })
            .is_err()
        {
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
        answered: saved_answers(parts.answered.as_slice())?,
        grants: grants.into_boxed(),
        window: smith_domain::Window { turns: parts.window.turns(), bytes: parts.window.bytes() },
    })
}

/// Move post-transcript host decisions into the domain's settled vocabulary.
fn saved_answers(source: &[channel::AnsweredCall]) -> Result<Box<[smith_domain::AnsweredCall]>, Error> {
    let Ok(count) = u32::try_from(source.len()) else {
        return Err(Error::InvalidSavedAnswer);
    };
    let mut answered = List::with_capacity(count);
    for call in source {
        let name = call.name();
        let reply = match call.reply() {
            channel::SavedReply::Host(host) => {
                let answer =
                    run::HostAnswer::new(Box::from(host.text()), host.error()).ok_or(Error::InvalidSavedAnswer)?;
                smith_domain::Answered::Host(answer)
            }
            channel::SavedReply::Delivery(delivery) => {
                smith_domain::Answered::Delivery(Box::new(saved_delivery(delivery.value())?))
            }
            channel::SavedReply::TooLarge => smith_domain::Answered::TooLarge,
        };
        let item = smith_domain::AnsweredCall {
            name: run::CallName {
                activation: name.activation(),
                completion: name.completion(),
                position: name.position(),
            },
            tool: Box::from(call.tool()),
            answer: reply,
        };
        if answered.push(item).is_err() {
            return Err(Error::InvalidSavedAnswer);
        }
    }
    Ok(answered.into_boxed())
}

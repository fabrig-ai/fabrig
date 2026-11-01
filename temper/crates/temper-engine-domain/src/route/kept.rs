//! Kept removal and reset acts and project release handoffs
//! (`temper/docs/design/draft/playground-reset.md`, 4 and 6.2;
//! `jig/docs/design/domain/people.md`, 5.3).
use super::{Decision, Delivery, Domain, Env, Limits, Request, Work, emit, forge};
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_domain_people as people;
use skein_lib::{Map, ReplyTo, Token};

#[derive(Debug)]
pub(crate) struct Replies {
    kept: Map<Token, forge::KeptAct>,
    reset: Map<Token, forge::ResetAct>,
    pub(super) start: Map<Token, super::playground::ActReply>,
}

impl Replies {
    pub(crate) fn new(limits: &forge::Limits) -> Self {
        let forges = limits.forges.checked_mul(2).expect("validated forge act bound");
        let kept = limits.tasks.checked_add(forges).expect("validated kept act bound");
        Self {
            kept: Map::with_capacity(kept),
            reset: Map::with_capacity(2),
            start: Map::with_capacity(limits.applications),
        }
    }
}

pub(crate) fn reply_token(domain: &mut Domain) -> Token {
    let token = Token::new(domain.next_act_reply);
    domain.next_act_reply = domain.next_act_reply.checked_add(1).expect("deployment act reply sequence");
    token
}

pub(crate) fn send_kept(domain: &mut Domain, row: forge::KeptAct) {
    match row.act {
        forge::KeptActKind::Cancel { project, task } => {
            let token = reply_token(domain);
            domain.kept_replies.kept.insert(token, row).expect("one reply per kept act");
            send(
                domain,
                token,
                row.key,
                people::Act::Cancel { project, task, reason: b"Forge removed".as_slice().into() },
            );
        }
        forge::KeptActKind::RemoveGroup => {
            assert!(super::now(domain, Request::RemoveGroup { forge: row.forge }), "remove group door");
        }
        forge::KeptActKind::EraseSecrets => {
            assert!(super::now(domain, Request::EraseSecrets { forge: row.forge }), "erase secrets door");
        }
    }
}

fn reset_key(purpose: &[u8], number: u32) -> [u8; 16] {
    let mut hash = skein_lib::sha256::Sha256::new();
    hash.update(b"temper playground reset\0");
    hash.update(&u64::from(number).to_be_bytes());
    hash.update(purpose);
    hash.finish().get(..16).expect("digest prefix").try_into().expect("16-byte key")
}

pub(crate) fn send_reset(domain: &mut Domain, reset: forge::ResetAct) {
    let (key, act) = match reset {
        forge::ResetAct::RemoveProject { project } => {
            (reset_key(b"project", project), people::Act::RemoveProject { project })
        }
        forge::ResetAct::RemoveAccount { account } => {
            (reset_key(b"account", account), people::Act::RemoveAccount { account })
        }
    };
    let token = reply_token(domain);
    domain.kept_replies.reset.insert(token, reset).expect("one reply per reset act");
    send(domain, token, key, act);
}

fn send(domain: &mut Domain, token: Token, key: [u8; 16], act: people::Act) {
    domain.work.push(Work::Core(core::Event::Act {
        reply_to: ReplyTo::new(token),
        connector: domain.config.forge_connector,
        key,
        act: Box::new(act),
        carried: None,
    }));
}

pub(crate) fn acted(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    to: ReplyTo,
    outcome: people::Outcome,
) {
    let token = to.into_token();
    if let Some(reply) = domain.kept_replies.start.remove(&token) {
        super::playground::acted(domain, env, decision, reply, outcome);
        return;
    }
    let (event, prefix, why) = if let Some(row) = domain.kept_replies.kept.remove(&token) {
        let (keep, why) = classify(&outcome, "cancel");
        (forge::Event::KeptActAnswered { key: row.key, keep }, b"forge removal: ".as_slice(), why)
    } else {
        let act = domain.kept_replies.reset.remove(&token).expect("kept or reset act reply");
        let grant = match act {
            forge::ResetAct::RemoveProject { .. } => "remove_project",
            forge::ResetAct::RemoveAccount { .. } => "remove_account",
        };
        let (keep, why) = classify(&outcome, grant);
        (forge::Event::ResetActed { act, keep }, b"playground reset: ".as_slice(), why)
    };
    if let Some(why) = why {
        emit(decision, &env.limits, Delivery::DemoNote { line: note(prefix, why) });
    }
    domain.work.push(Work::Forge(event));
}

fn classify(outcome: &people::Outcome, grant: &'static str) -> (bool, Option<&'static str>) {
    match outcome {
        people::Outcome::Again(_) => (true, None),
        people::Outcome::Refused(people::Refusal::NotOffered) => (
            true,
            Some(match grant {
                "cancel" => "the forge connector is not granted cancel (engine.acts)",
                "remove_project" => "the forge connector is not granted remove_project (engine.acts)",
                "remove_account" => "the forge connector is not granted remove_account (engine.acts)",
                _ => unreachable!("removal act grant"),
            }),
        ),
        people::Outcome::Refused(people::Refusal::Role { held: None }) => (true, Some("no first owner configured")),
        people::Outcome::Refused(people::Refusal::Unknown) => (false, None),
        people::Outcome::Refused(people::Refusal::Ended { .. }) | people::Outcome::Compound { .. }
            if grant == "cancel" =>
        {
            (false, None)
        }
        people::Outcome::Refused(why) | people::Outcome::CompoundRefused { why, .. } => {
            (false, Some(refusal_name(why)))
        }
        people::Outcome::ProjectRemoved { .. } if grant == "remove_project" => (false, None),
        people::Outcome::AccountRemoved { .. } if grant == "remove_account" => (false, None),
        people::Outcome::Compound { .. }
        | people::Outcome::ProjectRemoved { .. }
        | people::Outcome::AccountRemoved { .. }
        | people::Outcome::SettingsChanged { .. }
        | people::Outcome::MovedToNewest { .. }
        | people::Outcome::Renamed { .. }
        | people::Outcome::ProjectMade { .. }
        | people::Outcome::Created { .. }
        | people::Outcome::MarkedSeen { .. }
        | people::Outcome::Applied { .. }
        | people::Outcome::SignedOut
        | people::Outcome::NoteEdited { .. }
        | people::Outcome::ServiceMade { .. }
        | people::Outcome::Adopted { .. }
        | people::Outcome::Proposed { .. }
        | people::Outcome::Withdrawn { .. }
        | people::Outcome::Stopped { .. }
        | people::Outcome::Cancelled { .. }
        | people::Outcome::Released { .. }
        | people::Outcome::PersonTaken { .. }
        | people::Outcome::PersonHandedBack { .. }
        | people::Outcome::PersonAnswered { .. }
        | people::Outcome::Moved { .. }
        | people::Outcome::ProposalDecided { .. }
        | people::Outcome::Said { .. }
        | people::Outcome::QuestionAnswered { .. }
        | people::Outcome::Amended { .. }
        | people::Outcome::RolesSet { .. }
        | people::Outcome::PolicyChanged { .. }
        | people::Outcome::PoolSet { .. }
        | people::Outcome::EscalationDecided { .. }
        | people::Outcome::AccountSignInStarted { .. }
        | people::Outcome::AccountFinishing { .. }
        | people::Outcome::AccountSignedOut { .. }
        | people::Outcome::AccountKeyKept { .. }
        | people::Outcome::AccountProgramDeclared { .. }
        | people::Outcome::AccountChanged
        | people::Outcome::AccountMovingOut { .. }
        | people::Outcome::Lifted { .. }
        | people::Outcome::AccountMovedIn { .. } => unreachable!("removal act answer"),
    }
}

fn note(prefix: &[u8], why: &str) -> Box<[u8]> {
    let length = prefix.len().checked_add(why.len()).expect("bounded removal diagnostic");
    assert!(length <= 256, "bounded secret-free diagnostic");
    let mut writer = skein_lib::Writer::new(length);
    writer.put(prefix).expect("measured prefix");
    writer.put(why.as_bytes()).expect("measured reason");
    writer.finish()
}

pub(crate) fn release(domain: &mut Domain, connector: u16, project: u32) {
    if connector == domain.config.forge_connector {
        domain.work.push(Work::Forge(forge::Event::ReleaseProject { project }));
    } else {
        domain.work.push(Work::Core(core::Event::ProjectReleased { project, connector }));
    }
}

pub(crate) fn released(domain: &mut Domain, project: u32) {
    domain.work.push(Work::Core(core::Event::ProjectReleased { project, connector: domain.config.forge_connector }));
}

pub(super) fn refusal_name(why: &people::Refusal) -> &'static str {
    use people::Refusal;
    match why {
        Refusal::Stale { .. } => "Stale",
        Refusal::Full { .. } => "Full",
        Refusal::NameTaken => "NameTaken",
        Refusal::NotWellFormed { why: people::Malformed::Name(_), .. } => "Name",
        Refusal::Circle { .. } => "Circle",
        Refusal::Dependency { .. } => "Dependency",
        Refusal::Kept { .. } => "Kept",
        Refusal::Account { .. } => "Account",
        Refusal::NotWellFormed { .. } => "NotWellFormed",
        Refusal::Role { .. } => "Role",
        Refusal::Authority { .. } => "Authority",
        Refusal::Limit { .. } => "Limit",
        Refusal::Moved { .. } => "Moved",
        Refusal::Ended { .. } => "Ended",
        Refusal::Taken { .. } => "Taken",
        Refusal::NotWaiting => "NotWaiting",
        Refusal::NoFurther => "NoFurther",
        Refusal::Unknown => "Unknown",
        Refusal::NotOffered => "NotOffered",
        Refusal::Provider => "Provider",
        Refusal::SignInRedirect { .. } => "SignInRedirect",
        Refusal::KeyConflict => "KeyConflict",
        Refusal::NoEscalationHolder => "NoEscalationHolder",
        Refusal::Llm { .. } => "Llm",
        Refusal::AccountMove { .. } => "AccountMove",
    }
}

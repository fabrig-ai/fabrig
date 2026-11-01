//! Kept-act routing (temper/docs/plan/playground-reset-r4/notes/contracts.md, D4;
//! temper/docs/design/protocol/setup.md, 8.4).
use super::{Box, Decision, Delivery, Domain, Env, Limits, ReplyTo, Token, Work, core, emit, forge, people};

const REPLY_TAG: u64 = 1 << 62;

pub(crate) fn owns(token: Token) -> bool {
    token.raw() & REPLY_TAG != 0
}

pub(crate) fn send(domain: &mut Domain, row: forge::Kept) {
    let act = match row.act {
        forge::KeptAct::Cancel { project, task } => {
            let mut grace = None;
            for setting in &domain.core.current.offered() {
                let jig_domain_core::settings::Setting::Template(number) = setting else { continue };
                let template = domain.core.current.template(*number).expect("offered template");
                if template.flags.conversation {
                    grace = template.grace;
                    break;
                }
            }
            people::Act::Cancel { project, task, grace, reason: b"Removed with its forge".as_slice().into() }
        }
        forge::KeptAct::RemoveProject { project } => people::Act::RemoveProject { project },
        forge::KeptAct::RemoveAccount { account } => people::Act::RemoveAccount { account },
        forge::KeptAct::BoxRemove { forge } => {
            assert!(super::now(domain, crate::Request::BoxRemove { forge }), "box removal door reserved");
            return;
        }
        forge::KeptAct::RemoveForge { .. } => return,
    };
    assert!(domain.next_kept_reply < REPLY_TAG, "kept reply sequence space");
    let token = Token::new(REPLY_TAG | domain.next_kept_reply);
    domain.next_kept_reply = domain.next_kept_reply.checked_add(1).expect("kept reply sequence");
    domain.kept_replies.insert(token, row).expect("one reply per kept act");
    domain.work.push(Work::Core(core::Event::Act {
        reply_to: ReplyTo::new(token),
        connector: domain.config.forge_connector,
        key: row.key,
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
    let row = domain.kept_replies.remove(&to.into_token()).expect("kept act reply");
    let (keep, why) = match &outcome {
        people::Outcome::Again(_) => (true, None),
        people::Outcome::Refused(people::Refusal::NotOffered) => (
            true,
            Some(match row.act {
                forge::KeptAct::Cancel { .. } => "the forge connector is not granted cancel (engine.acts)",
                forge::KeptAct::RemoveProject { .. } => {
                    "the forge connector is not granted remove_project (engine.acts)"
                }
                forge::KeptAct::BoxRemove { .. } | forge::KeptAct::RemoveForge { .. } => {
                    unreachable!("connector-owned act")
                }
                forge::KeptAct::RemoveAccount { .. } => {
                    "the forge connector is not granted remove_account (engine.acts)"
                }
            }),
        ),
        people::Outcome::Refused(people::Refusal::Role { held: None }) => (true, Some("no first owner configured")),
        people::Outcome::Refused(people::Refusal::Unknown | people::Refusal::Ended { .. })
        | people::Outcome::Compound { .. }
        | people::Outcome::ProjectRemoved { .. }
        | people::Outcome::AccountRemoved { .. } => (false, None),
        people::Outcome::Refused(why) | people::Outcome::CompoundRefused { why, .. } => {
            (false, Some(refusal_name(why)))
        }
        people::Outcome::SettingsChanged { .. }
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
        | people::Outcome::AccountMovedIn { .. } => unreachable!("cancel, remove project or remove account answer"),
    };
    if let Some(why) = why {
        let line = note(row.act, why.as_bytes());
        emit(decision, &env.limits, Delivery::DemoNote { line });
    }
    domain.work.push(Work::Forge(forge::Event::KeptAnswered { key: row.key, keep }));
}

fn refusal_name(why: &people::Refusal) -> &'static str {
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

fn note(act: forge::KeptAct, why: &[u8]) -> Box<[u8]> {
    let (subject, number, action) = match act {
        forge::KeptAct::Cancel { task, .. } => (b"task ".as_slice(), task, b" not cancelled: ".as_slice()),
        forge::KeptAct::RemoveProject { project } => {
            (b"project ".as_slice(), u64::from(project), b" not removed: ".as_slice())
        }
        forge::KeptAct::BoxRemove { .. } | forge::KeptAct::RemoveForge { .. } => unreachable!("connector-owned act"),
        forge::KeptAct::RemoveAccount { account } => {
            (b"account ".as_slice(), u64::from(account), b" not removed: ".as_slice())
        }
    };
    let number = super::forge_route::decimal(number);
    let parts = [b"kept act: ".as_slice(), subject, number.as_ref(), action, why];
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("bounded kept diagnostic");
    }
    assert!(length <= 256, "bounded secret-free diagnostic");
    let mut writer = skein_lib::Writer::new(length);
    for part in parts {
        writer.put(part).expect("measured diagnostic");
    }
    writer.finish()
}

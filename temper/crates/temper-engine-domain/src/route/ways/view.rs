//! Views read stored settings in a project's scope (temper/docs/design/client/README.md, 5.16;
//! temper/docs/design/domain/forge.md, 8.13; temper/docs/design/draft/ask-temper.status.md, Design departs).
use crate::{Decision, Domain, ForgeReadAnswer, Limits, ways};
use alloc::boxed::Box;
use jig_domain_core::{documents, reads};
use jig_domain_people::Proposer;
use skein_lib::{Decimal, Env, List, ReplyTo, bytes};
use temper_engine_domain_forge::settings;

mod address;
mod charter;
mod roles;
mod rule;
use address::Address;

#[derive(Hash, Debug)]
pub(crate) struct Flight {
    project: u32,
    sign_in: u64,
    proposer: Proposer,
    number: u64,
}

pub(crate) fn read(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    reply: ReplyTo,
    sign_in: u64,
    project: u32,
    line: Box<[u8]>,
) {
    let party = domain.core.people.person(sign_in, env.now, env.wall).expect("authorized view reader");
    let view = match address::parse(&line) {
        Address::Way(name) => way(domain, project, party, &name),
        Address::Path { repository, branch, revision } => match old_revision(domain, revision) {
            Some(view) => view,
            None => path(domain, project, party, &repository, branch.as_deref()),
        },
        Address::Charter { number, revision } => match old_revision(domain, revision) {
            Some(view) => view,
            None => charter::draw(domain, project, party, number),
        },
        Address::Roles => roles::draw(domain, project, party),
        Address::Rule { kind, number } => rule::draw(domain, project, party, kind, number),
        Address::Proposal { proposer, number } => {
            let token = reply.into_token();
            let reply = ReplyTo::new(token);
            if domain.view_reads.insert(token, Flight { project, sign_in, proposer, number }).is_err() {
                super::super::emit(
                    decision,
                    &env.limits,
                    crate::Delivery::Read { reply, answer: reads::Answer::None(reads::Why::Busy) },
                );
                return;
            }
            let requests = reads::settings::project_proposal(
                &mut domain.core,
                &super::super::environment_core(env),
                reply,
                sign_in,
                project,
                proposer,
                number,
            );
            super::super::route_core_requests(domain, env, decision, requests);
            return;
        }
        Address::Unknown => not_drawn(&[
            b"no view ",
            &line,
            "; views: way NAME · path OWNER/NAME [BRANCH] · proposal tN/K · charter N · rule K/N.".as_bytes(),
        ]),
    };
    deliver(env, decision, reply, view);
}

fn old_revision(domain: &Domain, revision: Option<u32>) -> Option<ways::View> {
    if let Some(revision) = revision
        && revision != domain.core.current.newest()
    {
        return Some(not_drawn(&[
            b"revision ",
            Decimal::of(u64::from(revision)).as_bytes(),
            b" isn't drawn yet; views draw the newest settings, revision ",
            Decimal::of(u64::from(domain.core.current.newest())).as_bytes(),
            b".",
        ]));
    }
    None
}

fn way(domain: &Domain, project: u32, party: u64, name: &[u8]) -> ways::View {
    let document = super::document(domain, project, party);
    for (at, entry) in document.ways.iter().enumerate() {
        if entry.name.eq_ignore_ascii_case(name) {
            return ways::View::Way {
                name: entry.name.clone(),
                line: entry.line.clone(),
                path: entry.path.clone(),
                current: document.current == u32::try_from(at).ok(),
            };
        }
    }
    not_drawn(&[b"no way ", name, b"; the ways are You approve, Agents review and Draft first."])
}

fn path(domain: &Domain, project: u32, party: u64, name: &[u8], branch: Option<&[u8]>) -> ways::View {
    for repository in &domain.forge.readable_repositories(project) {
        if bytes::concat(&[&repository.owner, b"/", &repository.name]).as_ref() == name {
            return ways::View::Path(ways::derive(&super::inputs(
                domain,
                project,
                party,
                &[],
                Some(repository),
                branch,
            )));
        }
    }
    not_drawn(&[b"no repository ", name, b" in this project."])
}

pub(crate) fn answered(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    reply: ReplyTo,
    answer: reads::Answer,
) {
    let token = reply.into_token();
    let flight = domain.view_reads.remove(&token).expect("project proposal view flight");
    let reply = ReplyTo::new(token);
    let view = match answer {
        reads::Answer::Document(document) => {
            let documents::Document::Proposal(proposal) = *document else { unreachable!("project proposal document") };
            proposal_view(domain, env, &flight, &proposal)
        }
        reads::Answer::Ended(_) => decided(&flight),
        reads::Answer::None(reads::Why::Unknown | reads::Why::Gone) => {
            not_drawn(&[b"no proposal ", &proposal_name(&flight), b"."])
        }
        reads::Answer::None(why) => {
            super::super::emit(
                decision,
                &env.limits,
                crate::Delivery::Read { reply, answer: reads::Answer::None(why) },
            );
            return;
        }
        reads::Answer::Revision(_)
        | reads::Answer::Revisions(_)
        | reads::Answer::Setting(_)
        | reads::Answer::TaskText(_)
        | reads::Answer::Tasks(_)
        | reads::Answer::Page(_) => unreachable!("project proposal answer"),
    };
    deliver(env, decision, reply, view);
}

fn proposal_view(domain: &Domain, env: &Env<Limits>, flight: &Flight, proposal: &documents::Proposal) -> ways::View {
    let key = match &proposal.action {
        documents::ProposalAction::Settings { review, .. } => match review {
            Some(review) => review.key,
            None => None,
        },
        documents::ProposalAction::Batch(_)
        | documents::ProposalAction::Effect { .. }
        | documents::ProposalAction::ProcedureEffect { .. }
        | documents::ProposalAction::Amend { .. }
        | documents::ProposalAction::Widen { .. }
        | documents::ProposalAction::Release { .. } => None,
    };
    let Some(key) = key else { return decided(flight) };
    let kept = domain.forge.proposal_settings(key);
    if kept.is_empty() {
        return decided(flight);
    }
    let mut items = List::uncapped();
    for (key, edit) in &kept {
        if key.scope != flight.project {
            continue;
        }
        let edit = match edit {
            settings::Edit::Set(value) => ways::Edit::Set(value.value.clone()),
            settings::Edit::Remove | settings::Edit::Reset => ways::Edit::Remove,
        };
        items.push(ways::Item { kind: key.kind, scope: key.scope, number: key.number, edit }).expect("proposal items");
    }
    let party = domain.core.people.person(flight.sign_in, env.now, env.wall).expect("core rechecked proposal reader");
    let before = ways::derive(&super::inputs(domain, flight.project, party, &[], None, None));
    let after = ways::derive(&super::inputs(domain, flight.project, party, items.as_slice(), None, None));
    ways::View::Proposal { before, after }
}

fn proposal_name(flight: &Flight) -> Box<[u8]> {
    let (prefix, who) = match flight.proposer {
        Proposer::Task(who) => (b"t".as_slice(), who),
        Proposer::Party(who) => (b"p".as_slice(), who),
    };
    bytes::concat(&[prefix, Decimal::of(who).as_bytes(), b"/", Decimal::of(flight.number).as_bytes()])
}

fn decided(flight: &Flight) -> ways::View {
    not_drawn(&[b"proposal ", &proposal_name(flight), b" is decided; a decided proposal's paths aren't drawn yet."])
}

fn deliver(env: &Env<Limits>, decision: &mut Decision, reply: ReplyTo, view: ways::View) {
    super::super::emit(
        decision,
        &env.limits,
        crate::Delivery::ForgeRead { reply, answer: ForgeReadAnswer::View(Box::new(view)) },
    );
}

fn text(bytes: &[u8], limit: usize) -> Box<[u8]> {
    let mut end = bytes.len().min(limit);
    for _ in 0_u8..4 {
        if end == bytes.len() || bytes.get(end).expect("clipped byte") & 0xc0 != 0x80 {
            break;
        }
        end = end.saturating_sub(1);
    }
    bytes.get(..end).expect("bounded text").into()
}

fn fact(label: &[u8], value: &[u8]) -> ways::Fact {
    ways::Fact { label: label.into(), value: ways::cut_on_word(value, ways::VALUE) }
}
fn not_drawn(parts: &[&[u8]]) -> ways::View {
    let mut words = bytes::concat(parts);
    for byte in &mut words {
        if *byte == b'\n' || *byte == b'\r' {
            *byte = b' ';
        }
    }
    ways::View::NotDrawn(text(&words, 512))
}

//! An application-owned proposal section and dialog (client/domain/testing.md, 4).
use crate::bindings::{Bindings, Meaning};
use alloc::boxed::Box;
use jig_client::contract::{ActKind, ObjectKey, Proposal, ProposalKey};
use jig_client_test_domain::{
    Domain,
    first::{
        self,
        proposal::{Act, Choice},
    },
};
use jig_client_view::{objects::Body, phrases::jig::JigPhrase as P, words::Words};
use skein_ui::{Builder, Region, Role, Run};
const REGION: Region = Region { owner: skein_ui::Owner { number: 1 }, number: 30 };
fn document(root: &Domain, key: ProposalKey) -> Option<Proposal> {
    let client = root.jig().borrow(root.client());
    let objects = client.objects();
    let id = objects.find(ObjectKey::Proposal(key))?;
    let object = objects.get(id)?;
    let Some(Body::Proposal(proposal)) = object.body else {
        return None;
    };
    Some(*proposal)
}
pub(crate) fn section(
    b: &mut Builder,
    root: &Domain,
    key: ProposalKey,
    words: &Words,
    family: &crate::words::Words,
    bindings: &mut Bindings,
    hide: bool,
) {
    let Some(proposal) = document(root, key) else {
        return;
    };
    b.open(Role::Region);
    b.region(REGION);
    b.name(&family.say(crate::words::Word::Proposal, &[]));
    b.text(Run::Plain, proposal.reason.bytes.bytes());
    for (kind, accept, phrase) in [(ActKind::Accept, true, P::ActAccept), (ActKind::Reject, false, P::ActReject)] {
        if jig_client::facts::offer(proposal.acts.as_slice(), kind).is_some() {
            facts(b, &proposal, kind, words, hide);
            button(
                b,
                words,
                phrase,
                Act::Review(Choice { project: proposal.project, proposal: key, accept }),
                bindings,
            );
        }
    }
    b.close();
}
pub(crate) fn confirmation(
    b: &mut Builder,
    root: &Domain,
    words: &Words,
    family: &crate::words::Words,
    bindings: &mut Bindings,
    hide: bool,
) {
    let Some(choice) = root.first().proposal_confirmation() else {
        return;
    };
    let Some(proposal) = document(root, choice.proposal) else {
        return;
    };
    b.open(Role::Dialog);
    b.region(REGION);
    b.name(&family.say(crate::words::Word::ReviewProposal, &[]));
    b.text(Run::Plain, proposal.reason.bytes.bytes());
    facts(b, &proposal, if choice.accept { ActKind::Accept } else { ActKind::Reject }, words, hide);
    button(b, words, P::Confirm, Act::Confirm, bindings);
    button(b, words, P::Dismiss, Act::Dismiss, bindings);
    b.close();
}
fn button(b: &mut Builder, words: &Words, phrase: P, act: Act, bindings: &mut Bindings) {
    b.open(Role::Button);
    b.name(&words.say(phrase, &[]));
    b.bind(bindings.insert(Meaning::Action(first::Act::Proposal(act))));
    b.close();
}
fn facts(b: &mut Builder, proposal: &Proposal, kind: ActKind, words: &Words, hide: bool) {
    if hide {
        return;
    }
    let document = jig_client::contract::ReadValue::Proposal(Box::new(proposal.clone()));
    let facts = jig_client::facts::decision(Some(&document), kind);
    for text in jig_client::words::decision(&facts, words.book(), words.language()).into_boxed() {
        b.text(Run::Plain, &text);
    }
}

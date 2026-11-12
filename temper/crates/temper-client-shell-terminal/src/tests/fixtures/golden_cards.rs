//! Documents for the hand-recorded card examples; the production cards draw them.
use jig_client::contract as c;
use skein_lib::List;
use skein_ui::Builder;
const BUDGET: u64 = 40;
const LEFT: u64 = 120;
const HOLD_REVISION: u64 = 7;
const WINDOW: u32 = 2;
const TRIES: u32 = 2;
pub fn proposal(builder: &mut Builder) {
    let mut proposal = jig_client_kit::stories::proposal();
    proposal.reason = c::DocumentText::complete(jig_client_kit::stories::text(b"It touches sign-in."));
    proposal.action = c::ProposalAction::Batch(List::with_capacity(0));
    proposal.funding = jig_client_kit::test::list([c::FundingOffer {
        source: c::Funder::Pool { project: 1, person: 7, period: 3 },
        needed: BUDGET,
        left: Some(LEFT),
        after: Some(LEFT - BUDGET),
    }]);
    let page = c::ReadValue::Proposal(Box::new(proposal));
    let words = super::page_words();
    temper_client_view::cards::proposal_at(builder, &page, &words, WINDOW, temper_client_view::cards::Place::Inbox);
}
pub fn held(builder: &mut Builder) {
    let mut task = jig_client_kit::stories::task();
    task.hold = Some(Box::new(c::ChipHold {
        placed: None,
        words: None,
        circle: None,
        account: None,
        source: None,
        reason: c::HoldCategory::Failures { class: c::HoldFailure::Run },
        revision: HOLD_REVISION,
        before: c::PriorPhase::Active(c::RunStage::Idle),
        escalation: None,
        tries: c::FailureTries { transient: 0, permanent: 0, run: TRIES, agent: 0, lost: 0, invalid: 0, infrastructure: 0 },
        hold_wait_since: None,
    }));
    task.acts = List::with_capacity(1);
    task.acts
        .push(c::Offered {
            act: c::ActKind::Release,
            admission: c::Admission::Allowed,
            effect: Some(c::Effect::Release(c::ReleaseEffect::RunAgain)),
        })
        .expect("release offer fits");
    let page = c::ReadValue::Task(Box::new(task));
    let words = super::page_words();
    temper_client_view::cards::proposal(builder, &page, &words, WINDOW);
    temper_client_view::cards::decision(builder, &words, Some(&page), c::ActKind::Release);
}
pub fn report(builder: &mut Builder, bytes: &[u8]) {
    let report = c::ResultValue::Report(c::DocumentText::complete(jig_client_kit::stories::text(bytes)));
    temper_client_view::cards::value(builder, &report, &super::page_words());
}

/// The synthetic inbox keeps the shared reason line with its card examples.
pub fn why(builder: &mut Builder, kind: &[u8]) {
    use temper_client_domain::pages::{inbox::Card, needs::Need};
    let group = match kind {
        b"Proposal" => c::InboxGroup::Proposals,
        b"Held" => c::InboxGroup::Escalations,
        b"Question" => c::InboxGroup::Questions,
        b"Person task" => c::InboxGroup::PersonTasks,
        _ => unreachable!("needing fixture card"),
    };
    let root = temper_client_domain::unlinked(&crate::limits::native().client, 16);
    let card = Card {
        object: c::ObjectKey::Task(9),
        group,
        project: Some(1),
        position: None,
        unread: None,
        handle: jig_client::Handle { asker: jig_client::Asker::JIG, number: 1 },
        page: None,
        whole: None,
        words: None,
        key: None,
        outcome: None,
    };
    if let Some(why) = temper_client_view::shared::why(&root, &super::page_words(), Need::Card(&card)) {
        builder.open(skein_ui::Role::Paragraph);
        builder.style(temper_client_view::look::WHY);
        builder.text(skein_ui::Run::Plain, &why);
        builder.close();
    }
}

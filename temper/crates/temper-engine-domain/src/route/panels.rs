//! Make the forge's whole document in the decision that asked (domain/routing.md, 1).
use super::{Decision, Delivery, Domain, Env, Limits, Queue, Token, core, emit, environment_core};
use alloc::boxed::Box;
use jig_domain_people::ApplicationScope;
use jig_domain_views as views;

#[expect(clippy::too_many_arguments, reason = "the core document ask carries the watcher and its party")]
pub(crate) fn document(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    connector: u16,
    watcher: Token,
    scope: ApplicationScope,
    subject: Token,
    party: u64,
) {
    let mut ways = match scope {
        ApplicationScope::Project(project)
            if connector == domain.config.forge_connector
                && subject == Token::new(crate::ways::subject(project))
                && domain.core.people.role(party, project).is_some() =>
        {
            Some(super::ways::document(domain, project, party))
        }
        ApplicationScope::Project(_) | ApplicationScope::Own | ApplicationScope::Deployment => None,
    };
    let mut approvals = if connector == domain.config.forge_connector
        && scope == ApplicationScope::Own
        && subject == Token::new(crate::APPROVALS_SUBJECT)
    {
        Some(super::approvals::document(domain, env, party))
    } else {
        None
    };
    let mut trials = if connector == domain.config.forge_connector
        && scope == ApplicationScope::Own
        && subject == Token::new(crate::TRIALS_SUBJECT)
    {
        Some(trial_rows(domain, &env.limits, party))
    } else {
        None
    };
    let mut forges = if connector == domain.config.forge_connector
        && scope == ApplicationScope::Deployment
        && subject == Token::new(0)
    {
        Some(domain.forge.forges())
    } else {
        None
    };
    let mut panel = if connector == domain.config.forge_connector {
        domain.forge.document(subject.raw(), scope, party, env.wall, &env.limits.forge)
    } else {
        None
    };
    let outcome = if panel.is_some() || forges.is_some() || trials.is_some() || approvals.is_some() || ways.is_some() {
        views::Made::Ready
    } else {
        views::Made::Gone { why: views::End::Gone }
    };
    let mut requests = core::step(
        &mut domain.core,
        &environment_core(env),
        core::Event::View(views::Event::Made { watcher, outcome }),
    );
    for _ in 0..requests.len() {
        let request = requests.pop().expect("document completion output");
        if let core::Request::Held(held) = &request
            && let core::Held::ApplicationDelivery { watcher: due, connector: owner, subject: named } = &**held
        {
            assert!(*owner == connector && *named == subject, "document keeps its subject");
            let watcher = *due;
            emit(
                decision,
                &env.limits,
                if let Some(document) = ways.take() {
                    Delivery::Ways { watcher, document: Box::new(document) }
                } else if let Some(document) = approvals.take() {
                    Delivery::Approvals { watcher, document: Box::new(document) }
                } else {
                    match trials.take() {
                        Some(rows) => Delivery::Trials { watcher, rows },
                        None => match forges.take() {
                            Some(forges) => Delivery::Forges { watcher, forges },
                            None => match panel.take().expect("one whole ready document") {
                                temper_engine_domain_forge::Document::Change(panel) => {
                                    Delivery::Panel { watcher, panel: Box::new(panel) }
                                }
                                document @ (temper_engine_domain_forge::Document::Landing(_)
                                | temper_engine_domain_forge::Document::Landings(_)
                                | temper_engine_domain_forge::Document::Follows(_)
                                | temper_engine_domain_forge::Document::Aways(_)
                                | temper_engine_domain_forge::Document::Decisions(_)
                                | temper_engine_domain_forge::Document::Status(_)
                                | temper_engine_domain_forge::Document::Drafts(_)
                                | temper_engine_domain_forge::Document::Changes(_)
                                | temper_engine_domain_forge::Document::Dismissals(_)) => {
                                    Delivery::ForgeDocument { watcher, document: Box::new(document) }
                                }
                            },
                        },
                    }
                },
            );
        } else {
            let mut forward = Queue::with_capacity(1);
            forward.push(request);
            super::route_core_requests(domain, env, decision, forward);
        }
    }
    assert!(
        panel.is_none() && forges.is_none() && trials.is_none() && approvals.is_none() && ways.is_none(),
        "every ready document was taken where it was made"
    );
}

fn trial_rows(domain: &Domain, limits: &Limits, person: u64) -> Box<[temper_engine_domain_trials::Row]> {
    let mut ordered = skein_lib::Map::with_capacity(crate::limits::trial_limits(limits).rows);
    for row in domain.trials.rows(person) {
        let key = (u64::MAX.saturating_sub(row.began.as_nanos()), row.account);
        ordered.insert(key, row).expect("one row per account");
    }
    let mut rows = skein_lib::List::with_capacity(ordered.len());
    for (_, row) in &ordered {
        rows.push(temper_engine_domain_trials::Row {
            account: row.account,
            person: row.person,
            task: row.task,
            began: row.began,
            took: row.took,
            step: row.step,
            state: row.state,
        })
        .expect("bounded trials document");
    }
    rows.into_boxed()
}

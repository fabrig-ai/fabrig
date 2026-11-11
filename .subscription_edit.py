from pathlib import Path
b=Path('temper/crates/temper-engine-domain-forge/src')
p=b/'domain.rs';s=p.read_text();start=s.index('        if let Some(tasks)',s.index('    pub fn subscription_problem('));end=s.index('\n    /// Current subscriber',start)
s=s[:start]+'''        let problem = admission::subscription_problem(self, limits, subscription)?;
        let configured = match problem.failure {
            Some((_, crate::RefusalCause::Limit { configured, .. })) => Some(u32::try_from(configured).expect("u32 limit")),
            Some((_, crate::RefusalCause::Unadopted | crate::RefusalCause::InvalidName | crate::RefusalCause::Backend(_))) | None => None,
        };
        Some((problem.words, configured))
    }
''' + s[end:]
s=s.replace('''    if !d.can_subscribe(&env.limits, &subscription) {
        emit(out, Request::Refused { task });
        return;
    }''','''    if let Some(problem) = admission::subscription_problem(d, &env.limits, &subscription) {
        match problem.failure {
            Some((resource, cause)) => emit(out, Request::ResourceRefused { task, operation: b"subscribe", resource, cause }),
            None => emit(out, Request::Refused { task }),
        }
        return;
    }''');p.write_text(s)
p=b/'domain/admission.rs';s=p.read_text();start=s.index('/// Admit a landing branch');s=s[:start]+'''pub(super) struct SubscriptionProblem {
    pub words: &'static [u8],
    pub failure: Option<Failed>,
}

fn subscription_bound(words: &'static [u8], member: &'static str, configured: u32, needed: usize, resource: Option<Name>) -> SubscriptionProblem {
    SubscriptionProblem { words, failure: Some((resource, RefusalCause::Limit {
        member, configured: u64::from(configured), needed: skein_lib::len_u64(needed),
    })) }
}

pub(super) fn subscription_problem(d: &Domain, limits: &Limits, subscription: &crate::Subscriber) -> Option<SubscriptionProblem> {
    use crate::Topic;
    let (repository, what) = match &subscription.topic {
        Topic::Landings { repository, branch } => {
            let resource = Resource { repository: *repository, what: skein_forge_client::What::Branch(branch.clone()) };
            let name = super::from_client(&resource, limits);
            (*repository, name)
        }
        Topic::Ci { repository, .. } => (*repository, Some(Name { forge: repository.forge, repository: repository.repository, what: What::Repository })),
        Topic::Pull { repository, number } | Topic::Participation { repository, number } => (*repository, Some(Name { forge: repository.forge, repository: repository.repository, what: What::Pull(*number) })),
    };
    if let Some(tasks) = &subscription.goal_tasks
        && tasks.len() > skein_lib::usize_of(limits.issue_policy.plan_items) {
        return Some(subscription_bound(b"goal_tasks exceeds issue_policy.plan_items", "forge.issue_policy.plan_items", limits.issue_policy.plan_items, tasks.len(), what));
    }
    if subscription.number == 0 {
        return Some(SubscriptionProblem { words: b"subscription number must be positive", failure: None });
    }
    if subscription.paths.len() > skein_lib::usize_of(limits.paths_per_subscription) {
        return Some(subscription_bound(b"paths exceeds paths_per_subscription", "forge.paths_per_subscription", limits.paths_per_subscription, subscription.paths.len(), what));
    }
    match d.subscriptions.get(&(subscription.task, subscription.topic.clone())) {
        Some(existing) if existing.number != subscription.number => {
            return Some(SubscriptionProblem { words: b"duplicate topic; keep the existing subscription or unsubscribe it first", failure: None });
        }
        None if d.subscriptions.len() == limits.subscriptions => {
            return Some(subscription_bound(b"forge subscription room exhausted: subscriptions", "forge.subscriptions", limits.subscriptions, skein_lib::usize_of(d.subscriptions.len()).saturating_add(1), what));
        }
        Some(_) | None => {}
    }
    for path in &subscription.paths {
        if path.len() > skein_lib::usize_of(limits.name_bytes) {
            return Some(subscription_bound(b"path exceeds name_bytes", "forge.name_bytes", limits.name_bytes, path.len(), what));
        }
    }
    if let Topic::Landings { branch, .. } = &subscription.topic
        && branch.len() > skein_lib::usize_of(limits.name_bytes) {
        return Some(subscription_bound(b"branch exceeds name_bytes", "forge.name_bytes", limits.name_bytes, branch.len(), what));
    }
    if !d.repositories.contains_key(&repository) {
        return Some(SubscriptionProblem { words: b"repository is not adopted; select an adopted owner/name", failure: Some((what, RefusalCause::Unadopted)) });
    }
    match &subscription.topic {
        Topic::Landings { branch, .. } => match landing_subscription_problem(d, limits, repository, branch) {
            Ok(()) => None,
            Err(failure) => Some(SubscriptionProblem { words: b"landing branch watch does not fit forge resource limits", failure: Some(failure) }),
        },
        Topic::Ci { .. } | Topic::Pull { .. } | Topic::Participation { .. } => None,
    }
}

''' +s[start:];p.write_text(s)

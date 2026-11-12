//! Translations between the core's child vocabularies
//! (jig/docs/design/domain/engine/README.md, 3; jig/docs/design/domain/authority.md, 2–4).

use jig_domain_authority as authority;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::List;

/// Move a party's bounded pattern into authority's equivalent vocabulary.
#[must_use]
pub fn pattern_to_authority(value: people::Pattern) -> authority::Pattern {
    authority::Pattern {
        segments: value.segments,
        last: match value.last {
            people::Last::Exact(word) => authority::Last::Exact(word),
            people::Last::Open(word) => authority::Last::Open(word),
        },
    }
}

#[must_use]
pub fn authority_numbers(numbers: tasks::Numbers) -> authority::Numbers {
    authority::Numbers {
        budget: numbers.budget,
        spent: numbers.spent,
        spent_below: numbers.spent_below,
        reserved: numbers.reserved,
    }
}

#[must_use]
pub fn tasks_authority(value: &tasks::Authority) -> authority::Authority {
    let mut grants = List::with_capacity(u32::try_from(value.grants.len()).expect("validated task grants"));
    for grant in &value.grants {
        grants
            .push(authority::Grant {
                connector: grant.connector,
                kind: grant.kind,
                pattern: task_pattern_to_authority(&grant.pattern),
            })
            .expect("grant capacity");
    }
    let mut note_resources =
        List::with_capacity(u32::try_from(value.note_resources.len()).expect("validated task note scopes"));
    for scope in &value.note_resources {
        note_resources
            .push(authority::ResourceScope {
                connector: scope.connector,
                pattern: task_pattern_to_authority(&scope.pattern),
            })
            .expect("note scope capacity");
    }
    let mut kinds = List::with_capacity(u32::try_from(value.delegation.kinds.len()).expect("validated task executors"));
    for kind in &value.delegation.kinds {
        kinds
            .push(match kind {
                tasks::AuthorityExecutor::Charter(number) => authority::Executor::Charter(*number),
                tasks::AuthorityExecutor::Procedure(number) => authority::Executor::Procedure(*number),
                tasks::AuthorityExecutor::Role(number) => authority::Executor::Role(*number),
                tasks::AuthorityExecutor::Job(number) => authority::Executor::Job(*number),
            })
            .expect("executor capacity");
    }
    authority::Authority {
        tools: authority::Tools(value.tools.0),
        models: authority::Models::from_parts(value.models.bits(), value.models.is_all()),
        grants: grants.into_boxed(),
        delegation: authority::Delegation {
            kinds: kinds.into_boxed(),
            tasks: value.delegation.tasks,
            depth: value.delegation.depth,
        },
        budget: authority::Budget { spend: value.budget.spend, deadline: value.budget.deadline },
        notes: authority::Scopes(value.notes.0),
        note_resources: note_resources.into_boxed(),
    }
}

/// Copy authority into the task vocabulary for a connector-owned delegate.
#[must_use]
pub fn task_authority(value: &authority::Authority) -> tasks::Authority {
    let mut grants = List::with_capacity(u32::try_from(value.grants.len()).expect("validated authority grants"));
    for grant in &value.grants {
        grants
            .push(tasks::Grant {
                connector: grant.connector,
                kind: grant.kind,
                pattern: authority_pattern_to_tasks(&grant.pattern),
            })
            .expect("grant capacity");
    }
    let mut note_resources =
        List::with_capacity(u32::try_from(value.note_resources.len()).expect("validated note scopes"));
    for scope in &value.note_resources {
        note_resources
            .push(tasks::ResourceScope {
                connector: scope.connector,
                pattern: authority_pattern_to_tasks(&scope.pattern),
            })
            .expect("note scope capacity");
    }
    let mut kinds = List::with_capacity(u32::try_from(value.delegation.kinds.len()).expect("validated executors"));
    for kind in &value.delegation.kinds {
        kinds
            .push(match kind {
                authority::Executor::Charter(number) => tasks::AuthorityExecutor::Charter(*number),
                authority::Executor::Procedure(number) => tasks::AuthorityExecutor::Procedure(*number),
                authority::Executor::Role(number) => tasks::AuthorityExecutor::Role(*number),
                authority::Executor::Job(number) => tasks::AuthorityExecutor::Job(*number),
            })
            .expect("executor capacity");
    }
    tasks::Authority {
        tools: tasks::Tools(value.tools.0),
        models: tasks::Models::from_parts(value.models.bits(), value.models.is_all()),
        grants: grants.into_boxed(),
        delegation: tasks::Delegation {
            kinds: kinds.into_boxed(),
            tasks: value.delegation.tasks,
            depth: value.delegation.depth,
        },
        budget: tasks::Budget { spend: value.budget.spend, deadline: value.budget.deadline },
        notes: tasks::Scopes(value.notes.0),
        note_resources: note_resources.into_boxed(),
    }
}

/// Copy a task's resource pattern into the authority vocabulary.
pub fn task_pattern_to_authority(value: &tasks::Pattern) -> authority::Pattern {
    authority::Pattern {
        segments: value.segments.clone(),
        last: match &value.last {
            tasks::Last::Exact(bytes) => authority::Last::Exact(bytes.clone()),
            tasks::Last::Open(bytes) => authority::Last::Open(bytes.clone()),
        },
    }
}

/// Copy an authority resource pattern into the task vocabulary.
pub fn authority_pattern_to_tasks(value: &authority::Pattern) -> tasks::Pattern {
    tasks::Pattern {
        segments: value.segments.clone(),
        last: match &value.last {
            authority::Last::Exact(bytes) => tasks::Last::Exact(bytes.clone()),
            authority::Last::Open(bytes) => tasks::Last::Open(bytes.clone()),
        },
    }
}

pub fn grant_to_authority(value: people::Grant) -> authority::Grant {
    authority::Grant { connector: value.connector, kind: value.kind, pattern: pattern_to_authority(value.pattern) }
}

pub fn grant_to_people(value: &authority::Grant) -> people::Grant {
    people::Grant { connector: value.connector, kind: value.kind, pattern: pattern_to_people(&value.pattern) }
}

/// Copy an authority pattern into the party vocabulary.
#[must_use]
pub fn pattern_to_people(value: &authority::Pattern) -> people::Pattern {
    people::Pattern {
        segments: value.segments.clone(),
        last: match &value.last {
            authority::Last::Exact(word) => people::Last::Exact(word.clone()),
            authority::Last::Open(word) => people::Last::Open(word.clone()),
        },
    }
}

#[must_use]
pub fn people_authority(value: people::Authority) -> Option<authority::Authority> {
    let mut grants = List::with_capacity(u32::try_from(value.grants.len()).ok()?);
    for grant in value.grants {
        grants.push(grant_to_authority(grant)).ok()?;
    }
    let mut note_resources = List::with_capacity(u32::try_from(value.note_resources.len()).ok()?);
    for scope in value.note_resources {
        note_resources
            .push(authority::ResourceScope { connector: scope.connector, pattern: pattern_to_authority(scope.pattern) })
            .ok()?;
    }
    let mut kinds = List::with_capacity(u32::try_from(value.delegation.kinds.len()).ok()?);
    for kind in value.delegation.kinds {
        kinds
            .push(match kind {
                people::Executor::Charter(number) => authority::Executor::Charter(number),
                people::Executor::Procedure(number) => authority::Executor::Procedure(number),
                people::Executor::Job(number) => authority::Executor::Job(number),
                people::Executor::Role(number) => authority::Executor::Role(number),
            })
            .ok()?;
    }
    Some(authority::Authority {
        tools: authority::Tools(value.tools),
        models: match value.models {
            Some(models) => authority::Models::from_parts(models.bits(), models.is_all()),
            None => authority::Models::NONE,
        },
        grants: grants.into_boxed(),
        delegation: authority::Delegation {
            kinds: kinds.into_boxed(),
            tasks: value.delegation.tasks,
            depth: value.delegation.depth,
        },
        budget: authority::Budget { spend: value.spend, deadline: value.deadline },
        notes: authority::Scopes(value.notes),
        note_resources: note_resources.into_boxed(),
    })
}

/// Copy bounded authority into the party vocabulary.
#[must_use]
pub fn authority_to_people(value: &authority::Authority) -> Option<people::Authority> {
    let mut grants = List::with_capacity(u32::try_from(value.grants.len()).ok()?);
    for grant in &value.grants {
        grants.push(grant_to_people(grant)).ok()?;
    }
    let mut note_resources = List::with_capacity(u32::try_from(value.note_resources.len()).ok()?);
    for scope in &value.note_resources {
        note_resources
            .push(people::ResourceScope { connector: scope.connector, pattern: pattern_to_people(&scope.pattern) })
            .ok()?;
    }
    let mut kinds = List::with_capacity(u32::try_from(value.delegation.kinds.len()).ok()?);
    for kind in &value.delegation.kinds {
        kinds
            .push(match kind {
                authority::Executor::Charter(number) => people::Executor::Charter(*number),
                authority::Executor::Procedure(number) => people::Executor::Procedure(*number),
                authority::Executor::Role(number) => people::Executor::Role(*number),
                authority::Executor::Job(number) => people::Executor::Job(*number),
            })
            .ok()?;
    }
    Some(people::Authority {
        tools: value.tools.0,
        models: Some(people::Models::from_parts(value.models.bits(), value.models.is_all())),
        grants: grants.into_boxed(),
        delegation: people::Delegation {
            kinds: kinds.into_boxed(),
            tasks: value.delegation.tasks,
            depth: value.delegation.depth,
        },
        spend: value.budget.spend,
        deadline: value.budget.deadline,
        notes: value.notes.0,
        note_resources: note_resources.into_boxed(),
    })
}

#[must_use]
pub fn requirement_to_authority(value: people::Requirement) -> authority::Requirement {
    authority::Requirement {
        connector: value.connector,
        kind: value.kind,
        pattern: pattern_to_authority(value.pattern),
        judge: authority::Judge {
            connector: value.judge.connector,
            requirement: value.judge.requirement,
            parameters: value.judge.parameters,
        },
        guard: match value.guard {
            people::Guard::Guarded => authority::Guard::Guarded,
            people::Guard::Observed { freshness } => authority::Guard::Observed { freshness },
        },
        must_be_guarded: value.must_be_guarded,
    }
}

pub fn requirement_to_people(value: &authority::Requirement) -> people::Requirement {
    people::Requirement {
        connector: value.connector,
        kind: value.kind,
        pattern: pattern_to_people(&value.pattern),
        judge: people::Judge {
            connector: value.judge.connector,
            requirement: value.judge.requirement,
            parameters: value.judge.parameters,
        },
        guard: match value.guard {
            authority::Guard::Guarded => people::Guard::Guarded,
            authority::Guard::Observed { freshness } => people::Guard::Observed { freshness },
        },
        must_be_guarded: value.must_be_guarded,
    }
}

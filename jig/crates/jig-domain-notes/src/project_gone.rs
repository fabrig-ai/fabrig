//! Forget a deleted project's cached indexes (domain/engine.md, 10;
//! jig/docs/design/domain/core.md, 11). The core erases stored rows.
use crate::{Domain, Scope};
use skein_lib::List;

pub(crate) fn project_gone(domain: &mut Domain, project: u32) {
    let mut scopes = List::with_capacity(domain.indexes.len());
    for (scope, _) in &domain.indexes {
        let names_project = match scope {
            Scope::Project { project: owner }
            | Scope::Goal { project: owner, .. }
            | Scope::Resources { project: owner, .. } => *owner == project,
            Scope::Deployment => false,
        };
        if names_project {
            scopes.push(scope.clone()).expect("bounded cached scopes");
        }
    }
    for scope in scopes.into_boxed() {
        domain.invalidate(&scope);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skein_lib::{Env, Map, Queue, Time, Wall};

    #[test]
    fn a_gone_projects_cached_scopes_are_dropped() {
        let limits = crate::Limits {
            scopes: 5,
            entries_per_scope: 2,
            pattern_bytes: 16,
            description_bytes: 16,
            body_bytes: 32,
            references: 2,
            load_rows: 2,
            lines: 2,
            recalled: 2,
        };
        let mut domain = Domain::new(&limits);
        let gone = [
            Scope::Project { project: 3 },
            Scope::Goal { project: 3, goal: 8 },
            Scope::Resources {
                project: 3,
                connector: 1,
                pattern: crate::Pattern { segments: Box::new([]), last: crate::Last::Open(Box::new([])) },
            },
        ];
        for scope in gone.iter().chain([Scope::Deployment, Scope::Project { project: 4 }].iter()) {
            domain
                .indexes
                .insert(scope.clone(), crate::read::Cached { lines: Map::with_capacity(2), used: 0 })
                .unwrap();
        }
        let mut out = Queue::uncapped();
        crate::step(
            &mut domain,
            &Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            crate::Event::ProjectGone { project: 3 },
            &mut out,
        );
        for scope in gone {
            assert!(!domain.holds(&scope));
        }
        assert!(domain.holds(&Scope::Deployment));
        assert!(domain.holds(&Scope::Project { project: 4 }));
        assert!(out.is_empty());
    }
}

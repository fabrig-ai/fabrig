//! Standing landing requirements over held rules
//! (temper/docs/design/protocol/configuration.md, 13.3;
//! temper/docs/design/domain/forge.md, 8.7).
use alloc::boxed::Box;
use skein_lib::List;
use temper_engine_domain_forge_change as change;

use crate::{
    ChangeEvidence, Criterion, Domain, GateCandidate, JudgeFreshness, JudgeVerdict, Limits, Repository, Requires,
    Reviewer, domain::branch_resource_name, gates, judge, landing, settings,
};

impl Domain {
    /// Read the procedure's waits from the deployment's and project's held rules.
    #[must_use]
    pub fn held_requires(&self, repository: &Repository, branch: &[u8]) -> Requires {
        let Some(name) = branch_resource_name(repository, branch) else {
            return Requires { ci: true, up_to_date: true };
        };
        let (deployment, project) = self.landing_rules(repository.project);
        let mut held = List::uncapped();
        for rules in [deployment, project] {
            for rule in rules {
                held.push(rule.clone()).expect("held rules bound the scratch list");
            }
        }
        settings::requires(held.as_slice(), &name)
    }

    /// Select every covering rule's gates, with advisory rules holding nothing.
    #[must_use]
    pub fn held_gates(&self, limits: &Limits, repository: &Repository, branch: &[u8]) -> Option<Box<[change::Gate]>> {
        let name = branch_resource_name(repository, branch)?;
        let (deployment, project) = self.landing_rules(repository.project);
        let capacity = limits.landing_rules.checked_mul(limits.change_policy.gates)?.checked_mul(2)?;
        let mut candidates = List::with_capacity(capacity);
        let mut required = List::with_capacity(capacity);
        for rules in [deployment, project] {
            for held in rules {
                let rule = &held.rule;
                if !gates::covers(&rule.pattern, &name) {
                    continue;
                }
                for gate in &rule.gates {
                    candidates
                        .push(GateCandidate {
                            number: gate.number,
                            parameters: 0,
                            blocking: rule.enforce && gate.blocking,
                            project_scope: held.scope != 0,
                            freshness: match gate.freshness {
                                JudgeFreshness::Exact => change::Freshness::Exact,
                                JudgeFreshness::Clean => change::Freshness::Clean,
                            },
                        })
                        .ok()?;
                    required.push(rule.enforce).ok()?;
                }
            }
        }
        let mut selected = gates::selected(limits, repository, candidates.as_slice(), required.as_slice())?;
        // Selection keeps the first occurrence; later enforcing rules still add their requirements.
        for gate in &mut selected {
            for candidate in candidates.as_slice() {
                if gate.number == u64::from(candidate.number) && candidate.blocking {
                    gate.blocking = true;
                    if candidate.freshness == change::Freshness::Exact {
                        gate.freshness = change::Freshness::Exact;
                    }
                }
            }
        }
        Some(selected)
    }

    pub(crate) fn judge_held_landing(
        &self,
        up_to_date: bool,
        task: u64,
        head: skein_forge::api::Commit,
        evidence: &ChangeEvidence,
        reviewers: &[Reviewer],
    ) -> Option<JudgeVerdict> {
        let landing = self.landing.rows.get(&task);
        let mut change_task = task;
        if let Some(row) = landing {
            let mut found = false;
            for member in &row.landing.members {
                if member.candidate == Some(head) {
                    change_task = member.change;
                    found = true;
                    break;
                }
            }
            if !found {
                return None;
            }
        }
        let row = self.changes.get(&change_task)?;
        let repository = self.repositories.get(&row.repository)?;
        let name = branch_resource_name(repository, &row.base)?;
        let (deployment, project) = self.landing_rules(repository.project);
        let mut criteria = List::uncapped();
        for rules in [deployment, project] {
            for held in rules {
                let rule = &held.rule;
                if !rule.enforce || !gates::covers(&rule.pattern, &name) {
                    continue;
                }
                if up_to_date {
                    if rule.up_to_date {
                        criteria.push(Criterion::UpToDate).ok()?;
                        break;
                    }
                    continue;
                }
                if rule.ci {
                    criteria.push(Criterion::Ci).ok()?;
                }
                for gate in &rule.gates {
                    if gate.blocking {
                        criteria.push(Criterion::Gate { number: gate.number, freshness: gate.freshness }).ok()?;
                    }
                }
                for approval in &rule.approvals {
                    criteria
                        .push(Criterion::Approval {
                            role: approval.role,
                            people: approval.people,
                            freshness: approval.freshness,
                        })
                        .ok()?;
                }
            }
            if up_to_date && !criteria.is_empty() {
                break;
            }
        }
        let mut result = JudgeVerdict::Met;
        for criterion in criteria.into_boxed() {
            let verdict = match landing {
                Some(landing) => landing::judge(self, landing, criterion, head, evidence, reviewers)?,
                None => judge::judge(criterion, repository, head, &row.change.clean, evidence, reviewers),
            };
            match verdict {
                JudgeVerdict::Refuse => return Some(JudgeVerdict::Refuse),
                JudgeVerdict::Wait => result = JudgeVerdict::Wait,
                JudgeVerdict::Met => {}
            }
        }
        Some(result)
    }
}

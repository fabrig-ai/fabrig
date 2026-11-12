//! Standing landing requirements over held rules
//! (temper/docs/design/protocol/configuration/settings.md, 3;
//! temper/docs/design/domain/forge/change/gates.md, 2).
use alloc::boxed::Box;
use skein_lib::List;
use temper_engine_domain_forge_change as change;

use crate::{
    ChangeEvidence, Criterion, Domain, GateCandidate, JudgeFreshness, JudgeVerdict, Limits, Repository, Requires,
    Reviewer, Touched, domain::branch_resource_name, gates, judge, landing, settings,
};

impl Domain {
    /// Read the procedure's waits from the deployment's and project's held rules.
    #[must_use]
    pub fn held_requires(&self, repository: &Repository, branch: &[u8], touched: Touched) -> Requires {
        let held = self.landing_rules(repository.project);
        let Some(_) = branch_resource_name(repository, branch) else {
            return Requires { ci: true, up_to_date: true };
        };
        settings::requires(&held, repository, branch, touched)
    }

    /// Select every covering rule's gates, with advisory rules holding nothing.
    #[must_use]
    pub fn held_gates(
        &self,
        limits: &Limits,
        repository: &Repository,
        branch: &[u8],
        touched: Touched,
    ) -> Option<Box<[change::Gate]>> {
        branch_resource_name(repository, branch)?;
        let held = self.landing_rules(repository.project);
        let capacity = limits.landing_rules.checked_mul(limits.change_policy.gates)?.checked_mul(2)?;
        let mut candidates = List::with_capacity(capacity);
        let mut required = List::with_capacity(capacity);
        for held in &held {
            let rule = &held.rule;
            if !gates::covers(rule, repository, branch, touched) {
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

    /// Keep the change's own gates beside the rules selected at this head.
    pub(crate) fn step_gates(
        &self,
        limits: &Limits,
        row: &crate::ChangeRow,
        repository: &Repository,
        touched: Touched,
    ) -> Option<Box<[change::Gate]>> {
        let selected = self.held_gates(limits, repository, &row.base, touched)?;
        let rules = self.landing_rules(repository.project);
        let mut gates: List<change::Gate> = List::with_capacity(limits.change_policy.gates);
        for gate in &row.change.gates {
            let mut by_path = false;
            for held in &rules {
                if held.rule.sets.is_empty() || !gates::covers(&held.rule, repository, &row.base, Touched::UNKNOWN) {
                    continue;
                }
                for named in &held.rule.gates {
                    let check = !repository.ci && repository.checks.contains(&named.number);
                    let configured = change::Gate {
                        number: u64::from(named.number),
                        kind: if check { change::GateKind::Check } else { change::GateKind::Agent },
                        blocking: check || (held.rule.enforce && named.blocking),
                        freshness: if check {
                            change::Freshness::Exact
                        } else {
                            match named.freshness {
                                JudgeFreshness::Exact => change::Freshness::Exact,
                                JudgeFreshness::Clean => change::Freshness::Clean,
                            }
                        },
                        eager: false,
                    };
                    by_path |= *gate == configured;
                }
            }
            if !by_path {
                gates.push(*gate).ok()?;
            }
        }
        for gate in selected {
            let mut present = false;
            for own in gates.iter_mut() {
                if own.number == gate.number {
                    present = true;
                    own.blocking |= gate.blocking;
                    if gate.freshness == change::Freshness::Exact {
                        own.freshness = change::Freshness::Exact;
                    }
                    if gate.kind == change::GateKind::Check {
                        own.kind = change::GateKind::Check;
                    }
                }
            }
            if !present {
                gates.push(gate).ok()?;
            }
        }
        Some(gates.into_boxed())
    }

    pub(crate) fn judge_held_landing(
        &self,
        up_to_date: bool,
        task: u64,
        head: skein_forge::api::Commit,
        evidence: &ChangeEvidence,
        reviewers: &[Reviewer],
    ) -> Option<JudgeVerdict> {
        match self.held_landing_verdict(up_to_date, task, head, evidence, reviewers) {
            Some((verdict, _criterion)) => Some(verdict),
            None => None,
        }
    }

    /// Name the counted request only when requirement 5 first refuses on an approval
    /// of a change, never a goal landing (temper/docs/design/domain/forge/change/gates.md, 1).
    #[must_use]
    pub fn review_request(
        &self,
        task: u64,
        head: skein_forge::api::Commit,
        evidence: &ChangeEvidence,
        reviewers: &[Reviewer],
    ) -> Option<skein_forge::api::Commit> {
        if self.landing.rows.contains_key(&task) {
            return None;
        }
        let (JudgeVerdict::Refuse, Some(Criterion::Approval { role, freshness, .. })) =
            self.held_landing_verdict(false, task, head, evidence, reviewers)?
        else {
            return None;
        };
        let row = self.changes.get(&task)?;
        let (_, requested) = judge::approval_count(role, freshness, head, &row.change.clean, reviewers);
        requested
    }

    fn landing_change(&self, task: u64, head: skein_forge::api::Commit) -> Option<&crate::ChangeRow> {
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
        self.changes.get(&change_task)
    }

    /// The enforcing approvals covering the exact merge evidence, including a turn member's.
    #[must_use]
    pub fn landing_sign_offs(
        &self,
        task: u64,
        head: skein_forge::api::Commit,
        evidence: &ChangeEvidence,
        reviewers: &[Reviewer],
    ) -> Box<[crate::SignOff]> {
        let Some(row) = self.landing_change(task, head) else { return Box::new([]) };
        let Some(criteria) = self.landing_criteria(row, head, evidence, false) else { return Box::new([]) };
        let mut approvals = List::uncapped();
        for criterion in criteria {
            match criterion {
                Criterion::Approval { role, freshness, .. } => {
                    approvals.push((role, freshness)).expect("admitted criteria");
                }
                Criterion::Ci | Criterion::UpToDate | Criterion::Gate { .. } => {}
            }
        }
        judge::sign_offs(&approvals.into_boxed(), head, &row.change.clean, reviewers)
    }

    fn landing_criteria(
        &self,
        row: &crate::ChangeRow,
        head: skein_forge::api::Commit,
        evidence: &ChangeEvidence,
        up_to_date: bool,
    ) -> Option<Box<[Criterion]>> {
        let repository = self.repositories.get(&row.repository)?;
        branch_resource_name(repository, &row.base)?;
        let rules = self.landing_rules(repository.project);
        let touched = if evidence.head == Some(head) { evidence.touched } else { Touched::UNKNOWN };
        let mut criteria = List::uncapped();
        for held in &rules {
            let rule = &held.rule;
            if !rule.enforce || !gates::covers(rule, repository, &row.base, touched) {
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
        Some(criteria.into_boxed())
    }

    fn held_landing_verdict(
        &self,
        up_to_date: bool,
        task: u64,
        head: skein_forge::api::Commit,
        evidence: &ChangeEvidence,
        reviewers: &[Reviewer],
    ) -> Option<(JudgeVerdict, Option<Criterion>)> {
        let landing = self.landing.rows.get(&task);
        let row = self.landing_change(task, head)?;
        let repository = self.repositories.get(&row.repository)?;
        let criteria = self.landing_criteria(row, head, evidence, up_to_date)?;
        let mut result = JudgeVerdict::Met;
        for criterion in criteria {
            let verdict = match landing {
                Some(landing) => landing::judge(self, landing, criterion, head, evidence, reviewers)?,
                None => judge::judge(criterion, repository, head, &row.change.clean, evidence, reviewers),
            };
            match verdict {
                JudgeVerdict::Refuse => return Some((JudgeVerdict::Refuse, Some(criterion))),
                JudgeVerdict::Wait => result = JudgeVerdict::Wait,
                JudgeVerdict::Met => {}
            }
        }
        Some((result, None))
    }
}

impl Domain {
    pub(crate) fn panel_approvals(&self, row: &crate::ChangeRow) -> Box<[crate::PanelApproval]> {
        let mut approvals = List::uncapped();
        let Some(repository) = self.repositories.get(&row.repository) else { return approvals.into_boxed() };
        let Some(_) = branch_resource_name(repository, &row.base) else { return approvals.into_boxed() };
        let held = self.landing_rules(repository.project);
        for held in &held {
            if !held.rule.enforce || !gates::covers(&held.rule, repository, &row.base, self.touched_at(row)) {
                continue;
            }
            for approval in &held.rule.approvals {
                let mut line = self.panel_approval(row, *approval);
                line.scope = held.scope;
                line.rule = held.number;
                approvals.push(line).expect("admitted rules");
            }
        }
        approvals.into_boxed()
    }

    fn panel_approval(&self, row: &crate::ChangeRow, rule: settings::RuleApproval) -> crate::PanelApproval {
        let mut line = crate::PanelApproval {
            scope: 0,
            rule: 0,
            role: rule.role,
            freshness: rule.freshness,
            required: rule.people,
            read: crate::ReviewRead::Unread,
            counted: Box::new([]),
            remaining: None,
            refused: false,
        };
        if row.terminal() {
            return line;
        }
        let Some(facts) = self.review_facts.get(&row.task) else { return line };
        let Some(head) = row.change.last_head else { return line };
        if facts.head != Some(head) {
            return line;
        }
        line.read = if facts.reviews_complete { crate::ReviewRead::Complete } else { crate::ReviewRead::Incomplete };
        let (counted, refused) =
            judge::approval_count(rule.role, rule.freshness, head, &row.change.clean, &facts.reviewers);
        let have = u32::try_from(counted.len()).expect("admitted reviewers");
        line.remaining = Some(rule.people.saturating_sub(have));
        line.counted = counted;
        line.refused = refused.is_some();
        line
    }

    pub(crate) fn panel_unmapped(&self, row: &crate::ChangeRow) -> Box<[crate::UnmappedReview]> {
        if let Some(facts) = self.review_facts.get(&row.task)
            && facts.head == row.change.last_head
            && !row.terminal()
        {
            return facts.unmapped.clone();
        }
        Box::new([])
    }
}

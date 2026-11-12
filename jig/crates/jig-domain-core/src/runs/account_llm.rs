//! The choice that holds a new task to one account (jig/docs/design/domain/accounts.md, 4.3).

use jig_domain_tasks as tasks;

use crate::Core;

use super::{coverage, llm};

impl Core {
    /// The choice that holds a new task of `charter` to `account`
    /// (jig/docs/design/domain/accounts.md, 4.3): the charter's first model, in its order
    /// within `models`, that `offer(payer, project, models)` holds with `account`; else the
    /// first whose LLM provider is the account's and whose candidates for `payer` hold it.
    /// Each model at its default effort, as a task given no choice takes it, and
    /// `account: Some(account)`. None where the account serves none of them.
    #[must_use]
    pub fn account_llm(
        &self,
        charter: u32,
        models: tasks::Models,
        payer: tasks::Payer,
        project: u32,
        account: u32,
    ) -> Option<tasks::LlmChoice> {
        let revision = self.tasks.newest();
        let policy = &self.charter_at(charter, revision)?.policy;
        let mut choice = self.tasks.default_llm_at(charter, revision, models)?;
        choice.account = Some(account);
        let candidates = llm::candidates(&choice);
        let offer = self.offer(payer, project, models);
        for candidate in &candidates {
            for offered in &offer {
                if offered.model == candidate.model && offered.accounts.as_slice().contains(&account) {
                    choice.main = *candidate;
                    return Some(llm::defaults(&self.settings, policy, choice, candidates.as_slice()));
                }
            }
        }
        let provider = self.accounts.account(account)?.provider();
        let coverage = self.account_coverage(payer, Some(project));
        for candidate in &candidates {
            let name = &self.settings.models.get(usize::from(candidate.model))?.name;
            let model = llm::charter_model(policy, name)?;
            if self.provider(model) == Some(provider)
                && self
                    .accounts
                    .candidates(
                        coverage::payer(payer),
                        provider,
                        self.settings.model_ways(model),
                        None,
                        None,
                        coverage.as_slice(),
                    )
                    .as_slice()
                    .contains(&account)
            {
                choice.main = *candidate;
                return Some(llm::defaults(&self.settings, policy, choice, candidates.as_slice()));
            }
        }
        None
    }
}

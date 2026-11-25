//! The reference cases' defaults in the real loop.
mod values;
pub use values::items;

use jig_domain_core::settings::Defaults;
use serde_json::Value;
use skein_lib::{Duration, bytes};

pub(crate) fn apply(defaults: &mut Defaults, flags: &Value, stock_period_spend: u64) {
    // Project-period budgets stay in policy 1; the deployment keeps its stock period.
    defaults.rules.period_spend = stock_period_spend;
    defaults.templates[0].contract = jig_domain_tasks::Contract::Report {
        words: u32::try_from(flags["report_words"].as_u64().expect("case report words")).expect("report words"),
    };
    if let Some(cap) = flags["cap"].as_u64() {
        defaults.templates[0].budget = jig_domain_people::TemplateBudget::AsAllowed { cap: Some(cap) };
    }
    if let Some(executor) = flags["executor"].as_u64() {
        defaults.templates[0].executor =
            jig_domain_people::TemplateExecutor::Charter(u32::try_from(executor).expect("case executor"));
    }
    if let Some(resume) = flags["resume"].as_bool() {
        defaults.charters[0].policy.resume = resume;
    }
    // Charter 2 is a copy of the ordinary chat charter in these stories, with
    // its own wait/resume or brief. The coding stock's text is not the owner brief.
    if flags["second_wait_ms"].is_number() || flags["second_resume"].is_boolean() || flags["owner_brief"] == true {
        let mut second = defaults.charters[0].clone();
        second.number = 2;
        if let Some(wait) = flags["second_wait_ms"].as_u64() {
            second.policy.waiting = Duration::from_millis(wait);
        }
        if let Some(resume) = flags["second_resume"].as_bool() {
            second.policy.resume = resume;
        }
        if flags["owner_brief"] == true {
            second.policy.instructions = bytes::copy_of(super::OLD_BRIEF.as_bytes());
        }
        defaults.charters[1] = second;
    }
}

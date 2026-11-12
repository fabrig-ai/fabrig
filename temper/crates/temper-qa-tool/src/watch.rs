//! Read a move through every parent, then attribute builds (qa/landings.md, 2–5).
use crate::{
    Git, Ledger, Refusal,
    faces::{GitFace, trailers},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GoalLanding {
    pub goal: u64,
    pub pull: Option<u64>,
    pub title: String,
}

#[must_use]
pub fn parse_landing(message: &str, branch: &str) -> Option<GoalLanding> {
    let subject = message.lines().next()?;
    let block = trailers(message);
    if let Some(goal) =
        block.iter().find_map(|(key, value)| (key == "Goal" && decimal(value)).then(|| value.parse().ok()).flatten())
    {
        let pull = block
            .iter()
            .find_map(|(key, value)| (key == "Pull-request" && decimal(value)).then(|| value.parse().ok()).flatten());
        return Some(GoalLanding { goal, pull, title: subject.into() });
    }
    let subject = subject.strip_suffix(&format!(" into {branch}"))?;
    let (title_pull, from) = subject.rsplit_once(" from ")?;
    let (owner, goal) = from.rsplit_once("/g")?;
    if owner.is_empty() || owner.contains('/') || !decimal(goal) {
        return None;
    }
    let title_pull = title_pull.strip_suffix(')')?;
    let (title, pull) = title_pull.rsplit_once(" (#")?;
    if !decimal(pull) {
        return None;
    }
    let title = title.strip_prefix("Merge pull request '")?.strip_suffix('\'')?;
    Some(GoalLanding { goal: goal.parse().ok()?, pull: Some(pull.parse().ok()?), title: title.into() })
}
fn decimal(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())
}

pub fn watch(git: &mut impl GitFace, config: &Git, ledger: &mut Ledger, now: u64) -> Result<(), Refusal> {
    let reference = if let Some(remote) = &config.fetch {
        git.fetch(remote, &config.branch)?;
        format!("refs/temper-qa/{}", config.branch)
    } else {
        format!("refs/heads/{}", config.branch)
    };
    let head = git.head(&reference)?;
    if let Some(old) = &ledger.head
        && old != &head
    {
        for commit in git.commits(old, &head)? {
            if commit.parents.len() < 2 || ledger.landings.iter().any(|landing| landing.commit == commit.name) {
                continue;
            }
            if let Some(landing) = parse_landing(&git.message(&commit.name)?, &config.branch) {
                ledger.record_landing(&commit.name, landing.goal, landing.pull, &landing.title, now);
            }
        }
    }
    ledger.record_head(&head);
    let waiting: Vec<_> = ledger.builds_without_holds().iter().map(|build| build.commit.clone()).collect();
    for commit in waiting {
        if !git.contains(&commit, &head)? {
            continue;
        }
        let mut holds = Vec::new();
        for landing in &ledger.landings {
            if landing.commit == commit || git.contains(&landing.commit, &commit)? {
                holds.push(landing.number);
            }
        }
        ledger.set_build_holds(&commit, holds, git.count(&commit)?)?;
    }
    Ok(())
}

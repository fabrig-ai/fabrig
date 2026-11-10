//! Puller work shares the forge's own connection (forge/connection.md, 6.5).
use super::{Base, Common, GitHubForge, OwnPlan};
use crate::{
    Account, Change, Event, Fact, Limits, Stopped,
    hooks::Hinted,
    installations::{Read, Work},
    puller::Action,
};
use skein_forge::api;
use skein_forge::github::{self, delivery::Delivery, plans::plan::Bounds};
use skein_lib::{Env, Queue};

impl Common {
    pub(super) fn github_log_bounds(l: &Limits, delivery: bool) -> Bounds {
        Bounds {
            page_size: github::pin::PAGE_CAP.min(l.inner),
            rows: github::pin::PAGE_CAP.min(l.inner),
            // Log pages are independent of calls and include the dialect's bounded strings.
            answer_bytes: if delivery {
                l.delivery_bytes
            } else {
                crate::limits::log_page_bytes(l).expect("checked log page")
            },
            body_bytes: l.body_bytes,
            inner: l.inner,
        }
    }
    pub(super) fn github_log_active(gh: &GitHubForge) -> bool {
        match &gh.plan {
            Some(OwnPlan::GitHubLogPage(_) | OwnPlan::GitHubLogDelivery(_)) => true,
            Some(OwnPlan::GitHubInstallations(_) | OwnPlan::GitHubMinting(_) | OwnPlan::GitHubIdentity(_)) | None => {
                false
            }
        }
    }
    pub(super) fn github_log_selected(base: &Base, gh: &GitHubForge) -> bool {
        if !super::ready(base.start) {
            return false;
        }
        let data = &gh.state;
        let reading = match data.read {
            Read::Reading(_) | Read::Changes(_) => true,
            Read::Idle | Read::Waiting { .. } => false,
        };
        let working = match data.work {
            Work::Minting { .. } | Work::Identity { .. } => true,
            Work::Idle | Work::Waiting { .. } => false,
        };
        !reading && !working && data.puller.wants_request()
    }
    pub(super) fn github_log_local(
        &mut self,
        base: &mut Base,
        gh: &mut GitHubForge,
        env: &Env<Limits>,
        up: &mut Queue<Event>,
    ) -> bool {
        let forge = base.forge.number;
        let data = &mut gh.state;
        match data.puller.local(env) {
            Action::Idle => false,
            Action::Progress => true,
            Action::Installations => {
                data.reread = true;
                true
            }
            Action::Gap { reread } => {
                data.reread |= reread;
                up.push(Event::Fact { forge, fact: Fact::Gap });
                true
            }
            Action::Hint(hint) => {
                up.push(Event::Hint(Hinted { connection: data.puller.source.expect("delivery source"), hint }));
                self.wake = Some(env.now);
                false
            }
        }
    }
    pub(super) fn github_log_failed(gh: &mut GitHubForge, env: &Env<Limits>) {
        let delivery = match &gh.plan {
            Some(OwnPlan::GitHubLogDelivery(_)) => true,
            Some(OwnPlan::GitHubLogPage(_)) => false,
            Some(OwnPlan::GitHubInstallations(_) | OwnPlan::GitHubMinting(_) | OwnPlan::GitHubIdentity(_)) | None => {
                unreachable!("active log read")
            }
        };
        let data = &mut gh.state;
        if delivery {
            data.puller.reading();
        } else {
            data.puller.finish(env, false);
        }
        gh.plan = None;
    }
    pub(super) fn github_log_error(base: &mut Base, gh: &mut GitHubForge, env: &Env<Limits>, up: &mut Queue<Event>) {
        if gh.state.status == Some(401) {
            Self::github_stop(base, gh, Stopped::KeyRefused, up);
        } else {
            gh.state.puller.finish(env, false);
        }
    }
    pub(super) fn github_log_delivery(
        &mut self,
        base: &mut Base,
        gh: &mut GitHubForge,
        answer: Result<github::log::Logged, api::Error>,
        up: &mut Queue<Event>,
    ) {
        gh.state.puller.reading();
        match answer {
            Ok(logged) => self.github_logged(base, gh, logged, up),
            Err(_) => {
                if gh.state.status == Some(401) {
                    Self::github_stop(base, gh, Stopped::KeyRefused, up);
                }
            }
        }
    }
    pub(super) fn github_logged(
        &mut self,
        base: &mut Base,
        gh: &mut GitHubForge,
        logged: github::log::Logged,
        up: &mut Queue<Event>,
    ) {
        let forge = base.forge.number;
        match &logged.delivery {
            Delivery::Installation { installation, account, change } => {
                if *change == github::delivery::Change::Repositories {
                    let account = match gh.state.rows.get(installation) {
                        Some(row) => row.account.clone(),
                        None => Account { id: account.id, login: account.login.clone() },
                    };
                    up.push(Event::Installation {
                        forge,
                        installation: *installation,
                        account: Some(account),
                        change: Change::Repositories,
                    });
                }
            }
            Delivery::Issue { .. }
            | Delivery::Comment { .. }
            | Delivery::Pull { .. }
            | Delivery::Review { .. }
            | Delivery::Remark { .. }
            | Delivery::Push { .. }
            | Delivery::Ref { .. }
            | Delivery::Ci { .. }
            | Delivery::Other => {}
        }
        if let Some(hint) = github::delivery::hint(forge, &self.owner.word, &logged.delivery) {
            if let Some(path) = logged.repository
                && let Some(row) = self.github_tables.repository(hint.repository)
            {
                // A path alone cannot supply a missing row's owner account or settings (4.2).
                if row.owner == path.owner {
                    let row = github::plans::tables::Repository { name: path.name, ..row.clone() };
                    self.github_tables.keep_repository(hint.repository, row);
                } else {
                    // Another owner may be another account, which routes the calls: the next read fills it.
                    self.github_tables.drop_repository(hint.repository);
                }
            }
            gh.state.puller.source = base.own;
            gh.state.puller.hold(hint);
        }
    }
}

//! The referee (temper/docs/design/redeploy.md, 6, 7).
use crate::{
    description::Description,
    hosts::Hosts,
    record::{Before, Journal},
};

#[doc(hidden)]
pub mod judgement;
mod reads;

pub fn before(description: &Description, hosts: &mut dyn Hosts) -> Result<Before, String> {
    let read = reads::snapshot(description, hosts);
    match read.unread.first() {
        Some(why) => Err(why.clone()),
        None => Ok(read.snapshot),
    }
}

pub fn judge(description: &Description, journal: &mut Journal, hosts: &mut dyn Hosts) -> bool {
    let mut read = reads::snapshot(description, hosts);
    let tasks: Vec<_> = judgement::tasks(&journal.record.before, &read.snapshot).into_keys().collect();
    for task in tasks {
        reads::activity(description, hosts, task, journal.record.began, &mut read);
    }
    let judged = judgement::evaluate(description, &journal.record, &read);
    for line in judged.lines {
        if journal.line(hosts.now(), &format!("temper-redeploy referee: {line}")).is_err() {
            return false;
        }
    }
    judged.failures == 0
}

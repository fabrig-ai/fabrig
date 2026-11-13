//! The referee (temper/docs/design/redeploy.md, 6, 7).
use crate::{
    description::Description,
    hosts::Hosts,
    record::{Before, Journal},
};

pub fn before(_description: &Description, _hosts: &mut dyn Hosts) -> Result<Before, String> {
    Ok(Before::default())
}
pub fn judge(_description: &Description, journal: &mut Journal, hosts: &mut dyn Hosts) -> bool {
    let _ = journal.line(hosts.now(), "temper-redeploy referee: not built");
    false
}

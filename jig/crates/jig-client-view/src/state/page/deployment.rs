//! Deployment facts and connector part references, only for its admitted page.
//! Hosts, accounts and capacity are whole owner documents. No connector value is
//! kept. See client/domain/pages.md, 7, and client/domain/root.md, 6.
use crate::{Request, Ui};
use alloc::boxed::Box;
use jig_client::contract::{ApplicationPart, Deployment};
use skein_lib::{Env, List, Queue};
/// Latest deployment document for the admitted page.
#[derive(Debug)]
pub struct DeploymentPage {
    /// The latest deployment owner facts.
    pub owner: Option<Box<Deployment>>,
}
impl DeploymentPage {
    pub(crate) const fn new() -> Self {
        Self { owner: None }
    }
}

/// Keep the whole owner value only for the admitted current deployment page.
pub(crate) fn owner_document(
    domain: &mut Ui,
    env: &Env<crate::Config>,
    doc: Box<Deployment>,
    out: &mut Queue<Request>,
    lent: &jig_client::Client,
) {
    if !domain.shows(crate::state::page::Section::Deployment, lent) {
        return;
    }
    let mut parts = List::with_capacity(lent.domain_limits().parts.saturating_add(1));
    for connector in &doc.connectors {
        if parts.room() == 0 {
            break;
        }
        parts.push(ApplicationPart::Deployment { connector: *connector }).expect("configured reference fits");
    }
    domain.deployment_page.owner = Some(doc);
    crate::state::page::project::show(domain, env, parts, out, lent);
    domain.changed();
}

/// Show the watched deployment's references without retaining its document.
pub(crate) fn watched(
    domain: &mut Ui,
    env: &Env<crate::Config>,
    out: &mut Queue<Request>,
    client: &jig_client::Client,
) {
    let projected = domain.borrow(client).deployment_page();
    let mut parts = List::with_capacity(client.domain_limits().parts.saturating_add(1));
    if let Some(owner) = projected.owner {
        for connector in &owner.connectors {
            if parts.push(ApplicationPart::Deployment { connector: *connector }).is_err() {
                break;
            }
        }
    }
    crate::state::page::project::show(domain, env, parts, out, client);
    domain.changed();
}

//! Keep: the child routing shape and exhaustive connector variants.
//! Replace: Ops connector sides, family addresses and application vocabulary.
use crate::{Domain, Fact, Limits, Request, infrastructure, observability, route};
use jig_client::contract::ApplicationPart;
use skein_lib::{Env, Queue};
pub(crate) fn show(
    root: &mut Domain,
    env: &Env<Limits>,
    part: ApplicationPart,
    scope: jig_client::contract::PartScope,
    out: &mut Queue<Request>,
) {
    let number = connector(&part);
    match jig_ops_domain_connectors::connector(number) {
        Some(jig_ops_domain_connectors::Connector::Infrastructure) => route::infrastructure(
            root,
            env,
            infrastructure::Event::Show {
                part,
                scope: match scope {
                    jig_client::contract::PartScope::Project(project) => infrastructure::Scope::Project(project),
                    jig_client::contract::PartScope::Deployment => infrastructure::Scope::Deployment,
                },
            },
            out,
        ),
        Some(jig_ops_domain_connectors::Connector::Observability) => {
            route::observability(
                root,
                env,
                observability::Event::Show {
                    part,
                    scope: match scope {
                        jig_client::contract::PartScope::Project(project) => observability::Scope::Project(project),
                        jig_client::contract::PartScope::Deployment => observability::Scope::Deployment,
                    },
                },
                out,
            );
        }
        None => out.push(Request::Fact { fact: Fact::UnknownConnector }),
    }
}
#[must_use]
pub fn connector(part: &ApplicationPart) -> u16 {
    match part {
        ApplicationPart::Connector { connector }
        | ApplicationPart::Resource { connector, .. }
        | ApplicationPart::Kind { connector, .. }
        | ApplicationPart::Panel { connector, .. }
        | ApplicationPart::Result { connector, .. }
        | ApplicationPart::OwnerEffect { connector, .. }
        | ApplicationPart::Effect { connector, .. }
        | ApplicationPart::Outbox { connector, .. }
        | ApplicationPart::Hold { connector, .. }
        | ApplicationPart::Project { connector, .. }
        | ApplicationPart::Deployment { connector } => *connector,
    }
}

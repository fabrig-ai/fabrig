//! A part routes by its connector, without reading the application's value.
use crate::{Domain, Fact, Limits, Request, first, owner::Connector, route};
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
    match Connector::of(number) {
        Some(Connector::First) => route::first(
            root,
            env,
            first::Event::Show {
                part,
                scope: match scope {
                    jig_client::contract::PartScope::Project(project) => crate::family::Scope::Project(project),
                    jig_client::contract::PartScope::Deployment => crate::family::Scope::Deployment,
                },
            },
            out,
        ),
        Some(Connector::Second) => {
            if !root.second.show(part) {
                out.push(Request::Fact { fact: Fact::PartsFull });
            }
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
        | ApplicationPart::Effect { connector, .. }
        | ApplicationPart::OwnerEffect { connector, .. }
        | ApplicationPart::Outbox { connector, .. }
        | ApplicationPart::Hold { connector, .. }
        | ApplicationPart::Project { connector, .. }
        | ApplicationPart::Deployment { connector } => *connector,
    }
}

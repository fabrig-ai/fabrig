//! The engine key of a named tools request (client/ctl/requests.md, 1).
use crate::jig::key::{Name, named};
pub use temper_client_domain_forge::key::TOOLS;
#[must_use]
pub fn engine(name: &Name) -> skein_lib::RequestKey {
    named(TOOLS, name)
}

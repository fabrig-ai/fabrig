//! The engine key of a named tools request (client/tools.md, 7).
use crate::jig::key::{Name, named};
pub use temper_client_domain::key::TOOLS;
#[must_use]
pub fn engine(name: &Name) -> skein_lib::RequestKey {
    named(TOOLS, name)
}

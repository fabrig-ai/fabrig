use crate as fixtures;
use crate::{limits, settings};
use jig_protocol_config as config;

#[derive(Clone, Copy, Debug)]
pub enum Base {
    Fixture,
    Stock,
}

#[must_use]
pub fn settings(profile: Base) -> settings::Engine {
    let mut settings = fixtures::engine();
    settings.jig.limits = match profile {
        Base::Fixture => jig_protocol_config_fixtures::small(),
        Base::Stock => config::stock(),
    };
    settings.limits = match profile {
        Base::Fixture => temper_limits_fixtures::small(),
        Base::Stock => limits::stock(),
    };
    settings
}

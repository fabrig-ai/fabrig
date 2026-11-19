//! Notify the service of settings decisions after durability
//! (temper/docs/design/protocol/configuration/limit-settings.md, 4).
use alloc::boxed::Box;
use jig_domain_core::{self as core, settings as s};
use skein_lib::Env;
use temper_engine_domain_forge::settings;

use crate::{Decision, Delivery, Domain, Limits};

pub(super) fn saved(domain: &Domain, env: &Env<Limits>, decision: &mut Decision, record: &core::Record) {
    let core::Record::Core(core::CoreRecord::Settings(s::Record::Revision { history: false, revision })) = record
    else {
        return;
    };
    if !domain.ready() || revision.head.number != domain.core.current.newest() {
        return;
    }
    let mut named = false;
    for item in &revision.items {
        named |= item.setting == s::Setting::Limits;
    }
    for item in &revision.applications {
        named |= item.kind == 22;
    }
    if !named {
        return;
    }
    let jig: Box<[s::Override]> = match domain.core.current.value(s::Setting::Limits) {
        Some(value) => Box::from(value.as_limits().expect("limits setting has its own value")),
        None => Box::new([]),
    };
    let temper = if let Some(settings::SizedValue { value: settings::Value::Limits(values), .. }) =
        domain.forge.setting(22, 0, 1)
    {
        values.clone()
    } else {
        Box::new([])
    };
    super::emit(
        decision,
        &env.limits,
        Delivery::LimitSettingsCommitted { revision: revision.head.number, jig, temper },
    );
}

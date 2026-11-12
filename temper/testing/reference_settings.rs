//! Reference settings as a run reads them (temper/docs/design/protocol/configuration/reference.md).

pub fn items() -> Box<[jig_domain_core::settings::Item]> {
    use jig_client::contract as c;
    let limits = temper_client_tools::limits::tools();
    let items = temper_client_tools::jig::settings_file::decode(
        include_bytes!("../docs/examples/reference/settings.json"),
        &limits,
    )
    .expect("reference settings");
    let mut charters = skein_lib::List::with_capacity(4);
    for item in items.into_boxed() {
        if let c::Setting::Charter(_) = item.setting {
            charters.push(item).expect("reference charters");
        }
    }
    let ask = c::Ask::ChangeSettings(c::ChangeSettings { against: 1, items: charters, running: c::Running::Stays });
    let jig_client_protocol_wire::Ask::ChangeSettings(change) =
        jig_client_protocol_link::translate::asks::ask(&ask, &limits.settings).expect("reference settings wire")
    else {
        panic!("settings change")
    };
    jig_protocol_engine_clients::translate::settings::items(change.items().into(), &[], &limits.settings)
        .expect("reference core settings")
}

pub fn charter(defaults: &jig_domain_core::settings::Defaults, number: u32) -> jig_domain_core::Charter {
    use jig_domain_core::settings::{Edit, Setting, Value, overlaid};
    let items = items();
    let item = items.iter().find(|item| item.setting == Setting::Charter(number)).expect("reference charter item");
    let Edit::Set(Value::Overlay(overlay)) = &item.edit else { panic!("reference overlay") };
    let stock = defaults.charters.iter().find(|charter| charter.number == number).expect("stock charter");
    overlaid(stock, overlay)
}

//! Application reference checks are private to jig's settings transaction; exercise
//! them through the root (temper/docs/design/testing/settings-tests.md, 3, 6 and 7).
use jig_domain_authority::Executor;
use jig_domain_core::settings as s;
use jig_domain_people as people;
use temper_engine_domain::{Delivery, Record};
use temper_engine_domain_forge::{self as forge, settings};
use temper_engine_domain_world::{direct::Driver, kinds, store::Store};

fn clear(authority: &mut jig_domain_authority::Authority, charter: u32) {
    authority.delegation.kinds =
        authority.delegation.kinds.iter().copied().filter(|kind| *kind != Executor::Charter(charter)).collect();
}

#[test]
fn a_retired_producer_is_refused_while_named() {
    let limits = super::stock_limits();
    let mut config = super::stock_config();
    config.core.kinds = temper_engine_service_stock::stock::kinds(&limits, 1);
    config.kinds_stock = temper_engine_service_stock::stock::forge(&limits);
    let producer = config
        .kinds_stock
        .iter()
        .find_map(|row| match row.value.value {
            settings::Value::Charter(charter) if row.kind == 3 && row.number == 1 => Some(charter),
            _ => None,
        })
        .expect("stock kind 3 number 1: the change call's default producer");
    assert_eq!(producer, 2);
    // Leave the stock template and its work intact. Only competing authority
    // references are cleared, found by walking every stock policy and role.
    clear(&mut config.core.defaults.rules.ceiling, producer);
    let policies: Vec<_> = config.core.defaults.policies.iter().map(|(number, _)| *number).collect();
    for number in policies {
        let policy = config.core.defaults.policies.get_mut(&number).expect("stock policy");
        clear(&mut policy.policy.ceiling, producer);
        for role in &mut policy.policy.roles {
            clear(&mut role.authority, producer);
        }
    }
    let template = config.core.defaults.templates.iter().find(|t| t.number == 4).expect("stock draft goal");
    assert_eq!(template.executor, people::TemplateExecutor::Charter(5));
    let mut driver = Driver::cold(Store::with_limits(&limits), config, &limits);
    driver.settle();
    driver.sign_in();
    driver.settle();
    assert!(
        driver.store.rows.values().any(|record| matches!(record,
            Record::Forge { row } if matches!(row.as_ref(), forge::Stored::Setting(settings::SettingRow {
                key: settings::SettingKey { kind: 3, number: 1, slot: settings::SettingSlot::Default, .. },
                edit: settings::Edit::Set(settings::SizedValue { value: settings::Value::Charter(number), .. }),
            }) if *number == producer)
        )),
        "the forge keeps the stock producer's reference"
    );
    let against = kinds::revision(&driver.store.rows);
    eprintln!("P59: template 4 change work -> kind 3 number 1 -> remove producer charter {producer}");
    driver.send(kinds::change(
        driver.session(),
        against,
        31,
        Box::new([s::Item { setting: s::Setting::Charter(producer), edit: s::Edit::Removed }]),
    ));
    driver.settle();
    assert!(
        driver.delivered.iter().any(|delivery| matches!(
            delivery,
            Delivery::WebReply {
                reply: people::Reply::Outcome(people::Outcome::Refused(people::Refusal::NotWellFormed {
                    why: people::Malformed::Settings {
                        setting: people::Setting::Application { kind: 3, scope: 0, number: 1 },
                        fault: people::SettingFault::Reference,
                        ..
                    },
                    ..
                })),
                ..
            }
        )),
        "P59: retiring producer charter {producer}, named by template 4's change work through kind 3 number 1, must be refused: {:?}",
        driver.delivered
    );
    assert_eq!(kinds::revision(&driver.store.rows), against, "a refusal commits nothing");
}

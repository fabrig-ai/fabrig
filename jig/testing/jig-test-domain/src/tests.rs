pub(crate) mod fixtures;
mod link;
mod public;
mod start;

mod catalogue;

#[test]
fn declarations_use_each_named_connectors_kind_rows_and_world_rules() {
    use jig_domain_contract::{
        effects::{EffectForm, EffectKind, Recovery},
        resources::{ResourceKind, Rule, Taken},
    };
    use jig_test_connector::{Form, KindSpec, Recovery as ConnectorRecovery};
    let mut config = fixtures::configuration();
    let write = ResourceKind { kind: 6, hold: Rule::Exclusive { taken: Taken::Refuses } };
    let pool = ResourceKind { kind: 7, hold: Rule::Pooled { taken: Taken::Waits } };
    config.resource_kinds = [Box::new([write]), Box::new([pool])];
    config.first.kinds = Box::new([KindSpec {
        kind: 6,
        form: Form::Creation,
        recovery: ConnectorRecovery::Conditional,
        waits_for_dependencies: true,
        price: Some(2),
    }]);
    config.second.kinds = Box::new([KindSpec {
        kind: 9,
        form: Form::Set,
        recovery: ConnectorRecovery::Idempotent,
        waits_for_dependencies: false,
        price: None,
    }]);
    let first = EffectKind {
        kind: 6,
        form: EffectForm::Creation,
        recovery: Recovery::Conditional,
        waits_for_dependencies: true,
    };
    let second =
        EffectKind { kind: 9, form: EffectForm::Set, recovery: Recovery::Idempotent, waits_for_dependencies: false };
    let declarations = config.declarations();
    let [one, two] = declarations.as_ref() else { panic!("both configured connectors") };
    assert_eq!(
        (one.connector, one.resources.as_ref(), one.effects.as_ref()),
        (1, [write].as_slice(), [first].as_slice())
    );
    assert_eq!(
        (two.connector, two.resources.as_ref(), two.effects.as_ref()),
        (2, [pool].as_slice(), [second].as_slice())
    );
    config.core.connectors = Box::new([2]);
    assert_eq!(config.declarations().as_ref(), ::core::slice::from_ref(two));
}

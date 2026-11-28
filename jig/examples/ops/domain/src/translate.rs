//! Application vocabulary in small total functions. Copy the core/host pairs;
//! replace resource paths, effect kinds and tool declarations (domain/root.md, 6).
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_contract::{
    Connector as ContractConnector, Declaration,
    effects::{self, EffectKind},
    resources::{ResourceKind, Rule, Taken},
};
use jig_domain_core as core;
use jig_ops_domain_connectors::{Connector, number};
use jig_ops_domain_infrastructure as infrastructure;
use jig_ops_domain_observability as observability;
use skein_lib::List;

/// Read kinds and resources belong to the application's connector vocabulary.
pub(crate) fn read_description(
    number: u16,
    read: &observability::Read,
    vocabulary: &observability::Vocabulary,
) -> authority::Effect {
    let (kind, resource) = match read {
        observability::Read::Logs { service, .. } => {
            (vocabulary.logs_read, observability::Resource::Logs(service.clone()))
        }
        observability::Read::Series { service, .. } => {
            (vocabulary.series_read, observability::Resource::Metrics(service.clone()))
        }
        observability::Read::Fired { service, .. } => {
            (vocabulary.fired_read, observability::Resource::Service(service.clone()))
        }
    };
    authority::Effect {
        connector: number,
        kind,
        name: authority::Name { segments: resource.segments() },
        state: [0; 32],
        price: None,
        access: authority::EffectAccess::Context,
        additional: Box::new([]),
        guards: Box::new([]),
    }
}

/// The connector owns adoption roles; the root carries them into core admission.
pub(crate) const fn resource_role(role: infrastructure::ResourceRole) -> core::effects::connector::ResourceRole {
    match role {
        infrastructure::ResourceRole::Owned => core::effects::connector::ResourceRole::Owned,
        infrastructure::ResourceRole::Participant => core::effects::connector::ResourceRole::Participant,
        infrastructure::ResourceRole::Context => core::effects::connector::ResourceRole::Context,
        infrastructure::ResourceRole::Unavailable => core::effects::connector::ResourceRole::Unavailable,
    }
}

pub(crate) fn declarations(backend: infrastructure::Backend, limits: &crate::Limits) -> [Declaration; 2] {
    let infrastructure = &limits.infrastructure.vocabulary;
    [
        Declaration {
            connector: ContractConnector(number(Connector::Observability)),
            resources: Box::new([]),
            effects: Box::new([EffectKind {
                kind: limits.observability.vocabulary.silence_effect,
                form: effects::EffectForm::Set,
                recovery: effects::Recovery::Keyed,
                waits_for_dependencies: false,
            }]),
            projects_goals: false,
            sections: false,
        },
        Declaration {
            connector: ContractConnector(number(Connector::Infrastructure)),
            resources: hold_kinds(infrastructure),
            effects: effect_kinds(infrastructure, backend),
            projects_goals: false,
            sections: false,
        },
    ]
}

fn effect_kinds(vocabulary: &infrastructure::Vocabulary, backend: infrastructure::Backend) -> Box<[EffectKind]> {
    let values = infrastructure::effect_kinds(vocabulary, backend);
    let mut kinds = List::with_capacity(u32::try_from(values.len()).expect("effect kinds"));
    for value in values {
        let form = match value.form {
            infrastructure::Form::Creation => effects::EffectForm::Creation,
            infrastructure::Form::Transition => effects::EffectForm::Transition,
            infrastructure::Form::Set => effects::EffectForm::Set,
        };
        let recovery = match value.recovery {
            infrastructure::Recovery::Keyed => effects::Recovery::Keyed,
            infrastructure::Recovery::Conditional => effects::Recovery::Conditional,
            infrastructure::Recovery::Unrecoverable => effects::Recovery::Unrecoverable,
        };
        kinds
            .push(EffectKind { kind: value.kind, form, recovery, waits_for_dependencies: value.waits_for_dependencies })
            .expect("one kind per value");
    }
    kinds.into_boxed()
}

fn hold_kinds(vocabulary: &infrastructure::Vocabulary) -> Box<[ResourceKind]> {
    let values = infrastructure::hold_kinds(vocabulary);
    let mut kinds = List::with_capacity(u32::try_from(values.len()).expect("hold kinds"));
    for value in values {
        let hold = match value.hold {
            infrastructure::Hold::None => Rule::Shared,
            infrastructure::Hold::ExclusiveWait => Rule::Exclusive { taken: Taken::Waits },
            infrastructure::Hold::PooledWait => Rule::Pooled { taken: Taken::Waits },
        };
        kinds.push(ResourceKind { kind: value.kind, hold }).expect("one kind per value");
    }
    kinds.into_boxed()
}

/// Decode configured envelopes before giving the typed templates to their owner.
pub(crate) fn observability(
    triage: Box<[observability::TriageTemplate]>,
    lead: Box<[observability::LeadTemplate]>,
    limits: &crate::Limits,
) -> observability::Domain {
    let policy = limits.policy;
    let mut domain = observability::Domain::with_templates(&limits.observability, triage.clone())
        .with_leads(&limits.observability, lead.clone())
        .with_contract(
            ContractConnector(number(Connector::Observability)),
            observability::Requirements {
                healthy_replica_requirement: policy.healthy_replica_requirement,
                low_load_requirement: policy.low_load_requirement,
                scale_down_load_percent: policy.scale_down_load_percent,
                scale_down_load_seconds: policy.scale_down_load_seconds,
                freshness_seconds: policy.verdict_freshness_seconds,
            },
        );
    for template in triage {
        if let Some(batch) = jig_ops_protocol_observability::lead(&template.delegate, &policy.delegate) {
            domain.prepare_triage(template.number, batch);
        }
    }
    for template in lead {
        if let Some(batch) = jig_ops_protocol_observability::lead(&template.delegate, &policy.delegate) {
            domain.prepare_lead(template.number, batch);
        }
    }
    domain
}

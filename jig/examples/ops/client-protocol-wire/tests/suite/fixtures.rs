use jig_ops_client_protocol_wire::v1 as w;
use skein_lib::List;

pub const PROJECT: u32 = 7;
pub fn environment() -> w::InfrastructureEnvironment {
    w::InfrastructureEnvironment { pool: b"Q.pool".as_slice().into(), name: b"preview".as_slice().into() }
}
pub const REVISION: u64 = 3;
pub const UNTIL: u64 = 81;
const INCIDENT: u64 = 13;
const WATCH: u64 = 19;
const RAISED: u64 = 41;
const LEAD: u64 = 23;

pub fn request() -> w::InfrastructureAskRecord {
    let value = w::InfrastructureAskMoveEnd {
        project: PROJECT,
        object: 17,
        environment: environment(),
        revision: REVISION,
        until: UNTIL,
    };
    w::InfrastructureAskRecord { value: w::InfrastructureAsk::MoveEnd(value) }
}

pub fn infrastructure_subject() -> w::InfrastructureSubject {
    w::InfrastructureSubject { scope: w::InfrastructureScope::Deployment, kind: w::InfrastructureKind::Deployment }
}

pub fn incident() -> w::ObservabilityIncident {
    w::ObservabilityIncident {
        project: PROJECT,
        number: INCIDENT,
        watch: WATCH,
        raised: RAISED,
        service: w::ObservabilityService { environment: b"staging".as_slice().into(), name: b"api".as_slice().into() },
        severity: 2,
        state: w::ObservabilityState::Alerting,
        timeline: List::with_capacity(0),
        cut: true,
        lead: Some(LEAD),
        alerts: List::with_capacity(0),
    }
}

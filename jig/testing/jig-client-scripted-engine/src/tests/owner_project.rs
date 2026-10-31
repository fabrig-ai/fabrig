use jig_client_domain::contract::*;
use skein_lib::List;
pub(super) fn source() -> OwnerProject {
    OwnerProject {
        number: 1,
        name: DocumentText::complete(super::text(b"Kit")),
        acts: List::with_capacity(2),
        resources: List::with_capacity(2),
        policy: OwnerProjectPolicy {
            templates: skein_lib::List::with_capacity(0),
            ceiling: OwnerAuthorityDocument::complete(OwnerAuthority {
                models: None,
                tools: u64::MAX,
                grants: List::with_capacity(0),
                delegation: OwnerDelegation { kinds: List::with_capacity(0), tasks: u32::MAX, depth: u32::MAX },
                spend: u64::MAX,
                deadline: None,
                notes: u8::MAX,
                note_resources: List::with_capacity(0),
            }),
            period_spend: u64::MAX,
            escalation_role: None,
            projections: List::with_capacity(2),
            roles: List::with_capacity(2),
            requirements: List::with_capacity(2),
            permissions: None,
            permissions_cut: false,
            cut: true,
        },
        members: List::with_capacity(2),
        current_period: None,
        pools: List::with_capacity(2),
        configured_period: 0,
        configured_pools: List::with_capacity(2),
        configured_pools_cut: false,
        spend: List::with_capacity(2),
        recurring: List::with_capacity(2),
        cut: true,
    }
}

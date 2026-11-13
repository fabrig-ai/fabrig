use super::{self as f, connect_forge as forge};
use temper_client_domain::{self as d, Address, Step, pages};
use temper_client_protocol_wire::v1 as w;

#[must_use]
pub fn model_address(kind: d::WayKind) -> Address {
    Address::Welcome { step: Step::Model { way: Some(d::ModelWay { provider: 17, kind, source: None }) } }
}

#[must_use]
pub fn owned_adopted_rows() -> skein_lib::List<w::ForgeRow> {
    let mut rows = forge::first_rows(forge::connected(), forge::made());
    let mut parts = rows.get(0).unwrap().clone().into_parts();
    parts.repositories = skein_lib::List::with_capacity(1);
    parts
        .repositories
        .push(
            w::Adoption::new(
                &w::CEILINGS,
                w::AdoptionParts {
                    owner: b"work".as_slice().into(),
                    name: b"hello".as_slice().into(),
                    project: f::PROJECT,
                },
            )
            .unwrap(),
        )
        .unwrap();
    *rows.get_mut(0).unwrap() = w::ForgeRow::new(&w::CEILINGS, parts).unwrap();
    rows
}

#[must_use]
pub fn model_with_forge(kind: d::WayKind) -> (d::Domain, skein_lib::Token) {
    let (offered, _) = super::connect_model::flow_fixture(kind);
    let me = pages::accounts::current_me(offered.client()).unwrap().clone();
    let (mut root, me_watch, _, _, accounts, _) = f::root_at_with_forges(
        model_address(kind),
        &f::limits(),
        b"Shop",
        None,
        None,
        Some(forge::first_rows(forge::connected(), forge::made())),
    );
    f::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: me_watch,
                    event: jig_client::contract::WatchEvent::Delivered(jig_client::contract::Delivery::Me(Box::new(
                        me,
                    ))),
                },
            },
        },
    );
    (root, accounts.unwrap())
}

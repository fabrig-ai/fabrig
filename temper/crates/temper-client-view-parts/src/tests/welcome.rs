use super::fixtures as f;
pub(super) use super::fixtures::welcome::owned_adopted_rows;
use temper_client_domain::{self as d, Address, ChatsFilter, Step, pages};
use temper_client_protocol_wire::v1 as w;

pub(crate) fn ready_root(rows: skein_lib::List<w::ForgeRow>) -> d::Domain {
    let (mut root, account) = f::forge_root_accounts(Address::Welcome { step: Step::Ready }, rows);
    f::welcome_account(&mut root, account);
    root
}

pub(crate) fn first_run_destination(root: &mut d::Domain) -> Address {
    let mut out = skein_lib::Queue::uncapped();
    d::step(
        root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits: f::limits() },
        d::Event::Went { place: d::Place::Address(Address::Welcome { step: Step::Welcome }) },
        &mut out,
    );
    for request in &out {
        if let d::Request::Show { address, replace: true } = request {
            return address.clone();
        }
    }
    panic!("first-run destination");
}

#[test]
fn a_whole_store_moves_on() {
    let mut root = ready_root(owned_adopted_rows());
    assert_eq!(pages::welcome::decision(&root), Some(pages::welcome::FirstRun::Whole));
    assert_eq!(first_run_destination(&mut root), Address::Chats { filter: ChatsFilter::All });
}

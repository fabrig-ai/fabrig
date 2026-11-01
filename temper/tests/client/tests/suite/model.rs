//! Shared setup over existing fixtures and the standard client harness.
use jig_client_world::harness::{ClientId, Until, World};
use skein_fake_person::{Find, Step};
use skein_ui::Role;
use skein_world::domain::Verdict;
use temper_client_domain::ChatsFilter;
use temper_client_domain::{Address, Place, address};
use temper_client_world::{app::Temper, fixtures as f, front::Joint};

pub type Pages = World<Temper, Joint>;

pub fn at(path: &[u8]) -> Address {
    let Place::Address(address) = address::parse(path) else { panic!("page address") };
    address
}
pub fn world() -> Pages {
    f::model_world(f::SEED)
}
pub fn see(role: Role, name: &[u8]) -> Step<Address> {
    Step::See { find: Find::exactly_named(role, name), within: f::WITHIN }
}
pub fn press(role: Role, name: &[u8]) -> Step<Address> {
    Step::Press { find: Find::exactly_named(role, name) }
}
pub fn follow(world: &mut Pages, client: ClientId, steps: Vec<Step<Address>>) {
    world.person(client, steps);
    let result = world.run(Until::Done);
    assert_eq!(result.verdict, Verdict::Passed, "{result:?}");
}
pub fn open(world: &mut Pages, path: &[u8]) -> ClientId {
    let client = world.client(f::PARTY, Address::Chats { filter: ChatsFilter::All, start: false });
    follow(world, client, vec![see(Role::Status, b"Live"), Step::Go { address: at(path) }]);
    client
}

pub fn type_key() -> Step<Address> {
    Step::Type {
        find: skein_fake_person::Find::named(Role::TextField, b""),
        // Existing account_acts::kept acknowledges this fake value.
        words: b"test-private-key".as_slice().into(),
    }
}

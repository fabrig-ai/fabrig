//! Native pages received by the fake person, including the forms (temper/docs/design/client/README.md, 3.5).
use skein_fake_person::{Find, Step};
use skein_lib::Duration;
use temper_client_domain::{
    Address,
    address::{ChatsFilter, Choice, Filter, Step as Welcome},
};
const SETTLE: Duration = Duration::from_millis(100);
pub(super) fn steps() -> Box<[Step<Address>]> {
    let mut steps = Vec::new();
    for address in [
        Address::Forges,
        Address::Accounts,
        Address::Inbox { filter: Filter::ALL },
        Address::Chats { filter: ChatsFilter::All, start: false },
        Address::Adopt { project: 1 },
        Address::ConnectForge { forge: Some(Choice::Forge(1)) },
        Address::ConnectModel { way: None },
        Address::Welcome { step: Welcome::Welcome },
    ] {
        steps.push(Step::Go { address });
        steps.push(Step::Pause { span: SETTLE });
    }
    steps.push(Step::Go { address: Address::Forges });
    steps.push(Step::See { find: Find::text(b"temper-person"), within: Duration::from_secs(5) });
    steps.into_boxed_slice()
}

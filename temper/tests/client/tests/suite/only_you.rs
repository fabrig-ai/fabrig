//! Only you through the client and engine (temper/docs/design/client/README.md, 3.5).
use jig_client::contract as c;
use jig_client_kit::harness::App;
use jig_domain_accounts as a;
use skein_lib::Queue;
use temper_client_domain::{
    Address,
    address::Filter,
    pages::{
        only_you::{self, Item, Why},
        parts,
    },
};
use temper_client_world::{
    app::{Root, Temper},
    fixtures,
    front::{Joint, account_acts as accounts},
};

fn account_event(front: &mut Joint, root: &mut Root, event: a::Event) {
    front.engine.send(temper_engine_domain::Event::Account(event));
    accounts::settle(front, root);
}

fn inbox(front: &mut Joint, root: &mut Root) {
    accounts::event(front, root, Temper::went(Address::Inbox { filter: Filter::ALL }));
}

fn hidden(root: &Root) {
    assert!(only_you::read(root.person()).items.is_empty());
    let facts = parts::needs_facts(root.person(), fixtures::NOW);
    assert!(facts.known, "the inbox has answered");
    assert_eq!(facts.only, 0);
    assert!(!parts::needs(&facts).only);
}

fn listed(root: &Root, account: u32, source: u32, why: Why) {
    let read = only_you::read(root.person());
    assert_eq!(read.items.as_slice(), &[Item { account, source, kind: c::AccountKind::SignIn, why }]);
    let facts = parts::needs_facts(root.person(), fixtures::NOW);
    assert!(facts.known, "the inbox has answered");
    assert_eq!(facts.only, 1);
    assert!(parts::needs(&facts).only);
}

#[test]
fn a_login_is_listed_only_while_it_needs_its_person() {
    let (mut front, mut root, source) = accounts::signed_in_plan(fixtures::SEED);
    let document = root.person().pages().accounts_document(root.person().client()).expect("accounts delivered");
    let account = document
        .accounts
        .iter()
        .find(|account| account.sources.iter().any(|s| s.number == source))
        .expect("signed-in plan")
        .number;
    let generation = front
        .engine
        .protocol
        .accounts
        .iter()
        .rev()
        .find_map(|request| {
            if let a::Request::Granted { grant } = request {
                (grant.account == source).then_some(grant.generation)
            } else {
                None
            }
        })
        .unwrap_or_else(|| {
            let generation = 1;
            account_event(
                &mut front,
                &mut root,
                a::Event::Component(a::FromComponent::Granted {
                    account: source,
                    generation,
                    valid: skein_lib::Duration::from_secs(3600),
                    form: jig_domain_link::Form::Credential(jig_domain_link::Scheme::Bearer),
                }),
            );
            generation
        });
    inbox(&mut front, &mut root);
    hidden(&root);

    account_event(
        &mut front,
        &mut root,
        a::Event::Component(a::FromComponent::Expiring { account: source, generation }),
    );
    listed(&root, account, source, Why::Expiring);

    account_event(
        &mut front,
        &mut root,
        a::Event::Component(a::FromComponent::Failed { account: source, failure: a::Failure::SignInAgain }),
    );
    listed(&root, account, source, Why::SignedOut);

    accounts::event(&mut front, &mut root, Temper::went(Address::Accounts));
    let (_, mut view) = Temper::new(fixtures::SEED);
    Temper::build(&mut view, &root, &mut Queue::uncapped());
    let node = Temper::tree(&view)
        .nodes()
        .iter()
        .find(|node| node.role == skein_ui::Role::Button && node.name.as_deref() == Some(b"Continue with ChatGPT"))
        .expect("row offers sign-in again")
        .id;
    let press = Temper::act(&mut view, node, skein_ui::Act::Press).expect("row sign-in act");
    accounts::event(&mut front, &mut root, press);
    let renewal = accounts::signing(&front);
    account_event(
        &mut front,
        &mut root,
        a::Event::Visited { source: renewal, address: b"https://issuer/visit".as_slice().into() },
    );
    account_event(
        &mut front,
        &mut root,
        a::Event::SignedIn {
            source: renewal,
            subject: b"subject".as_slice().into(),
            display: b"alice@example.com".as_slice().into(),
            declined: false,
            said: a::Said::default(),
        },
    );
    inbox(&mut front, &mut root);
    let document = root.person().pages().accounts_document(root.person().client()).expect("accounts delivered");
    assert_eq!(
        document.last_end.as_ref().expect("renewal ended").end,
        c::AccountEnd::Renewed { account, declined: false }
    );
    hidden(&root);
}

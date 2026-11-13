//! Only you on Needs you (temper/docs/design/client/pages/README.md, Only you).
use crate::{
    accounts::words::{self, Word as W},
    bindings::Bindings,
    look,
    words::{Word, Words},
};
use jig_client::contract as c;
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::{
    Address, Domain,
    pages::{Act, only_you},
};

pub(crate) fn main(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings) {
    let read = only_you::read(root);
    let Some(document) = root.pages().accounts_document(root.client()) else { return };
    let heading = w
        .say(if read.cut { Word::OnlyCountCut } else { Word::OnlyCount }, &[Hole::Number(u64::from(read.items.len()))]);
    b.open(Role::Region);
    b.style(look::ONLY);
    b.name(&heading);
    b.open(Role::Slot);
    b.style(look::DECIDED_HEAD);
    crate::cards::text(b, Role::Heading { level: 2 }, &heading);
    crate::cards::text(b, Role::Text, &w.say(Word::OnlyAround, &[]));
    b.close();
    for item in &read.items {
        let Some((row, source)) = source(document, item) else { continue };
        let provider = match &row.provider_name {
            Some(name) if !name.cut => name.bytes(),
            Some(_) | None => b"",
        };
        let label = match item.kind {
            c::AccountKind::SignIn => super::accounts::named(w, W::PlanSignIn, words::provider_words(provider).1),
            c::AccountKind::Program => super::accounts::named(w, W::CodeLogin, source.name.as_deref().unwrap_or(b"")),
            c::AccountKind::Key | c::AccountKind::None => continue,
        };
        b.open(Role::Slot);
        b.style(look::ONLY_ITEM);
        b.open(Role::Paragraph);
        b.style(look::ASK_META);
        super::ask::text(b, Role::Text, &w.say(Word::RailDeployment, &[]), look::CHIP);
        crate::cards::text(b, Role::Text, &w.say(Word::OnlyAccount, &[]));
        b.close();
        let question = w.say(
            match item.why {
                only_you::Why::SignedOut => Word::OnlySignedOut,
                only_you::Why::Expiring => Word::OnlyExpiring,
            },
            &[Hole::Name(label)],
        );
        super::ask::text(b, Role::Paragraph, &question, look::ONLY_Q);
        b.open(Role::Slot);
        b.style(look::ASK_CHOICES);
        let act = w.say(Word::OnlySignInAgain, &[]);
        crate::bindings::link(b, &act, bindings.insert(Act::Go { address: Address::Accounts }));
        b.style(look::RECOMMENDED);
        b.text(Run::Plain, &act);
        b.close();
        b.close();
        b.close();
    }
    b.close();
}

fn source<'a>(document: &'a c::Accounts, item: &only_you::Item) -> Option<(&'a c::AccountLine, &'a c::AccountSource)> {
    for row in &document.accounts {
        if row.number != item.account {
            continue;
        }
        for source in &row.sources {
            if source.number == item.source {
                return Some((row, source));
            }
        }
    }
    None
}

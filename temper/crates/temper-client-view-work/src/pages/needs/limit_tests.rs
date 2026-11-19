#![expect(clippy::disallowed_methods, reason = "ordinary tests inspect the held card and its bound act")]
use super::*;
use crate::tests::fixtures as f;
use jig_client::{Answered, Outcome, contract as c};
use temper_client_protocol_wire::v1 as w;

#[test]
fn a_limit_card_offers_only_the_owner_a_raise_and_keeps_refusal_words() {
    let held =
        w::LimitHold { task: 42, limit: b"forge.entries".as_slice().into(), configured: 8, needed: 9, revision: 3 };
    let refused = Outcome::Answered(Answered::Jig(c::Answer::Refused {
        at: None,
        refusal: c::Refusal::Judged {
            at: 0,
            setting: c::Setting::Application { kind: 22, scope: 0, number: 1 },
            why: c::Text::new(b"forge.entries (setting temper-limits): refused by the start".as_slice().into(), 256)
                .unwrap(),
        },
    }));
    for owner in [true, false] {
        let mut builder = Builder::new(&f::tree_limits());
        let mut bindings = Bindings::new(16);
        limit_details(&mut builder, &f::words(), &mut bindings, &held, owner, true, Some(&refused));
        let tree = builder.finish();
        let raw = f::text(&tree);
        let text = core::str::from_utf8(&raw).unwrap();
        assert!(text.contains("forge.entries is 8; this needed 9."));
        assert!(text.contains("forge.entries (setting temper-limits): refused by the start"), "{text}");
        assert_eq!(text.contains("A deployment owner can raise it. You can ask temper."), !owner);
        let button = tree
            .nodes()
            .iter()
            .find(|node| node.role == Role::Button && node.name.as_deref() == Some(b"Raise forge.entries to 16"));
        assert_eq!(button.is_some(), owner);
        if let Some(button) = button {
            let action = bindings.decode(button.binding.unwrap().token, skein_ui::Act::Press).unwrap();
            assert_eq!(
                action,
                temper_client_domain::Action::Pages(Act::LimitSettings(
                    temper_client_domain::pages::limit_settings::Act::Raise { task: 42, revision: 3 }
                ))
            );
        }
    }
}

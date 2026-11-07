use super::fixtures;
use crate::forge::{bindings, page};
use skein_ui::Role;
use temper_client_domain::forge;
fn form() -> forge::Form {
    let mut form = forge::form(1);
    form.confirmed = true;
    form.texts[forge::Text::Owner.index()].text = Box::from(*b"acme");
    form.texts[forge::Text::Name.index()].text = Box::from(*b"shop");
    form
}
fn tree(form: &forge::Form) -> skein_ui::Tree {
    let words = fixtures::words();
    page_tree(form, &words)
}
fn page_tree(form: &forge::Form, words: &crate::words::Words) -> skein_ui::Tree {
    let mut limits = fixtures::tree_limits();
    limits.text_per_node = 2048;
    limits.text_bytes = 262_144;
    let mut builder = skein_ui::Builder::new(&limits);
    if let Some(answer) = &form.outcome {
        page::outcome(
            &mut builder,
            (b"acme", b"shop", b"temper/", b"1"),
            answer,
            words,
            fixtures::tree_limits().text_per_node,
        );
    }
    builder.finish()
}
fn adopted() -> forge::Answer {
    forge::Answer::Adopted(forge::Adopted {
        forge: 1,
        repository: 7,
        owner: Box::from(*b"acme"),
        name: Box::from(*b"shop"),
        role: forge::Role::Owned,
        home: true,
        kinds: forge::Kinds {
            read: true,
            push: true,
            open: true,
            land: true,
            review: true,
            status: true,
            comment: true,
            issue: true,
            branch: true,
        },
        protection: forge::Protection::Unknown,
    })
}
#[test]
fn each_outcome_reads_as_the_designs_sentence() {
    let words = fixtures::words();
    let rows: [(forge::Answer, &[u8], bool); 13] = [
        (adopted(), b"Adopted acme/shop on forge 1 as owned.It is the project's home.temper may: read; push; open pull requests; land; review; set statuses; comment; create issues; create and delete branches.temper's forge user may not read the branch protection: the forge shows it to repository administrators only.", false),
        (forge::Answer::ForgeRefused(forge::Reason::NoSuchForge), b"No forge 1 is configured.", true),
        (forge::Answer::ForgeRefused(forge::Reason::NotFound), b"No repository acme/shop on forge 1 that temper's forge user can see.", true),
        (forge::Answer::ForgeRefused(forge::Reason::NoAccess), b"temper's forge user may not read acme/shop on forge 1.", true),
        (forge::Answer::ForgeRefused(forge::Reason::Refused), b"The forge connector refused it: another project writes acme/shop, the project has a home already, or branches under temper/ are not this deployment's.", true),
        (forge::Answer::ForgeRefused(forge::Reason::AnotherProject), b"Another project writes acme/shop. Adopt it as context to read it, or use the project that writes it.", true),
        (forge::Answer::ForgeRefused(forge::Reason::HomeHeld), b"The project has a home already. Adopt this repository without making it home, or choose a new project.", true),
        (forge::Answer::ForgeRefused(forge::Reason::PrefixOccupied), b"Branches under temper/ are not this deployment's. Choose an unused branch prefix.", true),
        (forge::Answer::ForgeRefused(forge::Reason::InFlight), b"acme/shop is being adopted already; try again when that ends.", true),
        (forge::Answer::ForgeRefused(forge::Reason::TooLarge), b"A field is past the forge's bounds, or the forge's repositories are full.", true),
        (forge::Answer::ForgeRefused(forge::Reason::Unreachable), b"Forge 1 did not answer; nothing was adopted. Try again.", true),
        (forge::Answer::ForgeRefused(forge::Reason::InvalidAnswer), b"Forge 1 answered what temper did not expect; nothing was adopted.", true),
        (forge::Answer::Unknown, b"It is not known whether acme/shop was adopted.", false),
    ];
    for (answer, said, alert) in rows {
        let mut form = form();
        form.outcome = Some(answer.clone());
        let rendered = page_tree(&form, &words);
        let text = fixtures::text(&rendered);
        assert_eq!(text.as_ref(), said, "outcome {answer:?}");
        assert_eq!(has_role(&rendered, Role::Alert), alert, "refusal alert {answer:?}");
    }
}
#[test]
fn each_kind_of_jig_refusal_uses_jigs_words() {
    let refusal = jig_client::contract::Refusal::Role { held: None };
    for answer in [
        forge::Answer::CoreRefused(refusal.clone()),
        forge::Answer::EntranceRefused(
            jig_client::contract::EntranceRefusal::new(refusal.clone()).expect("entrance refusal"),
        ),
    ] {
        let mut form = form();
        form.outcome = Some(answer.clone());
        let rendered = tree(&form);
        assert_eq!(
            fixtures::text(&rendered).as_ref(),
            b"The held role does not permit this actionNo held role was supplied",
            "outcome {answer:?}"
        );
        assert!(has_role(&rendered, Role::Alert), "alert {answer:?}");
    }
}
#[test]
fn typed_fields_reject_invalid_numbers_and_duplicate_checks() {
    for (text, expected) in [
        (b"1".as_slice(), Some(1)),
        (b"65535", Some(65535)),
        (b"0", None),
        (b"65536", None),
        (b"-1", None),
        (b"", None),
    ] {
        assert_eq!(bindings::forge_number(text), expected, "forge {text:?}");
    }
    let rows: [(&[u8], Option<&[u32]>); 7] = [
        (b"", Some(&[])),
        (b"1, 2 3", Some(&[1, 2, 3])),
        (b"1, 1", None),
        (b"0", None),
        (b"x", None),
        (b"4294967296", None),
        (b"1 2 3 4", None),
    ];
    for (text, expected) in rows {
        assert_eq!(bindings::checks(text, 3).as_deref(), expected, "checks {text:?}");
    }
}
fn has_role(tree: &skein_ui::Tree, role: Role) -> bool {
    for node in tree.nodes() {
        if node.role == role {
            return true;
        }
    }
    false
}

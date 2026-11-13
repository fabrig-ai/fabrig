use super::fixtures as f;



use temper_client_domain::Address;

#[test]
fn the_playground_names_default_for_a_resolved_model() {
    use jig_client::contract as c;
    for (row, project, model, requested, expected) in [
        ("default", f::PROJECT, true, false, "Scripted model · Default"),
        ("requested", f::PROJECT, true, true, "Scripted model · Default"),
        ("another project", f::SECOND_PROJECT, true, false, "Scripted model · Default"),
    ] {
        let choice = if requested {
            Some(c::LlmChoice {
                later: None,
                account: None,
                main: c::ModelEffort { model: c::Text::new(b"selected".as_slice().into(), 32).unwrap(), effort: None },
                subagents: skein_lib::List::with_capacity(0),
            })
        } else {
            None
        };
        let root = f::playground_choice_root(Address::Chat { task: 1 }, project, model, choice);
        assert_eq!(using_text(&root), expected, "{row}");
    }
    let root = f::playground_root(Address::NewChat { draft: false }, f::PROJECT, true);
    assert_eq!(using_text(&root), "Scripted model · Default");
}


#[test]
fn a_real_account_keeps_the_playgrounds_scripted_model() {
    let root = f::playground_kept_root();
    assert!(root.forge().playground().is_some());
    assert_eq!(using_text(&root), "Scripted model · Default");
}


#[test]
fn the_hidden_deployment_playground_account_needs_only_its_public_provider() {
    use jig_client::contract::Readiness;
    for (ready, payer, expected) in [
        (Some(Readiness::Usable), 73, "Scripted model · Default"),
        (Some(Readiness::Unknown), 73, "Scripted model · Default"),
        (None, 73, "Scripted model · Default"),
        (Some(Readiness::Unavailable), 73, "Scripted model · Default"),
        (Some(Readiness::Usable), 74, "Scripted model · Default"),
    ] {
        let root = f::hidden_playground_root(ready, payer);
        assert!(using_text(&root).ends_with(expected));
    }
}

#[expect(clippy::disallowed_types, clippy::disallowed_methods, reason = "the render adapter inspects model-line words with ordinary Rust")]
fn using_text(root: &temper_client_domain::Domain) -> String {
    let mut b = skein_ui::Builder::new(&f::tree_limits());
    crate::choice::using(
        &mut b,
        root,
        root.pages().chat().and_then(|chat| chat.task(root.client())),
        &f::words(),
    );
    let text = String::from_utf8(f::text(&b.finish()).into_vec()).unwrap();
    let text = text.split(" · Manage usage").next().unwrap();
    if text == "Using the default model" {
        return text.to_owned();
    }
    if text.starts_with("Using your ") && text.contains("API key") {
        text.strip_prefix("Using your ").unwrap().to_owned()
    } else {
        text.strip_prefix("Using ").unwrap_or(text).to_owned()
    }
}

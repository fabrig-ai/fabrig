use super::fixtures;

#[test]
fn each_adoption_form_state_prints_as_its_golden_file() {
    use temper_client_domain::forge;
    let words = fixtures::page_words();
    let empty = fixtures::adoption_form(false);
    let mut invalid = empty.clone();
    for field in [forge::Text::Owner, forge::Text::Name, forge::Text::Landing] {
        invalid.texts[field.index()].broken = Some(forge::Broken::Required);
    }
    let filled = fixtures::adoption_form(true);
    let mut pending = filled.clone();
    pending.pending = Some((skein_lib::Token::new(1), skein_lib::RequestProgress::Parked));
    let mut adopted = filled.clone();
    adopted.outcome = Some(forge::Answer::Adopted(fixtures::adopted(forge::Protection::Unknown)));
    let mut not_found = filled.clone();
    not_found.outcome = Some(forge::Answer::ForgeRefused(forge::Reason::NotFound));
    let mut core = filled.clone();
    core.outcome = Some(forge::Answer::CoreRefused(fixtures::role_refusal()));
    let mut out_of_date = filled;
    out_of_date.served = forge::Served::OutOfDate;
    for (name, form) in [
        ("adopt_empty", empty),
        ("adopt_invalid", invalid),
        ("adopt_pending", pending),
        ("adopt_adopted", adopted),
        ("adopt_refused_not_found", not_found),
        ("adopt_refused_core", core),
        ("adopt_out_of_date", out_of_date),
    ] {
        let tree = fixtures::adoption_page(&form, &words);
        let print = fixtures::print(&tree);
        if std::env::var_os("SKEIN_CODEGEN_WRITE").is_some() {
            std::fs::write(fixtures::golden(name), print.text.as_slice()).expect("write adoption golden file");
        }
        let golden = std::fs::read(fixtures::golden(name)).expect("adoption golden file");
        assert_eq!(print.text.as_slice(), golden, "page {name}");
    }
}

#[test]
fn each_fixture_page_prints_as_its_golden_file() {
    for (name, tree) in [
        ("inbox", fixtures::inbox_page()),
        ("chats", fixtures::chats_page()),
        ("chat", fixtures::chat::page()),
        ("chat_delegation_running", fixtures::chat::delegation_page(false)),
        ("chat_delegation_landed", fixtures::chat::delegation_page(true)),
        ("first_run", fixtures::chat::first_run()),
        ("task", fixtures::task_page()),
        ("agent_task", fixtures::agent_task_page()),
        ("signed_out", fixtures::signed_out_page()),
        ("every_role", fixtures::every_role()),
        ("control_text", fixtures::control_text()),
    ] {
        let print = fixtures::print(&tree);
        if std::env::var_os("SKEIN_CODEGEN_WRITE").is_some() {
            std::fs::write(fixtures::golden(name), print.text.as_slice()).expect("write golden file");
        }
        let golden = std::fs::read(fixtures::golden(name)).expect("golden file");
        assert_eq!(String::from_utf8_lossy(print.text.as_slice()), String::from_utf8_lossy(&golden), "page {name}");
    }
}
#[test]
fn the_first_line_is_the_title_and_the_link_state() {
    let print = fixtures::print(&fixtures::header_page(Some(b"Inbox"), false));
    let text = String::from_utf8(print.text.as_slice().to_vec()).expect("printable UTF-8");
    assert_eq!(text.lines().next(), Some("== Inbox == Live"));
    assert_eq!(text.matches("Live").count(), 1, "link appears only in header");
    assert_eq!(text.matches("Inbox").count(), 2, "the title and temper navigation each name the inbox");
}
#[test]
fn a_behind_frame_adds_its_sentence_to_the_first_line() {
    let print = fixtures::print(&fixtures::header_page(Some(b"Inbox"), true));
    let text = String::from_utf8(print.text.as_slice().to_vec()).expect("printable UTF-8");
    assert_eq!(text.lines().next(), Some("== Inbox == Live; Behind: waiting for current facts"));
    assert_eq!(text.matches("Behind: waiting for current facts").count(), 1);
    assert!(text.contains("Pat"), "identity is separate");
}
#[test]
fn the_largest_fixture_tree_prints_within_its_bounds() {
    let limits = fixtures::tree_limits();
    let (lines, bytes) = skein_ui_text::render_bound(&limits).expect("fixture print envelope");
    for (name, tree) in [("roles", fixtures::every_role()), ("task", fixtures::task_page())] {
        let print = fixtures::print(&tree);
        assert!(u64::from(print.text.len()) <= bytes, "{name}: bytes");
        assert!(skein_lib::bytes::count(print.text.as_slice(), b"\n", u32::MAX) <= lines, "{name}: lines");
        assert!(print.numbers.len() <= limits.nodes, "{name}: numbers");
        assert!(skein_ui_text::render_bound(&limits).is_some(), "{name}: memory bound");
    }
    let native = crate::limits::view(&crate::limits::native()).expect("native view bounds").0;
    assert!(skein_ui_text::render_bound(&native.tree).is_some(), "native view has a print envelope: {:?}", native.tree);
}

#[test]
fn account_fields_print_hidden() {
    let mut builder = skein_ui::Builder::new(&fixtures::tree_limits());
    builder.open(skein_ui::Role::TextField);
    builder.name(b"Key (pasted once)");
    builder.entry(skein_ui::EntryKind::Secret);
    builder.value(b"*", 1);
    builder.close();
    let print = fixtures::print(&builder.finish());
    assert!(print.text.as_slice().windows(b"(hidden)".len()).any(|part| part == b"(hidden)"));
}
#[test]
fn account_sign_in_links_print_the_target() {
    const ADDRESS: &[u8] = b"https://issuer.example/visit";
    let mut builder = skein_ui::Builder::new(&fixtures::tree_limits());
    builder.open(skein_ui::Role::Link);
    builder.text(skein_ui::Run::Plain, b"Open the sign-in page");
    builder.external(ADDRESS);
    builder.close();
    let print = fixtures::print(&builder.finish());
    let expected = b"Open the sign-in page <https://issuer.example/visit>";
    assert!(print.text.as_slice().windows(expected.len()).any(|part| part == expected));
}

#[test]
fn the_first_run_prints_forges_and_the_banners_connect_link() {
    let print = fixtures::print(&fixtures::chat::first_run());
    let text = String::from_utf8_lossy(print.text.as_slice());
    assert!(text.contains("] Forges\n"));
    assert!(text.contains("] Connect a forge\n"));
    assert!(!text.contains("Connect a forge (disabled)"));
}

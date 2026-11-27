//! Client version constants follow their headers (jig/docs/design/protocol/README.md, 7).

#[test]
fn version_constants_equal_the_family_headers() {
    let client = &skein_codegen::values(crate::CLIENT_SCHEMA).schema;
    assert_eq!(client.family.as_bytes(), b"jig-client-wire");
    assert_eq!(crate::VERSION, u32::from(client.version));
    assert_eq!(crate::VERSION, 44);
    let public = &skein_codegen::values(crate::PUBLIC_SCHEMA).schema;
    assert_eq!(public.family.as_bytes(), b"jig-public-wire");
    assert_eq!(crate::PUBLIC_VERSION, public.version);
}

#[test]
fn layout_constants_equal_the_family_fingerprints() {
    for (text, layout) in [(crate::CLIENT_SCHEMA, crate::LAYOUT), (crate::PUBLIC_SCHEMA, crate::PUBLIC_LAYOUT)] {
        let schema = skein_codec::schema::parse(text.as_bytes()).expect("valid wire schema");
        assert_eq!(layout, schema.fingerprints().family().digest());
    }
}

#[test]
fn template_display_text_round_trips() {
    use crate as w;
    use crate::Record as _;
    use skein_lib::{List, Reader};

    let limits = super::fixtures::limits();
    let template = w::Template {
        number: 33,
        shown: w::DocumentText { bytes: Box::from(b"goal for a lean design".as_slice()), cut: false },
        executor: w::TemplateExecutor::Chosen,
        contract: w::TaskContractDocument::Report(w::TaskContractDocumentReport { words: 1024 }),
        shallower: 1,
        budget: w::TemplateBudget::Given,
        deadline: w::TemplateDeadline::None,
        tracked: Some(1),
        flags: w::TaskFlags {
            conversation: false,
            listed: true,
            retention: w::Retention::Kept,
            ended_by: w::EndedBy::Result,
        },
        label: 2,
        grace: None,
        llm: None,
        resources: List::with_capacity(0),
    };
    let bytes = template.encode(&limits).unwrap();
    let mut reader = Reader::new(&bytes);
    assert_eq!(w::Template::decode(&limits, &mut reader), Ok(template.clone()));
    assert_eq!(reader.remaining(), 0);

    let line = w::SettingLine {
        setting: w::Setting::Template(w::SettingNumber { number: template.number }),
        shown: Some(template.shown),
        follows: w::SettingFollows::Change,
        last: Some(4),
        application: None,
    };
    let bytes = line.encode(&limits).unwrap();
    let mut reader = Reader::new(&bytes);
    assert_eq!(w::SettingLine::decode(&limits, &mut reader), Ok(line));
    assert_eq!(reader.remaining(), 0);
}

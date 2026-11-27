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
fn template_and_inventory_names_round_trip() {
    use crate as w;
    use crate::Record as _;
    use skein_lib::{List, Reader};

    let limits = super::fixtures::limits();
    let mut template = w::Template {
        number: 33,
        name: None,
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
    for name in [None, Some(b"lean-goal".as_slice().into())] {
        template.name = name;
        let bytes = template.encode(&limits).unwrap();
        let mut reader = Reader::new(&bytes);
        assert_eq!(w::Template::decode(&limits, &mut reader), Ok(template.clone()));
        assert_eq!(reader.remaining(), 0);
    }

    let line = w::SettingLine {
        setting: w::Setting::Template(w::SettingNumber { number: template.number }),
        name: template.name,
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

#[test]
fn a_charter_name_round_trips() {
    use crate::{self as w, Record as _};
    use skein_lib::{Duration, List, Reader};

    let limits = super::fixtures::limits();
    let mut value = w::Charter {
        name: None,
        rest: false,
        number: 34,
        instructions: b"design".as_slice().into(),
        waiting: Duration::ZERO,
        resume: false,
        turns: 1,
        time: Duration::from_secs(1),
        reserve_turns: 0,
        reserve_time: Duration::ZERO,
        model: w::SettingModel {
            provider: 1,
            dialect: 1,
            endpoint: 1,
            program: None,
            name: b"model".as_slice().into(),
            window: 4096,
            output: 1024,
            input_price: 1,
            cached_price: 1,
            cache_write_price: None,
            output_price: 1,
            price_unit: 1000,
            efforts: List::with_capacity(0),
            effort: None,
            speeds: List::with_capacity(0),
            speed: None,
        },
        alternatives: List::with_capacity(0),
        inspect: true,
        modify: false,
        shell: false,
        agents: false,
        call_timeout: Duration::from_secs(1),
        tools: 1,
        hosts: w::SettingHosts::Both,
        spend: 10,
        wake: w::WakePolicy {
            words: w::WakeRule::Immediate,
            notices: w::WakeRule::Immediate,
            news: w::WakeRule::Immediate,
            results: w::ResultsWake::LastOrFailure,
            questions: true,
            answers: true,
            timers: true,
        },
        documents: List::with_capacity(0),
        parts: List::with_capacity(0),
    };
    for name in [None, Some(b"designer".as_slice().into())] {
        value.name = name;
        let bytes = value.encode(&limits).unwrap();
        let mut reader = Reader::new(&bytes);
        assert_eq!(w::Charter::decode(&limits, &mut reader), Ok(value.clone()));
        assert_eq!(reader.remaining(), 0);
    }
}

#[test]
fn a_setting_policy_name_round_trips() {
    use crate::{self as w, Record as _};
    use skein_lib::{List, Reader};

    let limits = super::fixtures::limits();
    let mut value = w::SettingPolicy {
        name: None,
        escalation_role: None,
        ceiling: super::fixtures::authority(),
        projections: List::with_capacity(0),
        period_spend: 10,
        roles: List::with_capacity(0),
        requirements: List::with_capacity(0),
        permissions: List::with_capacity(0),
        templates: List::with_capacity(0),
    };
    for name in [None, Some(b"lean-policy".as_slice().into())] {
        value.name = name;
        let bytes = value.encode(&limits).unwrap();
        let mut reader = Reader::new(&bytes);
        assert_eq!(w::SettingPolicy::decode(&limits, &mut reader), Ok(value.clone()));
        assert_eq!(reader.remaining(), 0);
    }
}

#[test]
fn me_carries_named_charters() {
    use crate::{self as w, Record as _};
    use skein_lib::Reader;

    let limits = super::fixtures::document_limits();
    let mut me = super::fixtures::frame_at_ceilings(&limits);
    me.charters = super::fixtures::repeated(w::MeCharter { number: 1, name: b"implementer".as_slice().into() }, 64);
    let bytes = me.encode(&limits).unwrap();
    let mut reader = Reader::new(&bytes);
    assert_eq!(w::Me::decode(&limits, &mut reader), Ok(me));
    assert_eq!(reader.remaining(), 0);
}

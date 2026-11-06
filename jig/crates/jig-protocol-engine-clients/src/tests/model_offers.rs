//! Model offers across engine, wire and client link (jig/docs/design/client/domain/contract.md, 5.1).
use super::fixtures;
use jig_client_domain::contract as c;
use jig_client_protocol_wire::v19 as w;
use skein_lib::{List, Reader};

fn list<T, const N: usize>(items: [T; N]) -> List<T> {
    List::from_boxed(Box::new(items))
}

fn me_with_offer(model: w::MeModel, defaults: List<w::MeDefault>) -> w::Me {
    let limits = fixtures::client_limits();
    let project = w::MeProject::new(
        limits.wire(),
        w::MeProjectParts {
            defaults,
            public: None,
            project: 3,
            name: w::DocumentText::new(limits.wire(), w::DocumentTextParts { bytes: Box::from(*b"Tally"), cut: false })
                .unwrap(),
            role: w::HeldRole::Owner,
            pool: None,
            templates: Box::default(),
            creating: None,
            models: list([model]),
        },
    )
    .unwrap();
    let mut encoded = skein_lib::Writer::new(usize::try_from(project.measure()).unwrap());
    project.encode(&mut encoded).unwrap();
    let encoded = encoded.finish();
    let mut pieces = skein_lib::Writer::new(encoded.len().checked_add(4).unwrap());
    skein_codec::piece::write(&mut pieces, &encoded).unwrap();
    let mut me = jig_client_test_records::frame_record().into_parts();
    me.projects = pieces.finish();
    let me = w::Me::new(limits.wire(), me).unwrap();
    let mut encoded = skein_lib::Writer::new(usize::try_from(me.measure()).unwrap());
    me.encode(&mut encoded).unwrap();
    w::Me::decode(limits.wire(), &mut Reader::new(&encoded.finish())).unwrap()
}

#[test]
fn model_offers_and_template_previews_cross_engine_wire_and_link_whole() {
    use crate as engine;
    let limits = fixtures::client_limits();
    for populated in [false, true] {
        let model = engine::Model {
            name: b"chosen".as_slice().into(),
            provider: Some(3),
            window: 8192,
            output: 1024,
            efforts: if populated {
                Box::new([Box::from(b"low".as_slice()), Box::from(b"high".as_slice())])
            } else {
                Box::new([])
            },
            effort: if populated { Some(b"high".as_slice().into()) } else { None },
            prices: engine::Prices { input: 2, cached: 1, cache_write: None, output: 7, unit: 1000 },
            description: None,
            shown: if populated { Some(b"Shown model".as_slice().into()) } else { None },
            picker: !populated,
        };
        let wire = engine::translate::llm::models(
            &[jig_domain_core::Offered { model: 0, accounts: List::with_capacity(0) }],
            core::slice::from_ref(&model),
            limits.wire(),
        )
        .unwrap();
        let mut wire = wire.get(0).unwrap().clone().into_parts();
        assert!(wire.accounts.is_empty(), "surface leaves accounts empty");
        wire.accounts = if populated { list([42, 7]) } else { List::with_capacity(0) };
        let mut defaults = List::with_capacity(1);
        let preview = jig_domain_tasks::LlmChoice {
            account: None,
            main: jig_domain_tasks::ModelEffort { model: 0, effort: if populated { Some(1) } else { None } },
            subagents: if populated {
                Box::new([jig_domain_tasks::ModelEffort { model: 0, effort: Some(0) }])
            } else {
                Box::new([])
            },
        };
        let choice = engine::translate::llm::choice(Some(&preview), core::slice::from_ref(&model), limits.wire())
            .unwrap()
            .unwrap();
        defaults.push(w::MeDefault::new(limits.wire(), w::MeDefaultParts { template: 9, choice }).unwrap()).unwrap();
        let decoded = jig_client_protocol_link::translate::documents::frame(
            me_with_offer(w::MeModel::new(limits.wire(), wire).unwrap(), defaults),
            &limits,
        )
        .unwrap();
        let project = decoded.projects.get(0).unwrap();
        let text = |bytes: &[u8]| c::Text::new(bytes.into(), 4096).unwrap();
        assert_eq!(
            project.models.get(0).unwrap(),
            &c::MeModel {
                name: text(b"chosen"),
                provider: Some(3),
                window: 8192,
                output: 1024,
                efforts: if populated { list([text(b"low"), text(b"high")]) } else { List::with_capacity(0) },
                effort: if populated { Some(text(b"high")) } else { None },
                prices: c::ModelPrices { input: 2, cached: 1, cache_write: None, output: 7, unit: 1000 },
                description: None,
                shown: if populated { Some(text(b"Shown model")) } else { None },
                accounts: if populated { list([42, 7]) } else { List::with_capacity(0) },
                picker: !populated,
            }
        );
        assert_eq!(
            project.defaults.get(0).unwrap(),
            &c::MeDefault {
                template: 9,
                choice: c::LlmChoice {
                    account: None,
                    main: c::ModelEffort {
                        model: text(b"chosen"),
                        effort: if populated { Some(text(b"high")) } else { None }
                    },
                    subagents: if populated {
                        list([c::ModelEffort { model: text(b"chosen"), effort: Some(text(b"low")) }])
                    } else {
                        List::with_capacity(0)
                    },
                }
            }
        );
    }
}

use crate::tests::fixtures;
use crate::{
    config::{self, schema as source},
    render::{self, revision::declaration as target},
    translate::declarations,
};
use skein_lib::bytes::copy_of;

fn position_back(at: render::revision::Position) -> config::Position {
    config::Position { offset: 0, line: at.line, column: at.column }
}
fn text_back(text: target::Text) -> config::declaration::Text {
    let target::Text { at, text } = text;
    config::declaration::Text { at: position_back(at), text }
}
fn form_back(form: target::Form) -> source::Form {
    match form {
        target::Form::Bool => source::Form::Bool,
        target::Form::Int => source::Form::Int,
        target::Form::String => source::Form::String,
        target::Form::Path => source::Form::Path,
        target::Form::Mode => source::Form::Mode,
        target::Form::Duration => source::Form::Duration,
        target::Form::Secret => source::Form::Secret,
        target::Form::Target => source::Form::Target,
        target::Form::File => source::Form::File,
        target::Form::Enum(values) => {
            let mut texts = skein_lib::List::with_capacity(u32::try_from(values.len()).expect("fixture count"));
            for value in values {
                texts.push(text_back(value)).expect("room");
            }
            source::Form::Enum(texts.into_boxed())
        }
        target::Form::List(ty) => source::Form::List(ty),
        target::Form::Map(ty) => source::Form::Map(ty),
        target::Form::Object(specs) => {
            let mut fields = skein_lib::List::with_capacity(u32::try_from(specs.len()).expect("fixture count"));
            for spec in specs {
                fields.push(spec_back(spec)).expect("room");
            }
            source::Form::Object(fields.into_boxed())
        }
    }
}
fn number_back(number: Option<target::Number>) -> Option<config::declaration::Number> {
    let target::Number { at, value } = number?;
    Some(config::declaration::Number { at: position_back(at), value })
}
fn spec_back(spec: target::Spec) -> source::Spec {
    let target::Spec { name, at, ty, default, optional, min, max, max_length } = spec;
    assert_eq!(default, None, "form fixture has no defaults");
    source::Spec {
        name: text_back(name),
        at: position_back(at),
        ty,
        default: None,
        optional,
        min: number_back(min),
        max: number_back(max),
        max_length: number_back(max_length),
    }
}
#[test]
fn each_form_translates_to_render_and_back() {
    for (row, input) in fixtures::every_form().into_iter().enumerate() {
        let back = form_back(declarations::form(input.clone(), &crate::limits::SHIPPED));
        assert_eq!(back, input, "row {row}: form");
    }
}

#[test]
fn a_declaration_s_types_settings_and_reads_cross_unchanged() {
    let declaration = fixtures::read_declaration(fixtures::PARAMS);
    let setting = fixtures::only(&declaration.settings);
    assert_eq!(
        (declaration.types.len(), &*setting.name.text, setting.ty, declaration.reads.as_ref()),
        (4, b"params".as_slice(), 0, &[][..])
    );
}
#[test]
fn a_declaration_s_nested_types_keep_their_forms_positions_and_indexes() {
    let declaration = fixtures::read_declaration(fixtures::PARAMS);
    let types = fixtures::parameter_types(&declaration);
    assert_eq!(
        (&types.map.form, types.map.at),
        (&target::Form::Map(1), render::revision::Position { line: 2, column: 11 })
    );
    assert_eq!(
        (&types.int.form, types.int.at),
        (&target::Form::Int, render::revision::Position { line: 4, column: 23 })
    );
    assert_eq!(
        (&types.string.form, types.string.at),
        (&target::Form::String, render::revision::Position { line: 3, column: 23 })
    );
    assert_eq!(
        (&*types.order.name.text, types.order.ty, &*types.param.name.text, types.param.ty),
        (b"order".as_slice(), 2, b"param".as_slice(), 3)
    );
}
#[test]
fn a_declaration_s_bounds_keep_their_numbers_and_positions() {
    let declaration = fixtures::read_declaration(fixtures::PARAMS);
    let types = fixtures::parameter_types(&declaration);
    let order = types.order;
    assert_eq!(
        (&order.min, &order.max),
        (
            &Some(target::Number { at: render::revision::Position { line: 4, column: 37 }, value: 0 }),
            &Some(target::Number { at: render::revision::Position { line: 4, column: 47 }, value: 999 })
        )
    );
}
#[test]
fn a_declaration_s_defaults_keep_their_values_and_positions() {
    let declaration = fixtures::read_declaration(fixtures::PARAMS);
    let types = fixtures::parameter_types(&declaration);
    let order = types.order;
    assert_eq!(
        &order.default,
        &Some(render::revision::Value {
            at: render::revision::Position { line: 4, column: 63 },
            kind: render::revision::Kind::Int(500)
        })
    );
    let setting = fixtures::only(&declaration.settings);
    assert_eq!(
        &setting.default,
        &Some(render::revision::Value {
            at: render::revision::Position { line: 5, column: 14 },
            kind: render::revision::Kind::Map(Box::new([]))
        })
    );
}
#[test]
fn a_declaration_s_reads_keep_their_positions_settings_and_flags() {
    let declaration = fixtures::read_declaration(fixtures::READS);
    assert_eq!(
        declaration.reads.as_ref(),
        &[target::Read {
            task: target::Text { at: render::revision::Position { line: 1, column: 11 }, text: copy_of(b"core") },
            at: render::revision::Position { line: 1, column: 18 },
            settings: Box::new([target::Text {
                at: render::revision::Position { line: 1, column: 31 },
                text: copy_of(b"domain")
            }]),
            exports: true,
            optional: true
        }]
    );
}

#[test]
fn a_declarations_requires_cross_to_render_and_back() {
    for (row, (guard, input)) in fixtures::guard_conditions().into_iter().enumerate() {
        let guard = declarations::guard(guard, &crate::limits::SHIPPED.template).expect("guard compiles");
        let back = super::back::expression(guard.when.expression);
        assert_eq!(back, input, "row {row}: guard condition");
    }
}

#[test]
fn script_order_defaults_to_500() {
    let declaration = fixtures::read_declaration(br#"{"scripts":{"customize.sh":{}}}"#);
    assert_eq!(fixtures::only(&declaration.scripts).order, fixtures::DEFAULT_SCRIPT_ORDER);
}

#[test]
fn declared_scripts_keep_their_fragments_positions_and_orders() {
    let declaration =
        fixtures::read_declaration(br#"{"scripts":{"customize.sh":{"order":300},"software.sh":{"order":200}}}"#);
    assert_eq!(
        declaration.scripts.as_ref(),
        [
            target::Script {
                fragment: target::Fragment::Customize,
                at: render::revision::Position { line: 1, column: 13 },
                order: 300
            },
            target::Script {
                fragment: target::Fragment::Software,
                at: render::revision::Position { line: 1, column: 42 },
                order: 200
            },
        ]
    );
}

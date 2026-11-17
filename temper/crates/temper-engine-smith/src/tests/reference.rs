#![expect(clippy::disallowed_methods, reason = "step fixtures validate the renderer's UTF-8 output")]

use skein_lib::Writer;
use smith_domain_run::HostEffect;

use crate::{
    calls::{DESCRIPTION_BYTES, MEMBER_BYTES, Offer, SPECS, Spec},
    json::{self, Value},
    reference::{reference, render},
};

const HEAD: &[u8] = b"# Host tools\n\nKind: reference\n\nEvery tool temper adds to its runs' own: what it does, as a run reads it, whether it reads or writes, and the members of its input.\n\nA run calls only the tools in its own list: a tool here that your list lacks is not yours.\n\nMembers are optional unless marked required. Numeric bounds in brackets are inclusive.\n\nFile, pull-file and compare replies fit min(core.call_answer_bytes minus tool name, agent.json profile.declared.tool_payload); raising the former cannot override the latter. Oversized comparisons give measured size and recovery. Forge refusals name the bound and exact size or at least bytes read; unknown checks say an unnamed bound, and figureless forge bounds say a bound of the forge's. All refusals say how to read on. Fitted file prefixes keep [file truncated] and size/recovery feedback if room permits. max_bytes selects a raw prefix without offset; cuts are incomplete. If metadata leaves no content room, shorten the path or raise the allowance. Oversized pull-file pages keep paths as room permits, distinguishing omitted contents from absent sides. Recover omitted paths by listing head/base directories; read paths at head/base commits, resolving base separately. Later pages list other files.\n";

fn fixture(schema: &'static [u8], effect: HostEffect) -> Box<[u8]> {
    let specs = [Spec {
        name: b"fixture_read",
        description: b"Read a fixture.",
        schema,
        effect,
        whole: None,
        offer: Offer::Every,
    }];
    let mut out = Writer::new(temper_docs::PAGE_BYTES);
    render(&specs, &mut out).expect("fixture fits page bound");
    let mut exact = Writer::new(out.written());
    render(&specs, &mut exact).expect("fixture fits measured length");
    exact.finish()
}

#[test]
fn a_tool_renders_its_description_effect_and_members() {
    let schema = br#"{
        "type":"object",
        "properties":{
            "number":{"type":"integer","minimum":1,"maximum":9,"description":"a number"},
            "page":{"type":"integer","minimum":1,"default":1},
            "kind":{"type":"string","enum":["issue","pull"],"description":"which"},
            "name":{"type":"string"},
            "limit":{"type":"integer","maximum":7},
            "flag":{"type":"boolean"},
            "shape":{"type":"object","properties":{"nested":{"type":"string"}}},
            "list":{"type":"array","items":{"type":"string"}},
            "any":{}
        },
        "required":["number","name"]
    }"#;
    let expected = b"\n## fixture_read\n\nRead a fixture.\n\nA read.\nOffered to every run but an asking one.\n\n\
- `number`: integer [1, 9]; required. a number\n\
- `page`: integer >=1, 1 when absent.\n\
- `kind`: one of `issue|pull`. which\n\
- `name`: text; required.\n\
- `limit`: integer <=7.\n\
- `flag`: boolean.\n\
- `shape`: object.\n  - `nested`: text.\n\
- `list`: array.\n\
- `any`: any value.\n";
    assert_eq!(fixture(schema, HostEffect::Read).as_ref(), [HEAD, expected].concat());
}

#[test]
fn a_tool_with_no_members_says_its_input_is_empty() {
    let expected = b"\n## fixture_read\n\nRead a fixture.\n\nA write.\nOffered to every run but an asking one.\n\nIts input is `{}`.\n";
    for schema in [br#"{"type":"object","properties":{}}"#.as_slice(), br#"{"type":"object"}"#] {
        assert_eq!(fixture(schema, HostEffect::Write).as_ref(), [HEAD, expected].concat());
    }
}

#[test]
fn the_page_holds_every_declared_tool_in_the_tables_order() {
    let mut out = Writer::new(temper_docs::PAGE_BYTES);
    reference(&mut out).expect("page fits PAGE_BYTES");
    let mut exact = Writer::new(out.written());
    reference(&mut exact).expect("page fits measured length");
    let page = exact.finish();
    let text = core::str::from_utf8(&page).expect("reference UTF-8");
    let mut headings = text.lines();
    for spec in &SPECS {
        let mut heading = None;
        for line in headings.by_ref() {
            if let Some(name) = line.strip_prefix("## ") {
                heading = Some(name.as_bytes());
                break;
            }
        }
        assert_eq!(heading, Some(spec.name));
    }
    for line in headings {
        assert!(!line.starts_with("## ") || line == "## Shared forms", "no undeclared tool heading");
    }
    let mut tail = SPECS.iter().rev();
    for name in [b"remember".as_slice(), b"document", b"see", b"read_docs"] {
        assert_eq!(tail.next().unwrap().name, name);
    }

    assert!(page.len() <= temper_docs::PAGE_BYTES);
}

#[test]
fn each_offered_line_agrees_with_the_runs_declarations() {
    use jig_domain_tasks::Tools;
    let families = jig_domain_core::ToolFamilies::JIG;
    let mut measured = Writer::new(temper_docs::PAGE_BYTES);
    reference(&mut measured).expect("page fits");
    let mut out = Writer::new(measured.written());
    reference(&mut out).expect("page fits measured length");
    let page = out.finish();
    let page = core::str::from_utf8(&page).expect("page UTF-8");
    for charter in [
        0,
        families.delegate.0,
        families.settings,
        families.delegate.0 | families.settings,
        families.ask.0,
        families.ask.0 | families.settings,
        families.ask.0 | families.delegate.0,
        families.ask.0 | families.delegate.0 | families.settings,
        u64::MAX,
    ] {
        for bits in 0_u64..16 {
            let authority = if bits & 1 != 0 { families.read.0 } else { 0 }
                | if bits & 2 != 0 { families.message.0 } else { 0 }
                | if bits & 4 != 0 { families.settings } else { 0 }
                | if bits & 8 != 0 { families.ask.0 } else { 0 };
            let offered = crate::calls::tools_for(skein_lib::Duration::from_secs(1), Tools(charter), Tools(authority));
            let asking = temper_engine_domain::asking(Tools(charter));
            for spec in &SPECS {
                let name = core::str::from_utf8(spec.name).expect("tool UTF-8");
                let (_, entry) = page.split_once(&format!("\n## {name}\n")).expect("tool on page");
                let entry = entry.split("\n## ").next().expect("entry");
                let line = entry.lines().find(|line| line.starts_with("Offered ")).expect("offering line");
                let expected = match line {
                    "Offered to every run but an asking one." => !asking,
                    "Offered where the run's charter names the settings family and its authority holds it." => {
                        charter & authority & families.settings != 0 && (!asking || name == "propose_settings")
                    }
                    "Offered to an asking run, where its authority holds the asking family." => {
                        asking && authority & families.ask.0 != 0
                    }
                    other => {
                        let family = other
                            .strip_prefix("Offered where the run's charter names the ")
                            .and_then(|rest| rest.strip_suffix(" family and its authority holds it."))
                            .expect("family offer line");
                        let bit = match family {
                            "delegate" => families.delegate.0,
                            "message" => families.message.0,
                            "control" => families.control.0,
                            "decide" => families.decide.0,
                            "propose" => families.propose.0,
                            "subscribe" => families.subscribe.0,
                            "effect" => families.effect.0,
                            "note" => families.note.0,
                            "recall" => families.recall.0,
                            "read" => families.read.0,
                            other => panic!("unknown offering family: {other}"),
                        };
                        !asking && charter & authority & bit != 0
                    }
                };
                assert_eq!(
                    offered.iter().any(|tool| tool.name.as_ref() == spec.name),
                    expected,
                    "{name}: charter {charter}, authority {authority}: {line}"
                );
            }
        }
    }
}

#[test]
fn member_types_keep_enum_order_and_json_defaults() {
    let schema = br#"{"properties":{
        "one":{"enum":["a"],"type":"integer"},
        "two":{"enum":["b","a"]},
        "three":{"enum":["a","b","c"]},
        "number":{"type":"number","minimum":-1.20e2,"maximum":4.5,"default":1e+2},
        "integer":{"type":"integer"}
    }}"#;
    let expected = br"
## fixture_read

Read a fixture.

A read.
Offered to every run but an asking one.

- `one`: one of `a`.
- `two`: one of `b|a`.
- `three`: one of `a|b|c`.
- `number`: number [-1.20e2, 4.5], 1e+2 when absent.
- `integer`: integer.
";
    assert_eq!(fixture(schema, HostEffect::Read).as_ref(), [HEAD, expected].concat());
}

/// A program run lists these tools over MCP, whose API requires an object root with alternatives
/// beneath a property: smith refuses the whole inventory otherwise, and the run fails at its start.
#[test]
fn every_schema_root_is_an_object_with_no_alternatives() {
    for spec in &SPECS {
        let tool = core::str::from_utf8(spec.name).expect("tool name UTF-8");
        let Value::Object(members) = json::parse(spec.schema).expect("declared schema") else {
            panic!("{tool}: schema root is not an object");
        };
        for (key, _) in &members {
            assert!(
                ![b"oneOf".as_slice(), b"allOf", b"anyOf"].contains(&key.as_ref()),
                "{tool}: a root alternative, which smith refuses",
            );
        }
    }
}

#[test]
fn every_description_is_within_its_bound() {
    for spec in &SPECS {
        let tool = core::str::from_utf8(spec.name).expect("tool name UTF-8");
        assert!(spec.description.len() <= DESCRIPTION_BYTES, "{tool}: description exceeds {DESCRIPTION_BYTES} bytes");
        let schema = json::parse(spec.schema).expect("declared schema");
        member_descriptions(&schema, tool, "input");
        let whole = json::parse(spec.whole()).expect("whole schema");
        member_descriptions(&whole, tool, "input");
        let properties = whole.required(b"properties").unwrap();
        let Value::Object(properties) = properties else { panic!("whole properties") };
        for (name, member) in properties {
            let name = core::str::from_utf8(name).unwrap();
            let description = member.required(b"description").unwrap().text(b"description").unwrap();
            assert!(!description.is_empty(), "{tool}.{name}: top-level description present");
        }
    }
}

fn member_descriptions(schema: &Value, tool: &str, member: &str) {
    match schema {
        Value::Object(fields) => {
            for (key, value) in fields {
                match key.as_ref() {
                    b"description" => {
                        let Value::String(text) = value else { panic!("{tool}.{member}: description is text") };
                        assert!(
                            text.len() <= MEMBER_BYTES,
                            "{tool}.{member}: description exceeds {MEMBER_BYTES} bytes"
                        );
                    }
                    b"properties" => {
                        let Value::Object(properties) = value else { panic!("declared properties are an object") };
                        for (name, property) in properties {
                            let name = core::str::from_utf8(name).expect("member name UTF-8");
                            member_descriptions(property, tool, &format!("{member}.{name}"));
                        }
                    }
                    b"items" => member_descriptions(value, tool, &format!("{member}[]")),
                    _ => member_descriptions(value, tool, member),
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                member_descriptions(value, tool, member);
            }
        }
        Value::String(_) | Value::Number(_) | Value::Boolean(_) | Value::Null => {}
    }
}

#[test]
fn a_whole_form_renders_nested_members_and_alternatives() {
    let whole = br#"{"properties":{"batch":{"type":"array","items":{"type":"object","properties":{"executor":{"type":"object","properties":{"kind":{"type":"string"}},"required":["kind"]}},"required":["executor"]}},"tracked":{"oneOf":[{"type":"object","properties":{"priority":{"type":"integer","minimum":0}},"required":["priority"]},{"type":"string"}]}} ,"required":["batch"]}"#;
    let specs = [Spec {
        name: b"fixture_read",
        description: b"Read a fixture.",
        schema: b"{}",
        whole: Some(whole),
        effect: HostEffect::Read,
        offer: Offer::Every,
    }];
    let mut out = Writer::new(temper_docs::PAGE_BYTES);
    render(&specs, &mut out).unwrap();
    let mut exact = Writer::new(out.written());
    render(&specs, &mut exact).unwrap();
    let page = exact.finish();
    let text = core::str::from_utf8(&page).unwrap();
    assert!(text.ends_with("- `batch`: array; required.\n  - `executor`: object; required.\n    - `kind`: text; required.\n- `tracked`: object or string.\n  - `priority`: integer >=0; required.\n"));
}

fn page_for(specs: &[Spec]) -> Box<[u8]> {
    let mut measured = Writer::new(temper_docs::PAGE_BYTES);
    render(specs, &mut measured).expect("whole forms fit page");
    let mut out = Writer::new(measured.written());
    render(specs, &mut out).unwrap();
    out.finish()
}

#[test]
fn every_whole_form_fact_is_on_the_page() {
    let page = page_for(&SPECS);
    let text = core::str::from_utf8(&page).unwrap();
    let (members, values) = page_facts(text);
    for spec in &SPECS {
        whole_form_facts(&json::parse(spec.whole()).unwrap(), text, &members, &values);
    }
}

type Members<'a> = alloc::collections::BTreeMap<&'a str, skein_lib::List<&'a str>>;
type Values<'a> = alloc::collections::BTreeSet<&'a str>;

fn page_facts(page: &str) -> (Members<'_>, Values<'_>) {
    let mut members = Members::new();
    let capacity = u32::try_from(page.lines().count()).unwrap();
    for line in page.lines() {
        if let Some(line) = line.trim_start().strip_prefix("- `")
            && let Some((name, rest)) = line.split_once("`: ")
        {
            members
                .entry(name)
                .or_insert_with(|| skein_lib::List::with_capacity(capacity))
                .push(rest)
                .expect("at most one member per page line");
        }
    }
    let mut values = Values::new();
    for span in page.split('`').skip(1).step_by(2) {
        for item in span.split('|') {
            values.insert(item);
        }
    }
    (members, values)
}

fn whole_form_facts(schema: &Value, page: &str, members: &Members<'_>, values: &Values<'_>) {
    if let Some(Value::Object(properties)) = schema.field(b"properties").unwrap() {
        for (name, member) in properties {
            let name = core::str::from_utf8(name).unwrap();
            let required = match schema.field(b"required").unwrap() {
                Some(Value::Array(names)) => names.contains(&Value::String(name.as_bytes().into())),
                _ => false,
            };
            let description = match member.field(b"description").unwrap() {
                Some(Value::String(text)) => core::str::from_utf8(text).unwrap(),
                _ => "",
            };
            let kind = match member.field(b"type").unwrap() {
                Some(Value::String(text)) if text.as_ref() == b"string" => "text",
                Some(Value::String(text)) => core::str::from_utf8(text).unwrap(),
                _ => "",
            };
            assert!(
                members.get(name).is_some_and(|lines| lines.iter().any(|line| {
                    let kind_present = if member.field(b"enum").unwrap().is_some() {
                        line.starts_with("one of `") || line.starts_with("text")
                    } else if member.field(b"oneOf").unwrap().is_some() {
                        // The variants' types are all on the carrier's line.
                        let variants = member.required(b"oneOf").unwrap().array(b"oneOf").unwrap();
                        variants.iter().all(|variant| match variant.field(b"type").unwrap() {
                            Some(Value::String(kind)) => line.contains(core::str::from_utf8(kind).unwrap()),
                            _ => true,
                        })
                    } else {
                        line.starts_with(kind)
                    };
                    kind_present && line.contains("; required.") == required && line.contains(description)
                })),
                "{name}: type {kind}, required {required}, description {description}"
            );
            whole_form_facts(member, page, members, values);
        }
    }
    if let Some(Value::Array(variants)) = schema.field(b"enum").unwrap() {
        for value in variants {
            let Value::String(value) = value else { panic!("enum strings") };
            let value = core::str::from_utf8(value).unwrap();
            assert!(values.contains(value), "enum value {value}");
        }
    }
    if let Some(items) = schema.field(b"items").unwrap() {
        if let Some(Value::String(description)) = items.field(b"description").unwrap() {
            assert!(page.contains(core::str::from_utf8(description).unwrap()));
        }
        whole_form_facts(items, page, members, values);
    }
    for alternative in [b"oneOf".as_slice(), b"anyOf", b"allOf"] {
        if let Some(Value::Array(variants)) = schema.field(alternative).unwrap() {
            for variant in variants {
                whole_form_facts(variant, page, members, values);
            }
        }
    }
}

#[test]
fn repeated_shapes_are_shared_by_value_with_distinct_names() {
    let wake_a = br#"{"properties":{"wake":{"type":"object","description":"First wake.","properties":{"words":{"type":"object","properties":{"count":{"type":"integer","description":"How many."}},"required":["count"]}},"required":["words"]},"kind":{"enum":["a","b"]}}}"#;
    let wake_b = br#"{"properties":{"wake":{"description":"Second wake.","type":"object","required":["words"],"properties":{"words":{"required":["count"],"properties":{"count":{"description":"How many.","type":"integer"}},"type":"object"}}},"kind":{"enum":["b","a"]}}}"#;
    let other = br#"{"properties":{"kind":{"enum":["x","y"]}}}"#;
    let specs = [
        Spec {
            name: b"one",
            description: b"One.",
            schema: wake_a,
            whole: None,
            effect: HostEffect::Read,
            offer: Offer::Every,
        },
        Spec {
            name: b"two",
            description: b"Two.",
            schema: wake_b,
            whole: None,
            effect: HostEffect::Read,
            offer: Offer::Every,
        },
        Spec {
            name: b"three",
            description: b"Three.",
            schema: other,
            whole: None,
            effect: HostEffect::Read,
            offer: Offer::Every,
        },
        Spec {
            name: b"four",
            description: b"Four.",
            schema: other,
            whole: None,
            effect: HostEffect::Read,
            offer: Offer::Every,
        },
    ];
    let page = page_for(&specs);
    let text = core::str::from_utf8(&page).unwrap();
    assert_eq!(text.matches("the `wake` form, under Shared forms.").count(), 2);
    assert_eq!(text.matches("\n### wake\n").count(), 1);
    assert_eq!(text.matches("- `count`: integer; required. How many.").count(), 1);
    assert!(text.contains("First wake. the `wake` form"));
    assert!(text.contains("Second wake. the `wake` form"));
    assert!(text.contains("\n## Shared forms\n\n### wake\n"), "largest subtree is factored first");
    assert!(!text.contains("\n### words\n"), "a subtree carried only by the shared wake isn't repeated");
    assert_eq!(text.matches("the `kind` form, under Shared forms.").count(), 2);
    assert_eq!(text.matches("the `kind-2` form, under Shared forms.").count(), 2);
    let (members, values) = page_facts(text);
    for spec in &specs {
        whole_form_facts(&json::parse(spec.whole()).unwrap(), text, &members, &values);
    }
}

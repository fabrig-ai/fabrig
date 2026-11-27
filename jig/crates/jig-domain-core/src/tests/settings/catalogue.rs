//! List patterns match whole names and drops win (jig/docs/design/protocol/configuration-engine.md, 1).
use crate::settings::catalogue::{CatalogueModel, ModelSource, NEW_FOR, from_list, kept, matches, shape};
use crate::settings::{self, Edit, Item, Setting, Value};
use core::slice::from_ref;
use jig_domain_accounts::ProviderList;
use jig_domain_authority::Models;
use jig_domain_link::ListedModel;
use skein_lib::{Duration, List, Wall};

#[test]
fn catalogue_patterns_match_whole_names_with_backtracking() {
    for (pattern, name, expected) in [
        (b"gpt-*".as_slice(), b"gpt-6-luna".as_slice(), true),
        (b"gpt-*", b"xgpt-6", false),
        (b"*audio*", b"audio-model", true),
        (b"*audio*", b"gpt-audio-model", true),
        (b"*audio*", b"gpt-audio", true),
        (b"*audio*", b"gpt-6", false),
        (b"*-20??-??-??", b"gpt-4o-2024-08-06", true),
        (b"*-20??-??-??", b"gpt-4o-audio", false),
        (b"?", b"a", true),
        (b"?", b"", false),
        (b"?", b"ab", false),
        (b"?", "é".as_bytes(), false),
        (b"??", "é".as_bytes(), true),
        (b"gpt-*", b"gpt-", true),
        (b"*", b"", true),
        (b"", b"", true),
        (b"", b"a", false),
        (b"**a**b*c?d", b"xaaybzzcxd", true),
        (b"*ab*ab?c", b"xabababxc", true),
        (b"*ab*ab?c", b"xabababxd", false),
        (b"a*b", b"abx", false),
        (b"**", b"abc", true),
    ] {
        assert_eq!(matches(pattern, name), expected, "{pattern:?} on {name:?}");
    }
}

#[test]
#[expect(clippy::disallowed_methods, reason = "ordinary bounded table fixtures (programming-model.md, 10.2)")]
fn catalogue_drop_patterns_win_over_keeps() {
    for (keep, drop, name, expected) in [
        (&[b"other*".as_slice(), b"gpt-*"][..], &[b"*audio*".as_slice()][..], b"gpt-6".as_slice(), true),
        (&[b"gpt-*".as_slice()][..], &[b"*audio*".as_slice(), b"*preview*".as_slice()][..], b"gpt-audio", false),
        (&[b"gpt-*".as_slice()][..], &[][..], b"other", false),
        (&[][..], &[][..], b"gpt-6", false),
        (&[b"*".as_slice()][..], &[][..], b"gpt-6", true),
        (&[b"gpt-*".as_slice()][..], &[b"*audio*".as_slice(), b"*preview*".as_slice()][..], b"gpt-preview", false),
    ] {
        let list = ProviderList {
            keep: keep.iter().map(|pattern| Box::from(*pattern)).collect(),
            drop: drop.iter().map(|pattern| Box::from(*pattern)).collect(),
        };
        assert_eq!(kept(name, &list), expected, "{list:?} on {name:?}");
    }
}

const READ_AT: Wall = Wall::from_nanos(1_000_000_000);
// Model numbers 64, 63 and 62 are bits 63, 62 and 61 (domain/engine/settings-catalogue.md, 1).
const LAST: u64 = 0x8000_0000_0000_0000;
const LAST_TWO: u64 = 0xc000_0000_0000_0000;
const LAST_THREE: u64 = 0xe000_0000_0000_0000;
// Two of the catalogue's 64 numbers are free.
const OCCUPIED: usize = 62;

struct ListCase {
    label: &'static str,
    provider: u32,
    models: Box<[CatalogueModel]>,
    rows: Box<[ListedModel]>,
    others: u64,
    expected: Box<[CatalogueModel]>,
    names: u64,
    left_out: u32,
}

fn list_case(
    label: &'static str,
    models: &[CatalogueModel],
    rows: &[ListedModel],
    others: u64,
    expected: &[CatalogueModel],
    names: u64,
    left_out: u32,
) -> ListCase {
    ListCase {
        label,
        provider: 1,
        models: models.into(),
        rows: rows.into(),
        others,
        expected: expected.into(),
        names,
        left_out,
    }
}

fn catalogue_model(number: u32, name: &[u8], source: ModelSource) -> CatalogueModel {
    let mut model = crate::tests::scripted_model();
    model.name = name.into();
    CatalogueModel {
        number,
        model,
        shown: Some(b"Shown".as_slice().into()),
        description: Some(b"Description".as_slice().into()),
        picker: true,
        withdrawn: false,
        source,
        estimated: false,
        first_read: None,
    }
}

// Seven stock settings and every deployment setting from 39 through 64 leave a gap at 8–38.
const STOCK_MODELS: u32 = 7;
const FIRST_ADDED_MODEL: u32 = 39;
const FIRST_HOLE_BIT: u64 = 1 << STOCK_MODELS;
const LAST_HOLE_BIT: u64 = 1 << (FIRST_ADDED_MODEL - 2);
const FIRST_ADDED_BIT: u64 = 1 << (FIRST_ADDED_MODEL - 1);
const ADDED_POLICY: Setting = Setting::Policy(settings::NUMBERS);

#[expect(clippy::disallowed_macros, reason = "numbered names for distinct models in a unit-test fixture")]
pub(super) fn sparse_catalogue() -> Box<[CatalogueModel]> {
    let mut models = List::with_capacity(settings::NUMBERS);
    let first = crate::tests::scripted_model();
    models.push(catalogue_model(1, &first.name, ModelSource::Stock)).expect("stock model");
    for (numbers, source) in
        [(2..=STOCK_MODELS, ModelSource::Stock), (FIRST_ADDED_MODEL..=settings::NUMBERS, ModelSource::Person)]
    {
        for number in numbers {
            models
                .push(catalogue_model(number, format!("model-{number}").as_bytes(), source))
                .expect("one setting per model");
        }
    }
    models.into_boxed()
}

fn check_model_authorities(
    rules_models: Models,
    ceiling_models: Models,
    role_models: Models,
) -> Result<(), settings::Failure> {
    // At most one edit per model setting, plus the rules and policy edits.
    const ITEMS: u32 = settings::NUMBERS + 2;
    let crate::Config { mut defaults, settings: facts, connectors, seed_projects, .. } = crate::tests::config(41);
    let mut stock = List::with_capacity(STOCK_MODELS);
    let mut items = List::with_capacity(ITEMS);
    for model in sparse_catalogue() {
        if model.number <= STOCK_MODELS {
            stock.push(model).expect("stock model");
        } else {
            items
                .push(Item { setting: Setting::Model(model.number), edit: Edit::Set(Value::Model(Box::new(model))) })
                .expect("added model");
        }
    }
    defaults.models = stock.into_boxed();
    let mut rules = defaults.rules.clone();
    rules.ceiling.models = rules_models;
    let mut policy = seed_projects.first().expect("fixture project").policy.clone();
    policy.permissions = Box::default();
    policy.policy.ceiling.models = ceiling_models;
    policy.policy.roles.first_mut().expect("owner role").authority.models = role_models;
    items.push(Item { setting: Setting::Rules, edit: Edit::Set(Value::Rules(Box::new(rules))) }).expect("rules edit");
    items.push(Item { setting: ADDED_POLICY, edit: Edit::Set(Value::Policy(Box::new(policy))) }).expect("policy edit");
    let limits = crate::tests::limits();
    let current = settings::Current::new(defaults, &limits, &[]);
    settings::check_change(&current, items.as_slice(), &facts, &connectors, &limits)
}

#[test]
fn authorities_naming_a_catalogue_hole_are_reference_faults() {
    for bit in [FIRST_HOLE_BIT, LAST_HOLE_BIT] {
        let hole = Models::Named(bit);
        let all = Models::All;
        for (name, rules, ceiling, role, setting) in [
            ("rules ceiling", hole, all, all, Setting::Rules),
            ("policy ceiling", all, hole, all, ADDED_POLICY),
            ("role authority", all, all, hole, ADDED_POLICY),
        ] {
            assert_eq!(
                check_model_authorities(rules, ceiling, role),
                Err(settings::Failure { setting, fault: settings::Fault::Reference, detail: None }),
                "{name}: {bit:#x}"
            );
        }
    }
}

#[test]
fn authorities_can_name_models_after_a_catalogue_hole() {
    for models in [Models::Named(FIRST_ADDED_BIT), Models::Named(LAST), Models::All] {
        assert_eq!(check_model_authorities(models, models, models), Ok(()), "{models:?}");
    }
}

fn fresh_model(number: u32, name: &[u8]) -> CatalogueModel {
    let mut model = catalogue_model(number, name, ModelSource::List);
    model.shown = None;
    model.description = None;
    model.picker = false;
    model.estimated = true;
    model.first_read = Some(READ_AT);
    model.model.efforts = Box::default();
    model
}

fn listed(name: &[u8]) -> ListedModel {
    ListedModel { name: name.into(), shown: None, window: None, output: None, efforts: Box::default() }
}

fn assert_list_case(case: &ListCase, list: &ProviderList, settings: &crate::Settings) {
    let change = from_list(case.provider, list, &case.rows, case.others, READ_AT, &case.models, &settings.endpoints);
    let mut expected = List::with_capacity(jig_domain_tasks::MODELS);
    for model in &case.expected {
        expected
            .push(Item {
                setting: Setting::Model(model.number),
                edit: Edit::Set(Value::Model(Box::new(model.clone()))),
            })
            .expect("table's expected edits fit the catalogue");
    }
    assert_eq!(change.items.as_ref(), expected.as_slice(), "{}: whole edits", case.label);
    assert_eq!(change.names, case.names, "{}: names", case.label);
    assert_eq!(change.left_out, case.left_out, "{}: left out", case.label);
    for item in &change.items {
        let model = match &item.edit {
            Edit::Set(value) => value.as_model().expect("list edits set models"),
            Edit::Removed | Edit::Reset => panic!("list edits keep whole values"),
        };
        assert_eq!(item.setting, Setting::Model(model.number), "{}: setting number", case.label);
        assert_eq!(
            shape(model, model.number, &[crate::tests::provider_setting(1, b"Fake", 1)]),
            Ok(()),
            "{}: shape",
            case.label
        );
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one table covers a read's additions, returns, withdrawals and metadata bounds"
)]
#[expect(clippy::disallowed_methods, reason = "ordinary bounded table fixtures (programming-model.md, 10.2)")]
fn catalogue_list_changes_preserve_whole_models_and_account_names() {
    let settings = crate::tests::config(1).settings;
    let list =
        ProviderList { keep: Box::new([b"keep-*".as_slice().into()]), drop: Box::new([b"*drop*".as_slice().into()]) };
    let stock = catalogue_model(1, b"keep-stock", ModelSource::Stock);
    let person = catalogue_model(3, b"keep-person", ModelSource::Person);
    let old = CatalogueModel {
        first_read: Some(Wall::EPOCH),
        estimated: true,
        ..catalogue_model(2, b"keep-old", ModelSource::List)
    };
    let gone = CatalogueModel { withdrawn: true, ..old.clone() };
    let other = CatalogueModel { model: crate::Model { provider: 2, ..old.model.clone() }, ..old.clone() };
    let first = fresh_model(64, b"keep-first");
    let second = fresh_model(63, b"keep-second");
    let third = fresh_model(62, b"keep-third");
    let first_row = listed(b"keep-first");
    let three_rows = [first_row.clone(), listed(b"keep-second"), listed(b"keep-third")];
    let full: Box<[_]> = (1..=jig_domain_tasks::MODELS)
        .map(|number| catalogue_model(number, number.to_string().as_bytes(), ModelSource::Stock))
        .collect();
    let almost_full = &full[..OCCUPIED];

    let figures = ListedModel {
        shown: Some(b"List's shown name".as_slice().into()),
        window: Some(4096),
        output: Some(512),
        efforts: Box::new([b"low".as_slice().into(), b"high".as_slice().into(), b"max".as_slice().into()]),
        ..first_row.clone()
    };
    let with_figures = CatalogueModel {
        shown: figures.shown.clone(),
        model: crate::Model {
            window: 4096,
            output: 512,
            efforts: figures.efforts.clone(),
            effort: Some(b"high".as_slice().into()),
            ..first.model.clone()
        },
        ..first.clone()
    };
    let first_effort = CatalogueModel {
        model: crate::Model {
            efforts: Box::new([b"medium".as_slice().into(), b"max".as_slice().into()]),
            effort: Some(b"medium".as_slice().into()),
            ..first.model.clone()
        },
        ..first.clone()
    };
    let shown_bound = usize::try_from(crate::MODEL_NAME_BYTES).expect("shown bound");
    let effort_bound = usize::try_from(crate::EFFORT_BYTES).expect("effort bound");
    let max_shown: Box<[u8]> = b"s".repeat(shown_bound).into();
    let max_effort: Box<[u8]> = b"e".repeat(effort_bound).into();
    let at_bounds = CatalogueModel {
        shown: Some(max_shown.clone()),
        model: crate::Model {
            efforts: Box::new([max_effort.clone()]),
            effort: Some(max_effort.clone()),
            ..first.model.clone()
        },
        ..first.clone()
    };
    let too_many: Box<[Box<[u8]>]> = (0..=crate::EFFORTS).map(|n| n.to_string().into_bytes().into()).collect();
    let bounded_efforts = CatalogueModel {
        model: crate::Model {
            efforts: too_many[..usize::try_from(crate::EFFORTS).expect("effort count bound")].into(),
            effort: Some(b"0".as_slice().into()),
            ..first.model.clone()
        },
        ..first.clone()
    };

    for case in [
        list_case("new", from_ref(&stock), from_ref(&first_row), 0, from_ref(&first), LAST, 0),
        list_case(
            "provider order",
            from_ref(&stock),
            &three_rows,
            0,
            &[first.clone(), second.clone(), third],
            LAST_THREE,
            0,
        ),
        list_case(
            "refused patterns and invalid names",
            from_ref(&stock),
            &[
                listed(b"other"),
                listed(b"keep-drop"),
                listed(b"keep-bad/name"),
                listed(b""),
                listed(&b"keep-".repeat(26)),
            ],
            0,
            &[],
            0,
            0,
        ),
        list_case(
            "stock and person",
            &[stock.clone(), person.clone()],
            &[
                ListedModel { shown: Some(b"ignored".as_slice().into()), ..listed(b"keep-stock") },
                listed(b"keep-person"),
            ],
            0,
            &[],
            5,
            0,
        ),
        list_case(
            "replace all old names",
            &[
                stock.clone(),
                old.clone(),
                CatalogueModel {
                    number: 3,
                    model: crate::Model { name: b"keep-old-second".as_slice().into(), ..old.model.clone() },
                    ..old.clone()
                },
            ],
            from_ref(&first_row),
            0,
            &[
                first.clone(),
                gone.clone(),
                CatalogueModel {
                    number: 3,
                    model: crate::Model { name: b"keep-old-second".as_slice().into(), ..gone.model.clone() },
                    ..gone.clone()
                },
            ],
            LAST,
            0,
        ),
        list_case("held elsewhere", &[stock.clone(), old.clone()], &[], 2, &[], 0, 0),
        list_case("withdraw whole value", &[stock.clone(), old.clone()], &[], 0, from_ref(&gone), 0, 0),
        list_case(
            "return keeps first read and figures",
            &[stock.clone(), gone.clone()],
            &[ListedModel { shown: Some(b"ignored".as_slice().into()), ..listed(b"keep-old") }],
            0,
            from_ref(&old),
            2,
            0,
        ),
        list_case(
            "withdrawn stock and person stay withdrawn",
            &[CatalogueModel { withdrawn: true, ..stock.clone() }, CatalogueModel { withdrawn: true, ..person }],
            &[listed(b"keep-stock"), listed(b"keep-person")],
            0,
            &[],
            0,
            0,
        ),
        list_case(
            "another provider owns the name",
            &[stock.clone(), other.clone()],
            &[listed(b"keep-old")],
            0,
            &[],
            0,
            0,
        ),
        list_case("another provider never withdrawn", &[stock.clone(), other], &[], 0, &[], 0, 0),
        list_case("unchanged list row", &[stock.clone(), old], &[listed(b"keep-old")], 0, &[], 2, 0),
        list_case("already withdrawn", &[stock.clone(), gone], &[], 0, &[], 0, 0),
        list_case("room for first two only", almost_full, &three_rows, 0, &[first.clone(), second], LAST_TWO, 1),
        list_case("full", &full, &three_rows, 0, &[], 0, 3),
        list_case("figures and high default", from_ref(&stock), from_ref(&figures), 0, &[with_figures], LAST, 0),
        list_case(
            "figures capped",
            from_ref(&stock),
            &[ListedModel { window: Some(u32::MAX), output: Some(u32::MAX), ..first_row.clone() }],
            0,
            from_ref(&first),
            LAST,
            0,
        ),
        list_case(
            "zero figures omitted",
            from_ref(&stock),
            &[ListedModel { window: Some(0), output: Some(0), ..first_row.clone() }],
            0,
            from_ref(&first),
            LAST,
            0,
        ),
        list_case(
            "first effort default",
            from_ref(&stock),
            &[ListedModel { efforts: first_effort.model.efforts.clone(), ..first_row.clone() }],
            0,
            &[first_effort],
            LAST,
            0,
        ),
        list_case(
            "metadata at byte bounds",
            from_ref(&stock),
            &[ListedModel { shown: Some(max_shown), efforts: Box::new([max_effort]), ..first_row.clone() }],
            0,
            &[at_bounds],
            LAST,
            0,
        ),
        list_case(
            "oversized metadata omitted",
            from_ref(&stock),
            &[ListedModel {
                shown: Some(b"s".repeat(shown_bound + 1).into()),
                efforts: Box::new([b"e".repeat(effort_bound + 1).into()]),
                ..first_row.clone()
            }],
            0,
            from_ref(&first),
            LAST,
            0,
        ),
        list_case(
            "empty metadata omitted",
            from_ref(&stock),
            &[ListedModel { shown: Some(Box::default()), efforts: Box::new([Box::default()]), ..first_row.clone() }],
            0,
            from_ref(&first),
            LAST,
            0,
        ),
        list_case(
            "efforts capped",
            from_ref(&stock),
            &[ListedModel { efforts: too_many, ..first_row.clone() }],
            0,
            &[bounded_efforts],
            LAST,
            0,
        ),
        list_case(
            "duplicate efforts omitted",
            from_ref(&stock),
            &[ListedModel {
                efforts: Box::new([b"high".as_slice().into(), b"high".as_slice().into()]),
                ..first_row.clone()
            }],
            0,
            &[CatalogueModel {
                model: crate::Model {
                    efforts: Box::new([b"high".as_slice().into()]),
                    effort: Some(b"high".as_slice().into()),
                    ..first.model.clone()
                },
                ..first.clone()
            }],
            LAST,
            0,
        ),
        list_case(
            "duplicate rows take one number",
            from_ref(&stock),
            &[first_row.clone(), first_row.clone()],
            0,
            from_ref(&first),
            LAST,
            0,
        ),
        ListCase { provider: 2, ..list_case("no own stock or endpoint", &[stock], from_ref(&first_row), 0, &[], 0, 1) },
        list_case("no stock anywhere", &[], &[first_row], 0, &[], 0, 1),
    ] {
        assert_list_case(&case, &list, &settings);
    }
    assert_eq!(NEW_FOR, Duration::from_secs(604_800));
}

#[test]
fn new_models_take_stock_figures_but_no_speeds() {
    let settings = crate::tests::config(1).settings;
    let mut stock = catalogue_model(1, b"stock", ModelSource::Stock);
    crate::tests::speed_example(&mut stock.model);
    let stock = [stock];
    let name = b"new-model";
    let added =
        settings::catalogue::new_model(1, name.as_slice().into(), ModelSource::Person, &stock, &settings.endpoints)
            .expect("free catalogue number");
    let list = ProviderList { keep: Box::new([b"*".as_slice().into()]), drop: Box::default() };
    let change = from_list(1, &list, &[listed(name)], 0, READ_AT, &stock, &settings.endpoints);
    let Edit::Set(value) = &change.items.first().expect("new listed model").edit else {
        panic!("list adds a model");
    };
    let listed = value.as_model().expect("listed model value");
    for model in [&added.model, &listed.model] {
        assert_eq!(model.input_price, stock[0].model.input_price);
        assert!(model.speeds.is_empty());
        assert_eq!(model.speed, None);
    }
}

#[test]
fn full_speed_declarations_fit_the_catalogue_and_count_in_a_charters_size() {
    let mut config = crate::tests::config(1);
    let policy = &mut config.defaults.charters[0].policy;
    let before = jig_domain_core_vocabulary::limits::policy_owned_bytes(policy).unwrap();
    let mut speeds = List::with_capacity(4);
    for (name, first, second) in [(b'a', b'b', b'c'), (b'd', b'e', b'f'), (b'g', b'h', b'i'), (b'j', b'k', b'l')] {
        speeds
            .push(crate::Speed {
                name: Box::new([name; 32]),
                also: Box::new([Box::new([first; 32]), Box::new([second; 32])]),
                input_price: 1,
                cached_price: 1,
                cache_write_price: Some(1),
                output_price: 1,
            })
            .unwrap();
    }
    let model = &mut policy.model;
    model.speeds = speeds.into_boxed();
    model.speed = Some(Box::new([b'a'; 32]));
    let row = CatalogueModel { model: model.clone(), ..catalogue_model(1, b"stock", ModelSource::Stock) };
    assert!(settings::catalogue::fits(&row, 1));
    // Each speed owns three names and two alias pointers. The default name is a separate allocation.
    let allocated = 4 * (size_of::<crate::Speed>() + 3 * 32 + 2 * size_of::<Box<[u8]>>()) + 32;
    // The route with four speeds has no program allocation.
    let carrier_without_speed_allocations = size_of::<CatalogueModel>()
        + 2 * usize::try_from(crate::MODEL_NAME_BYTES).unwrap()
        + usize::try_from(crate::MODEL_DESCRIPTION_BYTES).unwrap()
        + 16 * (size_of::<Box<[u8]>>() + 32)
        + 32;
    assert_eq!(
        settings::catalogue::bound().unwrap(),
        u64::try_from(carrier_without_speed_allocations + allocated).unwrap()
    );
    // The main model can also be a sub-agent: the run's worst case prices it twice.
    assert_eq!(
        jig_domain_core_vocabulary::limits::policy_owned_bytes(policy),
        Some(before + 2 * u64::try_from(allocated).unwrap())
    );
}

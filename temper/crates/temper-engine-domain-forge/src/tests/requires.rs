//! Procedure waits from covering landing rules (temper/docs/design/domain/forge/change/gates.md, 2).
use super::*;
use crate::settings;
use skein_lib::List;

fn template(repository: &Repository, last: GateLast, requires: Requires) -> settings::LandingRule {
    settings::LandingRule {
        sets: Box::new([]),
        pattern: GatePattern {
            segments: Box::new([
                Box::from(b"forge".as_slice()),
                repository.host.clone(),
                repository.owner.clone(),
                repository.name.clone(),
                Box::from(b"branch".as_slice()),
            ]),
            last,
        },
        gates: Box::new([]),
        enforce: false,
        ci: requires.ci,
        up_to_date: requires.up_to_date,
        approvals: Box::new([]),
    }
}

#[test]
fn requires_both_where_no_rule_covers_the_branch() {
    let d = domain();
    assert_eq!(d.requires(&adopted(), b"main", Touched::UNKNOWN), Requires { ci: true, up_to_date: true });
}

#[test]
fn requires_neither_under_a_rule_asking_neither() {
    let mut d = domain();
    let repository = adopted();
    let mut stock = List::uncapped();
    stock
        .push(stock_rule(
            0,
            template(
                &repository,
                GateLast::Open(Box::from(b"temper".as_slice())),
                Requires { ci: false, up_to_date: false },
            ),
        ))
        .unwrap();
    d.give_settings_defaults(stock.into_boxed());
    assert_eq!(d.requires(&repository, b"temper/7/g7", Touched::UNKNOWN), Requires { ci: false, up_to_date: false });
    assert_eq!(d.requires(&repository, b"main", Touched::UNKNOWN), Requires { ci: true, up_to_date: true });
}

#[test]
fn requires_the_union_of_the_covering_rules() {
    let mut d = domain();
    let repository = adopted();
    let ci =
        template(&repository, GateLast::Exact(Box::from(b"main".as_slice())), Requires { ci: true, up_to_date: false });
    let up_to_date =
        template(&repository, GateLast::Exact(Box::from(b"main".as_slice())), Requires { ci: false, up_to_date: true });
    let mut stock = List::uncapped();
    stock.push(stock_rule(0, ci.clone())).unwrap();
    stock.push(stock_rule(repository.project + 1, up_to_date.clone())).unwrap();
    d.give_settings_defaults(stock.into_boxed());
    assert_eq!(d.requires(&repository, b"main", Touched::UNKNOWN), Requires { ci: true, up_to_date: false });
    assert_eq!(d.requires(&repository, b"main/child", Touched::UNKNOWN), Requires { ci: true, up_to_date: true });

    let mut stock = List::uncapped();
    stock.push(stock_rule(0, ci)).unwrap();
    stock.push(stock_rule(repository.project, up_to_date.clone())).unwrap();
    stock.push(stock_rule(repository.project + 1, up_to_date)).unwrap();
    d.give_settings_defaults(stock.into_boxed());
    assert_eq!(d.requires(&repository, b"main", Touched::UNKNOWN), Requires { ci: true, up_to_date: true });
}

#[test]
fn requires_a_branch_with_no_name_keeps_the_waits() {
    let mut d = domain();
    let repository = adopted();
    let mut stock = List::uncapped();
    stock
        .push(stock_rule(
            0,
            template(&repository, GateLast::Open(Box::new([])), Requires { ci: false, up_to_date: false }),
        ))
        .unwrap();
    d.give_settings_defaults(stock.into_boxed());
    assert_eq!(d.requires(&repository, b"a/b", Touched::UNKNOWN), Requires { ci: false, up_to_date: false });
    for branch in [b"a//b".as_slice(), b"", b"/a", b"a/"] {
        assert_eq!(d.requires(&repository, branch, Touched::UNKNOWN), Requires { ci: true, up_to_date: true });
    }
}

fn stock_rule(scope: u32, rule: settings::LandingRule) -> settings::Stock {
    settings::Stock {
        kind: if scope == 0 { 1 } else { 2 },
        scope,
        number: 1,
        value: settings::SizedValue { size: 0, value: settings::Value::Rule(rule) },
    }
}

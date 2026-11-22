use hearth_builder_world::fixtures;
use hearth_test_render::golden;
use skein_scratch::paths;
use std::fs;

const SYNTAX: &[(&str, &str)] = &[
    ("condition", "{% if "),
    ("loop", "{% for word in "),
    ("loop with pairs", "{% for name, number in "),
    ("assignment", "{% set "),
    ("raw", "{% raw %}"),
    ("default", "| default("),
    ("join", "| join("),
    ("split", "| split("),
    ("first", "| first"),
    ("last", "| last"),
    ("length", "| length"),
    ("replace", "| replace("),
    ("items", "| items"),
    ("keys", "| keys"),
    ("values", "| values"),
    ("sort", "| sort"),
    ("tojson", "| tojson"),
    ("shell_quote", "| shell_quote"),
    ("systemd_escape", "| systemd_escape "),
    ("systemd_escape_path", "| systemd_escape_path"),
    ("basename", "| basename"),
    ("dirname", "| dirname"),
];
#[test]
fn templates_cover_every_statement_and_filter_format_md_names() {
    let root = paths::tree().join("tests/render/templates/tasks/syntax/tree/etc/syntax");
    let mut templates: Vec<_> = fs::read_dir(root)
        .expect("syntax templates")
        .map(|entry| entry.expect("syntax template").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "j2"))
        .collect();
    templates.sort();
    let source = templates.into_iter().map(|path| fs::read_to_string(path).expect("template text")).collect::<String>();
    for &(name, needle) in SYNTAX {
        assert!(source.contains(needle), "missing {name}");
    }
}

const X1_FIELDS: &[&str] = &[
    "target ",
    "placement ",
    "spec ",
    "packages ",
    "  sources ",
    "  request ",
    "  variant ",
    "  preseed    locales/default_environment_locale ",
    "  initramfs ",
    "config ",
    "  tmpfiles   /var/log/journal ",
    "  /etc/hostname ",
    "  /etc/localtime ",
    "data ",
    "  journal ",
];
const X1_RENDER_LINES: &str = concat!(
    "target       x1-prod   base x1   environment prod\n",
    "placement    metal efi 7th-gen\n",
    "spec         ............\n",
    "packages     part ............   resolution ............ needed\n",
    "  sources    forky amd64 main contrib non-free-firmware 20261001T030405Z\n",
    "  request    btrfs-progs ca-certificates dbus efibootmgr firmware-intel-sound iwd linux-image-amd64 locales sway tiny-initramfs tzdata\n",
    "  variant    apt\n",
    "  preseed    locales/default_environment_locale select en_US.UTF-8\n",
    "  initramfs  btrfs nvme\n",
    "config       part ............\n",
    "  tmpfiles   /var/log/journal d 2755 root systemd-journal\n",
    "  /etc/hostname                file 0644 root:root ............\n",
    "  /etc/localtime               link root:root /usr/share/zoneinfo/Etc/UTC\n",
    "data         part ............\n",
    "  journal    /var/log/journal\n",
);
#[test]
fn x1_prod_prints_render_md_9_2s_lines() {
    let text = include_bytes!("../../../render/fleet/expected/x1-prod");
    assert_eq!(golden::mask12(&fixtures::lines_starting(text, X1_FIELDS)), X1_RENDER_LINES);
}

#[test]
fn the_exports_golden_holds_hearth_20s_vpn_files() {
    let root = paths::tree().join("tests/render/exports");
    for file in fixtures::VPN {
        let bytes = fs::read(root.join(file.path)).expect(file.path);
        assert_eq!(bytes, file.body, "{}", file.path);
    }
}

#[test]
fn the_tool_and_resolution_errors_print_through_the_protocol_world() {
    for &(name, call) in fixtures::PROTOCOL_ERRORS {
        fixtures::check_protocol_golden(name, call);
    }
}

#[test]
fn errors_are_sorted_by_path_line_column_and_target() {
    let text = include_bytes!("../../../render/errors/sorted/expected/output");
    assert_eq!(
        fixtures::lines_starting(text, &["tasks/"]),
        concat!(
            "tasks/another/task.json:2:15: bad-prod: another: requires a: first file\n",
            "tasks/broken/task.json:3:3: bad-prod: broken: requires a: first guard\n",
            "tasks/broken/task.json:3:3: bad2-prod: broken: requires a: first guard\n",
            "tasks/broken/task.json:3:48: bad-prod: broken: requires b: second guard\n",
            "tasks/broken/task.json:3:48: bad2-prod: broken: requires b: second guard\n",
        )
    );
}

#[test]
fn every_kind_of_7_2_but_checks_and_store_has_a_case() {
    let root = skein_scratch::paths::tree().join("tests/render/errors");
    let mut names: Vec<_> = std::fs::read_dir(root)
        .expect("error golden directory")
        .map(|entry| entry.expect("error golden").file_name().into_string().expect("kind name"))
        .filter(|name| name != "sorted")
        .collect();
    names.sort();
    assert_eq!(names, fixtures::ERROR_KINDS);
}

#[test]
fn the_keys_line_is_the_one_hearth_24_pins() {
    let text = include_str!("../../../render/resolution/expected-known/test-min");
    let line = text.lines().find(|line| line.starts_with("keys ")).expect("known keys");
    assert_eq!(line, fixtures::TEST_MIN_KEYS_LINE);
}

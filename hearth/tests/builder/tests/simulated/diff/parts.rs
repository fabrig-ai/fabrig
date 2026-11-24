use hearth::Form;
use hearth_builder_world::{
    fixtures::{self, File, SIMULATION_SEED, Scenario, diffing, edited, lines_starting, part_changes},
    protocol,
};

struct Change {
    part: &'static str,
    before: &'static [File],
    after: File,
    prefixes: &'static [&'static str],
    text: &'static str,
    rebuild: &'static str,
}

const CHANGES: &[Change] = &[
    Change {
        part: "software",
        before: fixtures::software::SOFTWARE,
        after: part_changes::SOFTWARE_MOVED,
        prefixes: &["  software "],
        text: "  software ~ /opt/ripgrep                           tar 1 root:root d2571cab3997\n",
        rebuild: r#""rebuild":{"kind":"from_software"}"#,
    },
    Change {
        part: "secrets",
        before: core::slice::from_ref(&fixtures::SECRETS),
        after: part_changes::SECRETS_MOVED,
        prefixes: &["  secrets "],
        text: "  secrets  ~ secret mail/tls/mx1.key\n  secrets  ~ password alice\n",
        rebuild: r#""rebuild":{"kind":"slot_only"}"#,
    },
    Change {
        part: "live",
        before: core::slice::from_ref(&fixtures::LIVE),
        after: part_changes::LIVE_MOVED,
        prefixes: &["  live "],
        text: "  live     + /etc/nginx/                            motd nginx.service reload\n",
        rebuild: r#""rebuild":{"kind":"nothing"}"#,
    },
    Change {
        part: "seeds",
        before: core::slice::from_ref(&fixtures::ids::SECRETS_FIXED),
        after: part_changes::SECRETS_FIXED_MOVED,
        prefixes: &["x1-prod", "  packages "],
        text: concat!(
            "x1-prod                                             rebuild from packages, resolution needed\n",
            "  packages   resolution same\n",
            "  packages ~ seed group opendkim                    121\n"
        ),
        rebuild: r#""rebuild":{"kind":"from_packages","resolution":"needed"}"#,
    },
];

fn changed(change: &Change, form: Form) -> protocol::Run {
    let files = fixtures::thin_with(change.before);
    let later = edited(&files, &[change.after], &[]);
    protocol::run(&Scenario { form, ..diffing(&files, &later) }, SIMULATION_SEED)
}

#[test]
fn changed_parts_report_their_rebuild_kind_in_json() {
    for change in CHANGES {
        let run = changed(change, Form::JsonLines);
        let stdout = std::str::from_utf8(&run.stdout).expect("JSON");
        assert!(stdout.contains(change.rebuild), "{}: {stdout}", change.part);
    }
}

#[test]
fn each_changed_part_prints_its_delta_and_exits_1() {
    for change in CHANGES {
        let run = changed(change, Form::Text);
        assert_eq!(lines_starting(&run.stdout, change.prefixes), change.text, "{}", change.part);
        assert_eq!(run.code, Some(1), "{}", change.part);
    }
}

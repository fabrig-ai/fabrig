use hearth::{Call, Form};
use hearth_builder_world::{
    fixtures::{self, File, Laid, SIMULATION_SEED, Scenario, TEST_REPOSITORY, diffing, edited},
    protocol,
};
use hearth_lib::encode::{Digest, Item};
use hearth_protocol_tree::resolution::Package;

const ENABLE_EXTRA: &[File] = &[
    File { path: "tasks/extra/task.json", body: br#"{"packages":{"vim":{}}}"#, mode: 0o644 },
    File { path: "hosts/test/test-min.json", body: br#"{"extra":{}}"#, mode: 0o644 },
];
const VIM_KEY: Digest = Digest([
    0xfe, 0x02, 0x39, 0xc3, 0x52, 0x51, 0x49, 0xdb, 0xf1, 0x9e, 0x27, 0x76, 0x80, 0x30, 0xd8, 0xe6, 0x1b, 0x45, 0xe7,
    0x7a, 0x04, 0x57, 0x8c, 0xe1, 0x49, 0xe2, 0xea, 0x86, 0xcc, 0x75, 0x72, 0x42,
]);
const BEFORE_CLOSURE: Digest = Digest([
    0xdc, 0xc9, 0xcb, 0x83, 0x25, 0xeb, 0xe2, 0x7a, 0x4e, 0x87, 0xcd, 0xf6, 0xe8, 0x49, 0x1b, 0x04, 0x42, 0x50, 0x15,
    0xb9, 0xc6, 0x64, 0x19, 0x34, 0xc4, 0x3f, 0xfa, 0xb0, 0x85, 0xd9, 0x7b, 0xad,
]);
const AFTER_CLOSURE: Digest = Digest([
    0x84, 0x59, 0x33, 0x5e, 0xbe, 0xb4, 0x41, 0xd4, 0x15, 0xb6, 0xe4, 0x5b, 0xc8, 0x4a, 0x42, 0x3c, 0x4e, 0x18, 0x7c,
    0x2a, 0xfa, 0xff, 0x82, 0x8f, 0x51, 0x74, 0x72, 0x89, 0x76, 0x49, 0x25, 0xcb,
]);

fn package(name: &[u8], version: &[u8]) -> Package {
    Package {
        name: name.into(),
        architecture: b"amd64".as_slice().into(),
        version: version.into(),
        source: 0,
        filename: b"pool/a.deb".as_slice().into(),
        size: 1,
        sha256: Digest([1; 32]),
    }
}

fn recorded_resolution_diff(form: Form) -> protocol::Run {
    let mut before = fixtures::test_min_recorded();
    before.closure = Box::new([package(b"apt", b"2"), package(b"nano", b"8")]);
    before.digest = BEFORE_CLOSURE;
    let mut after = before.clone();
    after.key = VIM_KEY;
    after.inputs.request = Box::new([Item::List(1), Item::Bytes(b"vim".as_slice().into())]);
    after.closure = Box::new([package(b"apt", b"3"), package(b"vim", b"9")]);
    after.digest = AFTER_CLOSURE;
    let laid = [Laid::Resolution(before), Laid::Resolution(after)];
    let later = edited(TEST_REPOSITORY, ENABLE_EXTRA, &[]);
    let mut scenario = Scenario { form, laid: &laid, ..diffing(TEST_REPOSITORY, &later) };
    let Call::Diff { targets, .. } = &mut scenario.call else { unreachable!("diffing supplies a diff call") };
    *targets = Box::new([b"test-min".as_slice().into()]);
    protocol::run(&scenario, SIMULATION_SEED)
}

#[test]
fn two_recorded_resolutions_show_the_packages_a_change_moves() {
    let run = recorded_resolution_diff(Form::JsonLines);
    assert_eq!(
        fixtures::lines_starting(&run.stdout, &["{\"record\":\"change\""]),
        concat!(
            r#"{"record":"change","target":"test-min","kind":"changed","changed":{"parts":[{"kind":"packages","packages":{"request":{"added":["vim"],"removed":[]},"seeds":{"groups":[],"users":[]},"resolution":{"added":[{"architecture":"amd64","name":"vim","version":"9"}],"kind":"resolved","removed":[{"architecture":"amd64","name":"nano","version":"8"}],"versions_changed":[{"after":"3","architecture":"amd64","before":"2","name":"apt"}]}}}],"rebuild":{"kind":"from_packages","resolution":"known"}}}"#,
            "\n"
        )
    );
    assert_eq!(run.code, Some(1));
}
#[test]
fn recorded_resolution_changes_print_each_package_in_text() {
    let run = recorded_resolution_diff(Form::Text);
    assert_eq!(
        fixtures::lines_starting(&run.stdout, &["  packages", "    "]),
        concat!(
            "  packages   resolution new key, resolved\n",
            "    + vim:amd64 9\n",
            "    - nano:amd64 8\n",
            "    ~ apt:amd64 2 → 3\n",
            "  packages + vim\n"
        )
    );
}

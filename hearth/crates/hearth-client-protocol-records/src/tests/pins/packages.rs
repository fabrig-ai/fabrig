use super::*;
use crate::{diff::ResolutionChange, parts::packages::*, records::Resolution};
use skein_lib::bytes::copy_of;

#[test]
fn packages_needed_and_container_forms() {
    let mut rich = fixtures::packages_record();
    rich.initramfs_file = Some([7; 32]);
    rich.initramfs =
        Some(Initramfs { modules: Box::new([copy_of(b"btrfs"), copy_of(b"nvme")]), microcode: Microcode::None });
    rich.apt.install_recommends = true;
    rich.preseeds = Box::new([
        Preseed {
            question: copy_of(b"a/answer"),
            owner: copy_of(b"a"),
            kind: PreseedKind::String,
            value: copy_of(b"word"),
        },
        Preseed {
            question: copy_of(b"a/enabled"),
            owner: copy_of(b"a"),
            kind: PreseedKind::Boolean,
            value: copy_of(b"true"),
        },
        Preseed {
            question: copy_of(b"a/options"),
            owner: copy_of(b"a"),
            kind: PreseedKind::Multiselect,
            value: copy_of(b"one, two"),
        },
        Preseed {
            question: copy_of(b"tzdata/Areas"),
            owner: copy_of(b"other"),
            kind: PreseedKind::Select,
            value: copy_of(b"Europe"),
        },
    ]);
    rich.seeds = Seeds {
        users: Box::new([
            SeedUser { name: copy_of(b"mail"), uid: 120, gid: Some(121) },
            SeedUser { name: copy_of(b"postfix"), uid: 122, gid: None },
        ]),
        groups: Box::new([SeedGroup { name: copy_of(b"mail"), gid: 121 }]),
    };
    let mut rich_spec = fixtures::with_resolution(fixtures::x1_prod(), Resolution::Needed { key: [3; 32] });
    rich_spec.parts = Box::new([Part { digest: [0; 32], record: PartRecord::Packages(rich) }]);
    let mut container = fixtures::packages_record();
    container.variant = Variant::Container;
    container.initramfs = None;
    container.apt.install_recommends = true;
    container.sources.security = None;
    container.sources.updates = false;
    check(&[
        (
            Record::Spec(rich_spec),
            spec_text!(
                "packages     part 000000000000   resolution 030303030303 needed\n",
                "  sources    forky amd64 main contrib non-free non-free-firmware 20261001T030405Z updates security 20261002T030405Z\n",
                "  request    curl tzdata\n  variant    apt\n",
                "  preseed    a/answer string word\n  preseed    a/enabled boolean true\n  preseed    a/options multiselect one, two\n  preseed    tzdata/Areas select Europe owner other\n",
                "  initramfs  btrfs nvme\n  microcode  none\n  apt        install_recommends true\n",
                "  seed       user mail 120 121\n  seed       user postfix 122 -\n  seed       group mail 121\n"
            ),
            concat!(
                r#"{"record":"version","client":1}"#,
                "\n",
                r#"{"record":"spec","target":"x1-prod","commit":"1a2b3c4d00000000000000000000000000000000","digest":"9a0c5e1f2b3d0000000000000000000000000000000000000000000000000000","resolution":{"key":"0303030303030303030303030303030303030303030303030303030303030303","kind":"needed"}}"#,
                "\n",
                r#"{"record":"part","target":"x1-prod","name":"meta","digest":"c4d3a0b1e2f50000000000000000000000000000000000000000000000000000","meta":{"base":"x1","environment":"prod","placement":{"firmware":"efi","hardware":"7th-gen","kind":"metal"},"target":"x1-prod"}}"#,
                "\n",
                r#"{"record":"part","target":"x1-prod","name":"packages","digest":"0000000000000000000000000000000000000000000000000000000000000000","initramfs_file":"0707070707070707070707070707070707070707070707070707070707070707","packages":{"apt":{"install_recommends":true},"initramfs":{"microcode":"none","modules":["btrfs","nvme"]},"preseeds":{"a/answer":{"owner":"a","type":"string","value":"word"},"a/enabled":{"owner":"a","type":"boolean","value":"true"},"a/options":{"owner":"a","type":"multiselect","value":"one, two"},"tzdata/Areas":{"owner":"other","type":"select","value":"Europe"}},"request":["curl","tzdata"],"seeds":{"groups":{"mail":{"gid":121}},"users":{"mail":{"gid":121,"uid":120},"postfix":{"gid":null,"uid":122}}},"sources":{"architecture":"amd64","archive":"20261001T030405Z","components":["main","contrib","non-free","non-free-firmware"],"security":"20261002T030405Z","suite":"forky","updates":true},"variant":"apt"}}"#,
                "\n"
            ),
        ),
        (
            spec(PartRecord::Packages(container)),
            spec_text!(
                "packages     part 000000000000\n",
                "  sources    forky amd64 main contrib non-free non-free-firmware 20261001T030405Z\n",
                "  request    curl tzdata\n  variant    container\n  apt        install_recommends true\n"
            ),
            spec_json!(
                r#"{"record":"part","target":"x1-prod","name":"packages","digest":"0000000000000000000000000000000000000000000000000000000000000000","packages":{"apt":{"install_recommends":true},"preseeds":{},"request":["curl","tzdata"],"seeds":{"groups":{},"users":{}},"sources":{"architecture":"amd64","archive":"20261001T030405Z","components":["main","contrib","non-free","non-free-firmware"],"suite":"forky","updates":false},"variant":"container"}}"#,
                "\n"
            ),
        ),
    ]);
}
fn package() -> Change {
    let PartChange::Packages(packages) = fixtures::unchanged_packages_change() else { panic!("packages") };
    packages
}
macro_rules! pkg_json {
    ($($body:literal),* $(,)?) => { change_json!(r#"{"kind":"packages","packages":{"#, $($body),*, "}}") };
}
#[test]
fn packages_pairs_and_new_resolution_key() {
    let mut sources = package();
    let before = fixtures::packages_record().sources;
    let mut after = before.clone();
    after.updates = false;
    after.security = None;
    sources.sources = Some(SourcesPair { before, after });
    let mut variant = package();
    variant.variant = Some(VariantPair { before: Variant::Apt, after: Variant::Container });
    let mut apt = package();
    apt.apt = Some(AptPair { before: Apt { install_recommends: false }, after: Apt { install_recommends: true } });
    let mut resolution = package();
    resolution.resolution = ResolutionChange::NewKey;
    check(&[
        (
            change(PartChange::Packages(sources)),
            change_text!(
                "  packages   resolution same\n",
                "  packages ~ sources {\"architecture\":\"amd64\",\"archive\":\"20261001T030405Z\",\"components\":[\"main\",\"contrib\",\"non-free\",\"non-free-firmware\"],\"security\":\"20261002T030405Z\",\"suite\":\"forky\",\"updates\":true} → {\"architecture\":\"amd64\",\"archive\":\"20261001T030405Z\",\"components\":[\"main\",\"contrib\",\"non-free\",\"non-free-firmware\"],\"suite\":\"forky\",\"updates\":false}\n"
            ),
            pkg_json!(
                r#""seeds":{"groups":[],"users":[]},"resolution":{"kind":"same"},"sources":{"after":{"architecture":"amd64","archive":"20261001T030405Z","components":["main","contrib","non-free","non-free-firmware"],"suite":"forky","updates":false},"before":{"architecture":"amd64","archive":"20261001T030405Z","components":["main","contrib","non-free","non-free-firmware"],"security":"20261002T030405Z","suite":"forky","updates":true}}"#
            ),
        ),
        (
            change(PartChange::Packages(variant)),
            change_text!("  packages   resolution same\n  packages ~ variant \"apt\" → \"container\"\n"),
            pkg_json!(
                r#""seeds":{"groups":[],"users":[]},"resolution":{"kind":"same"},"variant":{"after":"container","before":"apt"}"#
            ),
        ),
        (
            change(PartChange::Packages(apt)),
            change_text!(
                "  packages   resolution same\n  packages ~ apt {\"install_recommends\":false} → {\"install_recommends\":true}\n"
            ),
            pkg_json!(
                r#""apt":{"after":{"install_recommends":true},"before":{"install_recommends":false}},"seeds":{"groups":[],"users":[]},"resolution":{"kind":"same"}"#
            ),
        ),
        (
            change(PartChange::Packages(resolution)),
            change_text!("  packages   resolution new key\n"),
            pkg_json!(r#""seeds":{"groups":[],"users":[]},"resolution":{"kind":"new_key"}"#),
        ),
        (
            change(fixtures::preseed_change()),
            change_text!(
                "  packages   resolution same\n",
                r#"  packages ~ preseeds {} → {"a/answer":{"owner":"a","type":"string","value":"word"},"tzdata/Areas":{"owner":"other","type":"select","value":"Europe"}}"#,
                "\n"
            ),
            pkg_json!(
                r#""preseeds":{"after":{"a/answer":{"owner":"a","type":"string","value":"word"},"tzdata/Areas":{"owner":"other","type":"select","value":"Europe"}},"before":{}},"seeds":{"groups":[],"users":[]},"resolution":{"kind":"same"}"#
            ),
        ),
        (
            change(fixtures::initramfs_change()),
            change_text!(
                "  packages   resolution same\n",
                r#"  packages ~ initramfs null → {"microcode":"generic","modules":["nvme"]}"#,
                "\n"
            ),
            pkg_json!(
                r#""initramfs":{"after":{"microcode":"generic","modules":["nvme"]},"before":null},"seeds":{"groups":[],"users":[]},"resolution":{"kind":"same"}"#
            ),
        ),
    ]);
}

#[test]
fn seed_keyed_change_forms() {
    let mut added = package();
    added.seeds.users =
        Box::new([SeedUserKeyed::Added { key: copy_of(b"mail"), value: SeedIds { uid: 120, gid: None } }]);
    added.seeds.groups = Box::new([SeedGroupKeyed::Added { key: copy_of(b"mail"), value: 121 }]);
    let mut removed = package();
    removed.seeds.users =
        Box::new([SeedUserKeyed::Removed { key: copy_of(b"mail"), value: SeedIds { uid: 120, gid: Some(121) } }]);
    removed.seeds.groups = Box::new([SeedGroupKeyed::Removed { key: copy_of(b"mail"), value: 121 }]);
    let mut changed = package();
    changed.seeds.users = Box::new([SeedUserKeyed::Changed {
        key: copy_of(b"mail"),
        before: SeedIds { uid: 120, gid: Some(121) },
        after: SeedIds { uid: 122, gid: None },
    }]);
    check(&[
        (
            change(PartChange::Packages(added)),
            change_text!(
                "  packages   resolution same\n  packages + seed user mail                         120 -\n  packages + seed group mail                        121\n"
            ),
            pkg_json!(
                r#""seeds":{"groups":[{"kind":"added","value":121,"key":"mail"}],"users":[{"kind":"added","value":{"gid":null,"uid":120},"key":"mail"}]},"resolution":{"kind":"same"}"#
            ),
        ),
        (
            change(PartChange::Packages(removed)),
            change_text!(
                "  packages   resolution same\n  packages - seed user mail                         120 121\n  packages - seed group mail                        121\n"
            ),
            pkg_json!(
                r#""seeds":{"groups":[{"kind":"removed","value":121,"key":"mail"}],"users":[{"kind":"removed","value":{"gid":121,"uid":120},"key":"mail"}]},"resolution":{"kind":"same"}"#
            ),
        ),
        (
            change(PartChange::Packages(changed)),
            change_text!("  packages   resolution same\n  packages ~ seed user mail                         122 -\n"),
            pkg_json!(
                r#""seeds":{"groups":[],"users":[{"after":{"gid":null,"uid":122},"before":{"gid":121,"uid":120},"kind":"changed","key":"mail"}]},"resolution":{"kind":"same"}"#
            ),
        ),
    ]);
}

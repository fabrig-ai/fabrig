use super::assert_prints;
use crate::tests::fixtures;
use hearth_domain_render_vocabulary as render;
use render::errors::Rule;

#[test]
fn reserved_bytes() {
    let cases = [
        (
            fixtures::reserved::error(Rule::ActivatorKey),
            "\"rootwait\" is reserved: the activator owns this command-line key (domain/machine/boot.md, 3.2)\n",
            concat!(
                r#"{"record":"error","kind":"reserved","reserved":{"rule":"activator_key","what":"rootwait"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            fixtures::reserved::error(Rule::RootKey),
            "\"rootwait\" is reserved: it changes how tiny-initramfs finds or starts the root (domain/machine/boot.md, 3.2)\n",
            concat!(
                r#"{"record":"error","kind":"reserved","reserved":{"rule":"root_key","what":"rootwait"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            fixtures::reserved::error(Rule::PasswordPreseed),
            "\"rootwait\" is reserved: a password preseed leaves its value in /var/cache/debconf/passwords.dat (protocol/format/tasks.md, 11)\n",
            concat!(
                r#"{"record":"error","kind":"reserved","reserved":{"rule":"password_preseed","what":"rootwait"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            fixtures::reserved::error(Rule::TrialName),
            "\"rootwait\" is reserved: hearth owns trial and check units (domain/slots/prepare.md, 5)\n",
            concat!(
                r#"{"record":"error","kind":"reserved","reserved":{"rule":"trial_name","what":"rootwait"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            fixtures::reserved::error(Rule::Overlay),
            "\"rootwait\" is reserved: extensions would merge over hearth's live overlays (protocol/format/tasks.md, 11)\n",
            concat!(
                r#"{"record":"error","kind":"reserved","reserved":{"rule":"overlay","what":"rootwait"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            fixtures::reserved::error(Rule::TmpfilesSubvolume),
            "\"rootwait\" is reserved: a nested subvolume breaks a layer (protocol/format/tasks.md, 11)\n",
            concat!(
                r#"{"record":"error","kind":"reserved","reserved":{"rule":"tmpfiles_subvolume","what":"rootwait"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            fixtures::reserved::error(Rule::TmpfilesWrite),
            "\"rootwait\" is reserved: a tmpfiles write outside /var, /run or /tmp would fail on the read-only root (protocol/format/tasks.md, 11)\n",
            concat!(
                r#"{"record":"error","kind":"reserved","reserved":{"rule":"tmpfiles_write","what":"rootwait"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            fixtures::reserved::error(Rule::TmpfilesPath),
            "\"rootwait\" is reserved: a tmpfiles path holds no glob or specifier (protocol/format/tasks.md, 11)\n",
            concat!(
                r#"{"record":"error","kind":"reserved","reserved":{"rule":"tmpfiles_path","what":"rootwait"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            fixtures::reserved::error(Rule::ContainerModule),
            "\"rootwait\" is reserved: a container has no initramfs module (domain/render/spec.md, 1.2)\n",
            concat!(
                r#"{"record":"error","kind":"reserved","reserved":{"rule":"container_module","what":"rootwait"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            fixtures::reserved::error(Rule::ContainerParam),
            "\"rootwait\" is reserved: a container has no boot parameter (domain/render/spec.md, 1.2)\n",
            concat!(
                r#"{"record":"error","kind":"reserved","reserved":{"rule":"container_param","what":"rootwait"},"class":"configuration"}"#,
                "\n"
            ),
        ),
    ];
    for (error, text, json) in cases {
        assert_prints(&error, text, json);
    }
}

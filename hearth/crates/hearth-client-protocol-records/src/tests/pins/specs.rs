use super::*;
use crate::parts::{boot::Boot, data::*, exports::*, live, meta::*, secrets};
use skein_lib::bytes::copy_of;

#[test]
fn meta_placements_and_members() {
    let mut vm = fixtures::x1_prod();
    vm.parts = Box::new([]);
    vm.meta.member = Some(copy_of(b"a"));
    vm.meta.placement = Placement {
        kind: PlacementKind::Vm,
        firmware: Some(Firmware::Bios),
        hardware: None,
        on: Some(copy_of(b"host-prod")),
    };
    let mut container = vm.clone();
    container.meta.placement =
        Placement { kind: PlacementKind::Container, firmware: None, hardware: None, on: Some(copy_of(b"host-prod")) };
    check(&[
        (
            Record::Spec(vm),
            concat!(
                "target       x1-prod   base x1   member a   environment prod\n",
                "placement    vm bios on host-prod\nspec         9a0c5e1f2b3d\n"
            ),
            concat!(
                r#"{"record":"version","client":1}"#,
                "\n",
                r#"{"record":"spec","target":"x1-prod","commit":"1a2b3c4d00000000000000000000000000000000","digest":"9a0c5e1f2b3d0000000000000000000000000000000000000000000000000000"}"#,
                "\n",
                r#"{"record":"part","target":"x1-prod","name":"meta","digest":"c4d3a0b1e2f50000000000000000000000000000000000000000000000000000","meta":{"base":"x1","environment":"prod","member":"a","placement":{"firmware":"bios","kind":"vm","on":"host-prod"},"target":"x1-prod"}}"#,
                "\n"
            ),
        ),
        (
            Record::Spec(container),
            concat!(
                "target       x1-prod   base x1   member a   environment prod\n",
                "placement    container on host-prod\nspec         9a0c5e1f2b3d\n"
            ),
            concat!(
                r#"{"record":"version","client":1}"#,
                "\n",
                r#"{"record":"spec","target":"x1-prod","commit":"1a2b3c4d00000000000000000000000000000000","digest":"9a0c5e1f2b3d0000000000000000000000000000000000000000000000000000"}"#,
                "\n",
                r#"{"record":"part","target":"x1-prod","name":"meta","digest":"c4d3a0b1e2f50000000000000000000000000000000000000000000000000000","meta":{"base":"x1","environment":"prod","member":"a","placement":{"kind":"container","on":"host-prod"},"target":"x1-prod"}}"#,
                "\n"
            ),
        ),
    ]);
}

#[test]
fn immutable_boot_and_data_classes() {
    check(&[
        (
            spec(PartRecord::Boot(Boot { mutable: false, params: Box::new([]) })),
            spec_text!("boot         part 000000000000\n", "  mutable    false\n"),
            spec_json!(
                r#"{"record":"part","target":"x1-prod","name":"boot","digest":"0000000000000000000000000000000000000000000000000000000000000000","boot":{"mutable":false,"params":[]}}"#,
                "\n"
            ),
        ),
        (
            spec(PartRecord::Data(Data {
                entries: Box::new([DataEntry {
                    name: copy_of(b"mail"),
                    value: fixtures::data::value(DataClass::Data),
                }]),
            })),
            spec_text!("data         part 000000000000\n  mail       /var/lib/iwd\n"),
            spec_json!(
                r#"{"record":"part","target":"x1-prod","name":"data","digest":"0000000000000000000000000000000000000000000000000000000000000000","data":{"entries":{"mail":{"class":{"kind":"data"},"path":"/var/lib/iwd"}}}}"#,
                "\n"
            ),
        ),
        (
            spec(PartRecord::Data(Data {
                entries: Box::new([DataEntry {
                    name: copy_of(b"mail"),
                    value: fixtures::data::value(DataClass::Cache),
                }]),
            })),
            spec_text!("data         part 000000000000\n  mail       /var/lib/iwd\n"),
            spec_json!(
                r#"{"record":"part","target":"x1-prod","name":"data","digest":"0000000000000000000000000000000000000000000000000000000000000000","data":{"entries":{"mail":{"class":{"kind":"cache"},"path":"/var/lib/iwd"}}}}"#,
                "\n"
            ),
        ),
    ]);
}

fn live_spec(action: live::Action) -> Record {
    spec(PartRecord::Live(live::Live {
        paths: Box::new([live::Covered { path: copy_of(b"/etc/mailname"), consumers: consumers(action) }]),
    }))
}
pub(super) fn consumers(action: live::Action) -> Box<[live::Consumer]> {
    Box::new([live::Consumer { task: copy_of(b"mail"), unit: Some(copy_of(b"mail.service")), action }])
}
#[test]
fn live_actions_with_units() {
    check(&[
        (
            live_spec(live::Action::Reload),
            spec_text!("live         part 000000000000\n  path       /etc/mailname mail mail.service reload\n"),
            spec_json!(
                r#"{"record":"part","target":"x1-prod","name":"live","digest":"0000000000000000000000000000000000000000000000000000000000000000","live":{"/etc/mailname":[{"action":"reload","task":"mail","unit":"mail.service"}]}}"#,
                "\n"
            ),
        ),
        (
            live_spec(live::Action::Restart),
            spec_text!("live         part 000000000000\n  path       /etc/mailname mail mail.service restart\n"),
            spec_json!(
                r#"{"record":"part","target":"x1-prod","name":"live","digest":"0000000000000000000000000000000000000000000000000000000000000000","live":{"/etc/mailname":[{"action":"restart","task":"mail","unit":"mail.service"}]}}"#,
                "\n"
            ),
        ),
        (
            live_spec(live::Action::ReloadOrRestart),
            spec_text!(
                "live         part 000000000000\n  path       /etc/mailname mail mail.service reload-or-restart\n"
            ),
            spec_json!(
                r#"{"record":"part","target":"x1-prod","name":"live","digest":"0000000000000000000000000000000000000000000000000000000000000000","live":{"/etc/mailname":[{"action":"reload-or-restart","task":"mail","unit":"mail.service"}]}}"#,
                "\n"
            ),
        ),
        (
            live_spec(live::Action::Start),
            spec_text!("live         part 000000000000\n  path       /etc/mailname mail mail.service start\n"),
            spec_json!(
                r#"{"record":"part","target":"x1-prod","name":"live","digest":"0000000000000000000000000000000000000000000000000000000000000000","live":{"/etc/mailname":[{"action":"start","task":"mail","unit":"mail.service"}]}}"#,
                "\n"
            ),
        ),
    ]);
}
fn secret(algorithm: secrets::Algorithm) -> Record {
    let mut secret = fixtures::secret_record(algorithm);
    secret.files = Box::new([]);
    secret.passwords = Box::new([]);
    fixtures::only_mut(&mut secret.identity).public_path = None;
    spec(PartRecord::Secrets(secret))
}
#[test]
fn identity_algorithms_without_public_paths() {
    check(&[
        (
            secret(secrets::Algorithm::Ecdsa),
            spec_text!(
                "secrets      part 000000000000\n  identity   ssh ssh-host-key ecdsa /etc/ssh/key - 0600 root:root\n"
            ),
            spec_json!(
                r#"{"record":"part","target":"x1-prod","name":"secrets","digest":"0000000000000000000000000000000000000000000000000000000000000000","secrets":{"files":{},"identity":{"ssh":{"algorithm":"ecdsa","group":"root","kind":"ssh-host-key","mode":"0600","owner":"root","path":"/etc/ssh/key"}},"passwords":{}}}"#,
                "\n"
            ),
        ),
        (
            secret(secrets::Algorithm::Rsa),
            spec_text!(
                "secrets      part 000000000000\n  identity   ssh ssh-host-key rsa /etc/ssh/key - 0600 root:root\n"
            ),
            spec_json!(
                r#"{"record":"part","target":"x1-prod","name":"secrets","digest":"0000000000000000000000000000000000000000000000000000000000000000","secrets":{"files":{},"identity":{"ssh":{"algorithm":"rsa","group":"root","kind":"ssh-host-key","mode":"0600","owner":"root","path":"/etc/ssh/key"}},"passwords":{}}}"#,
                "\n"
            ),
        ),
    ]);
}
fn export(value: Value) -> Record {
    Record::Spec(Box::new(fixtures::with_exports(*fixtures::x1_prod(), b"vpn", b"endpoint", value)))
}
#[test]
fn export_value_forms() {
    check(&[
        (
            export(Value::Bool(true)),
            spec_text!("exports      part 000000000000\n  vpn.endpoint   true\n"),
            spec_json!(
                r#"{"record":"part","target":"x1-prod","name":"exports","digest":"0000000000000000000000000000000000000000000000000000000000000000","exports":{"vpn":{"endpoint":true}}}"#,
                "\n"
            ),
        ),
        (
            export(Value::Int(-7)),
            spec_text!("exports      part 000000000000\n  vpn.endpoint   -7\n"),
            spec_json!(
                r#"{"record":"part","target":"x1-prod","name":"exports","digest":"0000000000000000000000000000000000000000000000000000000000000000","exports":{"vpn":{"endpoint":-7}}}"#,
                "\n"
            ),
        ),
        (
            export(Value::String(copy_of(b"up"))),
            spec_text!("exports      part 000000000000\n  vpn.endpoint   \"up\"\n"),
            spec_json!(
                r#"{"record":"part","target":"x1-prod","name":"exports","digest":"0000000000000000000000000000000000000000000000000000000000000000","exports":{"vpn":{"endpoint":"up"}}}"#,
                "\n"
            ),
        ),
        (
            export(Value::List(Box::new([Value::Bool(false), Value::Int(3)]))),
            spec_text!("exports      part 000000000000\n  vpn.endpoint   [false,3]\n"),
            spec_json!(
                r#"{"record":"part","target":"x1-prod","name":"exports","digest":"0000000000000000000000000000000000000000000000000000000000000000","exports":{"vpn":{"endpoint":[false,3]}}}"#,
                "\n"
            ),
        ),
        (
            export(Value::Map(Box::new([(copy_of(b"up"), Value::Bool(true))]))),
            spec_text!("exports      part 000000000000\n  vpn.endpoint   {\"up\":true}\n"),
            spec_json!(
                r#"{"record":"part","target":"x1-prod","name":"exports","digest":"0000000000000000000000000000000000000000000000000000000000000000","exports":{"vpn":{"endpoint":{"up":true}}}}"#,
                "\n"
            ),
        ),
    ]);
}

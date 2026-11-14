use super::assert_prints;
use crate::{
    errors::{Owns, Why},
    tests::fixtures,
};

#[test]
fn ids_bytes() {
    let cases = [
        (
            fixtures::ids_error(Owns::Data, Why::NoUser),
            "tasks/boot/task.json:22:7: carbon-lab: boot: mail owns mail/tls.key and has no fixed id: no user mail is declared\n",
            concat!(
                r#"{"record":"error","kind":"ids","ids":{"entry":"mail/tls.key","of":"data","owner":"mail","why":"no_user"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-lab","task":"boot"}"#,
                "\n"
            ),
        ),
        (
            fixtures::ids_error(Owns::Identity, Why::NoUser),
            "tasks/boot/task.json:22:7: carbon-lab: boot: mail owns mail/tls.key and has no fixed id: no user mail is declared\n",
            concat!(
                r#"{"record":"error","kind":"ids","ids":{"entry":"mail/tls.key","of":"identity","owner":"mail","why":"no_user"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-lab","task":"boot"}"#,
                "\n"
            ),
        ),
        (
            fixtures::ids_error(Owns::Download, Why::NotPackageUser),
            "tasks/boot/task.json:22:7: carbon-lab: boot: mail owns mail/tls.key and has no fixed id: a download needs a package user\n",
            concat!(
                r#"{"record":"error","kind":"ids","ids":{"entry":"mail/tls.key","of":"download","owner":"mail","why":"not_package_user"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-lab","task":"boot"}"#,
                "\n"
            ),
        ),
        (
            fixtures::ids_error(Owns::User, Why::OneId),
            "tasks/boot/task.json:22:7: carbon-lab: boot: mail owns mail and has no fixed id: a package user needs both uid and gid or neither\n",
            concat!(
                r#"{"record":"error","kind":"ids","ids":{"entry":"mail","of":"user","owner":"mail","why":"one_id"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-lab","task":"boot"}"#,
                "\n"
            ),
        ),
        (
            fixtures::ids_error(Owns::Secret, Why::NoUser),
            "tasks/boot/task.json:22:7: carbon-lab: boot: mail owns mail/tls.key and has no fixed id: no user mail is declared\n",
            concat!(
                r#"{"record":"error","kind":"ids","ids":{"entry":"mail/tls.key","of":"secret","owner":"mail","why":"no_user"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-lab","task":"boot"}"#,
                "\n"
            ),
        ),
        (
            fixtures::ids_error(Owns::Secret, Why::NoGroup),
            "tasks/boot/task.json:22:7: carbon-lab: boot: mail owns mail/tls.key and has no fixed id: no group mail is declared\n",
            concat!(
                r#"{"record":"error","kind":"ids","ids":{"entry":"mail/tls.key","of":"secret","owner":"mail","why":"no_group"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-lab","task":"boot"}"#,
                "\n"
            ),
        ),
        (
            fixtures::ids_error(Owns::Secret, Why::NoUid),
            "tasks/boot/task.json:22:7: carbon-lab: boot: mail owns mail/tls.key and has no fixed id: the user has no uid\n",
            concat!(
                r#"{"record":"error","kind":"ids","ids":{"entry":"mail/tls.key","of":"secret","owner":"mail","why":"no_uid"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-lab","task":"boot"}"#,
                "\n"
            ),
        ),
        (
            fixtures::ids_error(Owns::Secret, Why::NoGid),
            "tasks/boot/task.json:22:7: carbon-lab: boot: mail owns mail/tls.key and has no fixed id: the group has no gid\n",
            concat!(
                r#"{"record":"error","kind":"ids","ids":{"entry":"mail/tls.key","of":"secret","owner":"mail","why":"no_gid"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-lab","task":"boot"}"#,
                "\n"
            ),
        ),
        (
            fixtures::ids_error(Owns::Secret, Why::OneId),
            "tasks/boot/task.json:22:7: carbon-lab: boot: mail owns mail/tls.key and has no fixed id: a package user needs both uid and gid or neither\n",
            concat!(
                r#"{"record":"error","kind":"ids","ids":{"entry":"mail/tls.key","of":"secret","owner":"mail","why":"one_id"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-lab","task":"boot"}"#,
                "\n"
            ),
        ),
        (
            fixtures::ids_error(Owns::Secret, Why::NotPackageUser),
            "tasks/boot/task.json:22:7: carbon-lab: boot: mail owns mail/tls.key and has no fixed id: a download needs a package user\n",
            concat!(
                r#"{"record":"error","kind":"ids","ids":{"entry":"mail/tls.key","of":"secret","owner":"mail","why":"not_package_user"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-lab","task":"boot"}"#,
                "\n"
            ),
        ),
        (
            fixtures::ids_error(Owns::Secret, Why::NotPackageGroup),
            "tasks/boot/task.json:22:7: carbon-lab: boot: mail owns mail/tls.key and has no fixed id: a download needs a package group\n",
            concat!(
                r#"{"record":"error","kind":"ids","ids":{"entry":"mail/tls.key","of":"secret","owner":"mail","why":"not_package_group"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-lab","task":"boot"}"#,
                "\n"
            ),
        ),
    ];
    for (error, text, json) in cases {
        assert_prints(&error, text, json);
    }
}

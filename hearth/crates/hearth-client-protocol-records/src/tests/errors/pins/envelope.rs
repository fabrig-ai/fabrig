use super::assert_prints;
use crate::{diff::Side, errors::Error, tests::fixtures};
fn sided(side: Side) -> Error {
    let mut error = fixtures::duplicate();
    error.side = Some(side);
    error
}

fn path_only() -> Error {
    let mut error = fixtures::modules_guard();
    error.at.as_mut().expect("fixture place").position = None;
    error
}

#[test]
fn envelope_bytes() {
    let cases = [
        (
            sided(Side::From),
            "from: envs/lab.json:1:14: duplicate key \"motd\" (envs/lab.json:1:2)\n",
            concat!(
                r#"{"record":"error","side":"from","kind":"decode","decode":{"key":"motd","kind":"duplicate"},"class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"},"also":{"column":2,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            sided(Side::Both),
            "both: envs/lab.json:1:14: duplicate key \"motd\" (envs/lab.json:1:2)\n",
            concat!(
                r#"{"record":"error","side":"both","kind":"decode","decode":{"key":"motd","kind":"duplicate"},"class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"},"also":{"column":2,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            path_only(),
            "tasks/boot/task.json: carbon-lab: boot: requires modules: no initramfs modules for this hardware\n",
            concat!(
                r#"{"record":"error","kind":"guard","guard":{"message":"no initramfs modules for this hardware","name":"modules"},"class":"configuration","at":{"path":"tasks/boot/task.json"},"target":"carbon-lab","task":"boot"}"#,
                "\n"
            ),
        ),
    ];
    for (error, text, json) in cases {
        assert_prints(&error, text, json);
    }
}

use super::assert_prints;
use crate::tests::fixtures;
use hearth_domain_render_vocabulary as render;
use render::errors::{Command, Ended};

use skein_lib::bytes::copy_of;

#[test]
fn tool_tail_lines_are_indented_in_order() {
    let error = super::plain(render::errors::Kind::Tool(render::errors::Tool {
        command: Command::Mmdebstrap,
        ended: Ended::Exit(1),
        tail: Box::new([copy_of(b"a"), copy_of(b"b")]),
    }));
    assert_prints(
        &error,
        "mmdebstrap exited 1\n  a\n  b\n",
        concat!(
            r#"{"record":"error","kind":"tool","tool":{"command":"mmdebstrap","ended":{"code":1,"kind":"exit"},"tail":["a","b"]},"class":"configuration"}"#,
            "\n"
        ),
    );
}

#[test]
fn tool_bytes() {
    let cases = [
        (
            fixtures::git_failure(Command::Mmdebstrap, 128),
            "mmdebstrap exited 128\n  fatal: not a tree object\n",
            concat!(
                r#"{"record":"error","kind":"tool","tool":{"command":"mmdebstrap","ended":{"code":128,"kind":"exit"},"tail":["fatal: not a tree object"]},"class":"operational"}"#,
                "\n"
            ),
        ),
        (
            fixtures::git_failure(Command::RevParse, 128),
            "git rev-parse exited 128\n  fatal: not a tree object\n",
            concat!(
                r#"{"record":"error","kind":"tool","tool":{"command":"rev_parse","ended":{"code":128,"kind":"exit"},"tail":["fatal: not a tree object"]},"class":"operational"}"#,
                "\n"
            ),
        ),
        (
            fixtures::git_failure(Command::CatFile, 128),
            "git cat-file exited 128\n  fatal: not a tree object\n",
            concat!(
                r#"{"record":"error","kind":"tool","tool":{"command":"cat_file","ended":{"code":128,"kind":"exit"},"tail":["fatal: not a tree object"]},"class":"operational"}"#,
                "\n"
            ),
        ),
        (
            fixtures::mmdebstrap_error(Ended::Timeout),
            "mmdebstrap timed out\n",
            concat!(
                r#"{"record":"error","kind":"tool","tool":{"command":"mmdebstrap","ended":{"kind":"timeout"},"tail":[]},"class":"operational"}"#,
                "\n"
            ),
        ),
        (
            fixtures::mmdebstrap_error(Ended::Start),
            "mmdebstrap could not start\n",
            concat!(
                r#"{"record":"error","kind":"tool","tool":{"command":"mmdebstrap","ended":{"kind":"start"},"tail":[]},"class":"operational"}"#,
                "\n"
            ),
        ),
        (
            fixtures::mmdebstrap_error(Ended::Signal(9)),
            "mmdebstrap was killed by signal 9\n",
            concat!(
                r#"{"record":"error","kind":"tool","tool":{"command":"mmdebstrap","ended":{"signal":9,"kind":"signal"},"tail":[]},"class":"operational"}"#,
                "\n"
            ),
        ),
        (
            fixtures::mmdebstrap_error(Ended::Output),
            "mmdebstrap gave invalid output\n",
            concat!(
                r#"{"record":"error","kind":"tool","tool":{"command":"mmdebstrap","ended":{"kind":"output"},"tail":[]},"class":"operational"}"#,
                "\n"
            ),
        ),
        (
            fixtures::mmdebstrap_error(Ended::Missing { oid: render::revision::Oid(copy_of(b"1a2b3c4d")) }),
            "mmdebstrap is missing object 1a2b3c4d\n",
            concat!(
                r#"{"record":"error","kind":"tool","tool":{"command":"mmdebstrap","ended":{"kind":"missing","oid":"1a2b3c4d"},"tail":[]},"class":"operational"}"#,
                "\n"
            ),
        ),
        (
            fixtures::mmdebstrap_error(Ended::Stream),
            "mmdebstrap failed to read or write a pipe\n",
            concat!(
                r#"{"record":"error","kind":"tool","tool":{"command":"mmdebstrap","ended":{"kind":"stream"},"tail":[]},"class":"operational"}"#,
                "\n"
            ),
        ),
    ];
    for (error, text, json) in cases {
        assert_prints(&error, text, json);
    }
}

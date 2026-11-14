use super::assert_prints;
use crate::{
    errors::{Syntax, syntax::Expected},
    tests::fixtures,
};
use skein_lib::bytes::copy_of;

#[expect(clippy::too_many_lines, reason = "literal text and JSON pins keep each error beside its bytes")]
#[test]
fn syntax_bytes() {
    let cases = [
        (
            fixtures::syntax_error(Syntax::TooLong),
            "tasks/motd/tree/etc/issue.j2:2:1: motd: a template past its length limit\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"syntax","fault":"too_long"},"class":"configuration","at":{"column":1,"line":2,"path":"tasks/motd/tree/etc/issue.j2"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            fixtures::syntax_error(Syntax::Character),
            "tasks/motd/tree/etc/issue.j2:2:1: motd: an unexpected character\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"syntax","fault":"character"},"class":"configuration","at":{"column":1,"line":2,"path":"tasks/motd/tree/etc/issue.j2"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            fixtures::syntax_error(Syntax::Unterminated),
            "tasks/motd/tree/etc/issue.j2:2:1: motd: a string with no end\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"syntax","fault":"unterminated"},"class":"configuration","at":{"column":1,"line":2,"path":"tasks/motd/tree/etc/issue.j2"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            fixtures::syntax_error(Syntax::Escape),
            "tasks/motd/tree/etc/issue.j2:2:1: motd: an invalid escape\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"syntax","fault":"escape"},"class":"configuration","at":{"column":1,"line":2,"path":"tasks/motd/tree/etc/issue.j2"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            fixtures::syntax_error(Syntax::Integer),
            "tasks/motd/tree/etc/issue.j2:2:1: motd: an integer outside i64\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"syntax","fault":"integer"},"class":"configuration","at":{"column":1,"line":2,"path":"tasks/motd/tree/etc/issue.j2"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            fixtures::syntax_error(Syntax::TooDeep),
            "tasks/motd/tree/etc/issue.j2:2:1: motd: too deep\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"syntax","fault":"too_deep"},"class":"configuration","at":{"column":1,"line":2,"path":"tasks/motd/tree/etc/issue.j2"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            fixtures::syntax_error(Syntax::Misplaced),
            "tasks/motd/tree/etc/issue.j2:2:1: motd: a misplaced expression\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"syntax","fault":"misplaced"},"class":"configuration","at":{"column":1,"line":2,"path":"tasks/motd/tree/etc/issue.j2"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            fixtures::syntax_error(Syntax::UnknownFilter),
            "tasks/motd/tree/etc/issue.j2:2:1: motd: an unknown filter\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"syntax","fault":"unknown_filter"},"class":"configuration","at":{"column":1,"line":2,"path":"tasks/motd/tree/etc/issue.j2"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            fixtures::syntax_error(Syntax::UnknownTest),
            "tasks/motd/tree/etc/issue.j2:2:1: motd: an unknown test\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"syntax","fault":"unknown_test"},"class":"configuration","at":{"column":1,"line":2,"path":"tasks/motd/tree/etc/issue.j2"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            fixtures::syntax_error(Syntax::Utf8),
            "tasks/motd/tree/etc/issue.j2:2:1: motd: not UTF-8\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"syntax","fault":"utf8"},"class":"configuration","at":{"column":1,"line":2,"path":"tasks/motd/tree/etc/issue.j2"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            fixtures::syntax_error(Syntax::Return),
            "tasks/motd/tree/etc/issue.j2:2:1: motd: a carriage return\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"syntax","fault":"return"},"class":"configuration","at":{"column":1,"line":2,"path":"tasks/motd/tree/etc/issue.j2"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            fixtures::syntax_error(Syntax::UnknownStatement),
            "tasks/motd/tree/etc/issue.j2:2:1: motd: an unknown statement\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"syntax","fault":"unknown_statement"},"class":"configuration","at":{"column":1,"line":2,"path":"tasks/motd/tree/etc/issue.j2"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            fixtures::syntax_error(Syntax::Unmatched),
            "tasks/motd/tree/etc/issue.j2:2:1: motd: a close with no open block\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"syntax","fault":"unmatched"},"class":"configuration","at":{"column":1,"line":2,"path":"tasks/motd/tree/etc/issue.j2"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            fixtures::syntax_error(Syntax::Unended),
            "tasks/motd/tree/etc/issue.j2:2:1: motd: a block left open\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"syntax","fault":"unended"},"class":"configuration","at":{"column":1,"line":2,"path":"tasks/motd/tree/etc/issue.j2"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            fixtures::syntax_error(Syntax::Shadows),
            "tasks/motd/tree/etc/issue.j2:2:1: motd: a root of the context bound\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"syntax","fault":"shadows"},"class":"configuration","at":{"column":1,"line":2,"path":"tasks/motd/tree/etc/issue.j2"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            fixtures::syntax_error(Syntax::Interpolation),
            "tasks/motd/tree/etc/issue.j2:2:1: motd: a statement in an interpolated string\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"syntax","fault":"interpolation"},"class":"configuration","at":{"column":1,"line":2,"path":"tasks/motd/tree/etc/issue.j2"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            fixtures::syntax_error(Syntax::Expected(Expected::Operand)),
            "tasks/motd/tree/etc/issue.j2:2:1: motd: expected operand\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"syntax","fault":"expected","expected":"operand"},"class":"configuration","at":{"column":1,"line":2,"path":"tasks/motd/tree/etc/issue.j2"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            fixtures::syntax_error(Syntax::Expected(Expected::Operator)),
            "tasks/motd/tree/etc/issue.j2:2:1: motd: expected operator\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"syntax","fault":"expected","expected":"operator"},"class":"configuration","at":{"column":1,"line":2,"path":"tasks/motd/tree/etc/issue.j2"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            fixtures::syntax_error(Syntax::Expected(Expected::Name)),
            "tasks/motd/tree/etc/issue.j2:2:1: motd: expected name\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"syntax","fault":"expected","expected":"name"},"class":"configuration","at":{"column":1,"line":2,"path":"tasks/motd/tree/etc/issue.j2"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            fixtures::syntax_error(Syntax::Expected(Expected::CloseParen)),
            "tasks/motd/tree/etc/issue.j2:2:1: motd: expected )\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"syntax","fault":"expected","expected":")"},"class":"configuration","at":{"column":1,"line":2,"path":"tasks/motd/tree/etc/issue.j2"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            fixtures::syntax_error(Syntax::Expected(Expected::CloseBracket)),
            "tasks/motd/tree/etc/issue.j2:2:1: motd: expected ]\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"syntax","fault":"expected","expected":"]"},"class":"configuration","at":{"column":1,"line":2,"path":"tasks/motd/tree/etc/issue.j2"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            fixtures::syntax_error(Syntax::Expected(Expected::In)),
            "tasks/motd/tree/etc/issue.j2:2:1: motd: expected in\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"syntax","fault":"expected","expected":"in"},"class":"configuration","at":{"column":1,"line":2,"path":"tasks/motd/tree/etc/issue.j2"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            fixtures::syntax_error(Syntax::Expected(Expected::Else)),
            "tasks/motd/tree/etc/issue.j2:2:1: motd: expected else\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"syntax","fault":"expected","expected":"else"},"class":"configuration","at":{"column":1,"line":2,"path":"tasks/motd/tree/etc/issue.j2"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            fixtures::syntax_error(Syntax::Expected(Expected::Statement)),
            "tasks/motd/tree/etc/issue.j2:2:1: motd: expected statement\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"syntax","fault":"expected","expected":"statement"},"class":"configuration","at":{"column":1,"line":2,"path":"tasks/motd/tree/etc/issue.j2"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            fixtures::syntax_error(Syntax::Expected(Expected::Assign)),
            "tasks/motd/tree/etc/issue.j2:2:1: motd: expected =\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"syntax","fault":"expected","expected":"="},"class":"configuration","at":{"column":1,"line":2,"path":"tasks/motd/tree/etc/issue.j2"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            fixtures::syntax_error(Syntax::Expected(Expected::TagEnd)),
            "tasks/motd/tree/etc/issue.j2:2:1: motd: expected tag end\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"syntax","fault":"expected","expected":"tag end"},"class":"configuration","at":{"column":1,"line":2,"path":"tasks/motd/tree/etc/issue.j2"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            fixtures::syntax_error(Syntax::Arity { filter: copy_of(b"join"), expected: 1, given: 2 }),
            "tasks/motd/tree/etc/issue.j2:2:1: motd: \"join\" takes 1 argument, given 2\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"syntax","fault":"arity","filter":"join","expected":1,"given":2},"class":"configuration","at":{"column":1,"line":2,"path":"tasks/motd/tree/etc/issue.j2"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            fixtures::syntax_error(Syntax::Arity { filter: copy_of(b"join"), expected: 2, given: 1 }),
            "tasks/motd/tree/etc/issue.j2:2:1: motd: \"join\" takes 2 arguments, given 1\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"syntax","fault":"arity","filter":"join","expected":2,"given":1},"class":"configuration","at":{"column":1,"line":2,"path":"tasks/motd/tree/etc/issue.j2"},"task":"motd"}"#,
                "\n"
            ),
        ),
    ];
    for (error, text, json) in cases {
        assert_prints(&error, text, json);
    }
}

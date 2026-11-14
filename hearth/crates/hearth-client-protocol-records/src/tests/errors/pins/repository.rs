use super::assert_prints;
use crate::{errors::Repository, tests::fixtures};
use skein_lib::bytes::copy_of;

#[test]
fn repository_bytes() {
    let cases = [
        (
            fixtures::repository_error(Repository::NoTree),
            "no tree entry at this path\n",
            concat!(r#"{"record":"error","kind":"repository","repository":"no_tree","class":"configuration"}"#, "\n"),
        ),
        (
            fixtures::repository_error(Repository::LinkTarget),
            "a symbolic link target out of form\n",
            concat!(
                r#"{"record":"error","kind":"repository","repository":"link_target","class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            fixtures::repository_error(Repository::Name),
            "a name protocol/format/README.md 2.4 does not allow\n",
            concat!(r#"{"record":"error","kind":"repository","repository":"name","class":"configuration"}"#, "\n"),
        ),
        (
            fixtures::repository_error(Repository::Declaration),
            "a task directory without task.json\n",
            concat!(
                r#"{"record":"error","kind":"repository","repository":"declaration","class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            fixtures::repository_error(Repository::Static),
            "a template without a destination name\n",
            concat!(r#"{"record":"error","kind":"repository","repository":"static","class":"configuration"}"#, "\n"),
        ),
        (
            fixtures::repository_error(Repository::Destination),
            "two sources of one destination\n",
            concat!(
                r#"{"record":"error","kind":"repository","repository":"destination","class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            fixtures::repository_error(Repository::Submodule),
            "a submodule under a path render reads\n",
            concat!(r#"{"record":"error","kind":"repository","repository":"submodule","class":"configuration"}"#, "\n"),
        ),
        (
            fixtures::repository_error(Repository::Link),
            "a document or template listed as a symbolic link\n",
            concat!(r#"{"record":"error","kind":"repository","repository":"link","class":"configuration"}"#, "\n"),
        ),
        (
            fixtures::repository_error(Repository::Stray),
            "a path protocol/format/README.md 2.1 does not allow\n",
            concat!(r#"{"record":"error","kind":"repository","repository":"stray","class":"configuration"}"#, "\n"),
        ),
        (
            fixtures::repository_error(Repository::Environment { env: copy_of(b"prod") }),
            "no environment prod under envs/\n",
            concat!(
                r#"{"record":"error","kind":"repository","repository":{"kind":"environment","env":"prod"},"class":"configuration"}"#,
                "\n"
            ),
        ),
    ];
    for (error, text, json) in cases {
        assert_prints(&error, text, json);
    }
}

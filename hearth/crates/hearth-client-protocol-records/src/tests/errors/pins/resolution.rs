use super::assert_prints;
use super::plain;
use crate::tests::fixtures::{self, ResolutionInput};
use hearth_domain_render_vocabulary as render;
use render::errors::{Kind, Resolution};

#[test]
fn resolution_bytes() {
    let cases = [
        (
            fixtures::resolution_error(ResolutionInput::TooLarge),
            "a resolution record past 4194304 bytes\n",
            concat!(
                r#"{"record":"error","kind":"resolution","resolution":{"kind":"too_large","max":4194304},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            fixtures::resolution_error(ResolutionInput::Unknown),
            "apt cannot locate package \"apt\"\n",
            concat!(
                r#"{"record":"error","kind":"resolution","resolution":{"kind":"unknown","package":"apt"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            fixtures::resolution_error(ResolutionInput::Unsatisfiable),
            "unsatisfiable dependencies of \"apt\"\n",
            concat!(
                r#"{"record":"error","kind":"resolution","resolution":{"kind":"unsatisfiable","package":"apt"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            fixtures::resolution_error(ResolutionInput::Absent),
            "the archive http://one/ does not have forky\n",
            concat!(
                r#"{"record":"error","kind":"resolution","resolution":{"kind":"absent","uri":"http://one/","suite":"forky"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            fixtures::resolution_error(ResolutionInput::Unmatched),
            "apt would install apt 3.3.3 amd64, which no Packages index lists\n",
            concat!(
                r#"{"record":"error","kind":"resolution","resolution":{"kind":"unmatched","name":"apt","version":"3.3.3","architecture":"amd64"},"class":"operational"}"#,
                "\n"
            ),
        ),
        (
            fixtures::resolution_error(ResolutionInput::Disagree),
            "the Packages indices disagree on apt 3.3.3 amd64\n",
            concat!(
                r#"{"record":"error","kind":"resolution","resolution":{"kind":"disagree","name":"apt","version":"3.3.3","architecture":"amd64"},"class":"operational"}"#,
                "\n"
            ),
        ),
        (
            fixtures::resolution_error(ResolutionInput::TooMany),
            "a closure past 8192 packages\n",
            concat!(
                r#"{"record":"error","kind":"resolution","resolution":{"kind":"too_many","max":8192},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Resolution(Resolution::Unsatisfiable { package: None })),
            "unsatisfiable dependencies\n",
            concat!(
                r#"{"record":"error","kind":"resolution","resolution":{"kind":"unsatisfiable"},"class":"configuration"}"#,
                "\n"
            ),
        ),
    ];
    for (error, text, json) in cases {
        assert_prints(&error, text, json);
    }
}

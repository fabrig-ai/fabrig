use super::fixtures;
use crate::{
    form::{self, Form},
    limits,
};
#[test]
fn accounts_print_as_choices_9_shows() {
    let text = form::print(Form::Text, &fixtures::accounts_spec(), None, &limits::SHIPPED).expect("spec");
    assert_eq!(
        text.as_ref(),
        concat!(
            "target       x1-prod   base x1   environment prod\n",
            "placement    metal efi 7th-gen\n",
            "spec         9a0c5e1f2b3d\n",
            "config       part 000000000000\n",
            "  groups     mail package\n",
            "  groups     ops 1500\n",
            "  users      alice 1000 alice /home/alice /bin/bash Alice User\n",
            "  users      postfix package\n",
            "  members    alice ops\n"
        )
        .as_bytes()
    );
}
#[test]
fn json_nests_accounts_under_config() {
    let json = form::print(Form::JsonLines, &fixtures::accounts_spec(), None, &limits::SHIPPED).expect("spec");
    let start =
        skein_lib::bytes::find(&json, br#"{"record":"part","target":"x1-prod","name":"config""#).expect("config part");
    assert_eq!(json.get(start..).expect("suffix"),concat!(r#"{"record":"part","target":"x1-prod","name":"config","digest":"0000000000000000000000000000000000000000000000000000000000000000","config":{"groups":{"mail":{"kind":"package"},"ops":{"gid":1500,"kind":"sysusers"}},"members":[{"group":"ops","user":"alice"}],"presets":{},"scripts":[],"tmpfiles":{},"tree":{},"users":{"alice":{"gecos":"Alice User","group":"alice","home":"/home/alice","kind":"sysusers","shell":"/bin/bash","uid":1000},"postfix":{"kind":"package"}}}}"#,"\n").as_bytes());
}
#[test]
fn account_changes_pad_their_keys_to_the_diff_column() {
    let text = form::print(Form::Text, &fixtures::accounts_change(), None, &limits::SHIPPED).expect("diff");
    assert_eq!(
        text.as_ref(),
        concat!(
            "x1-prod                                             rebuild from config\n",
            "  config   - groups ops                             1500\n",
            "  config   ~ users alice                            1001 alice /home/alice /bin/bash\n",
            "  config   - members alice ops\n",
            "  config   + members alice video\n"
        )
        .as_bytes()
    );
}
#[test]
fn account_changes_keep_their_values_in_json() {
    let json = form::print(
        Form::JsonLines,
        &fixtures::accounts_change(),
        Some(crate::records::Type::Change),
        &limits::SHIPPED,
    )
    .expect("diff");
    assert_eq!(json.as_ref(),concat!(r#"{"record":"change","target":"x1-prod","kind":"changed","changed":{"parts":[{"kind":"config","groups":[{"key":"ops","kind":"removed","value":{"gid":1500,"kind":"sysusers"}}],"users":[{"after":{"gecos":"","group":"alice","home":"/home/alice","kind":"sysusers","shell":"/bin/bash","uid":1001},"before":{"kind":"package"},"key":"alice","kind":"changed"}],"members":[{"kind":"removed","value":{"group":"ops","user":"alice"}},{"kind":"added","value":{"group":"video","user":"alice"}}],"presets":[],"tmpfiles":[],"tree":[]}],"rebuild":{"kind":"from_config"}}}"#,"\n").as_bytes());
}

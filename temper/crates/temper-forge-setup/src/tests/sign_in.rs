use super::fixtures;
use crate::sign_in::{instructions, quote, secret};
#[test]
fn instructions_name_the_files_and_never_the_secret() {
    let secret = secret(&[0; 32]);
    let printed = instructions(
        b"/opt/forgejo",
        Some(&fixtures::place()),
        b"forgejo",
        b"/srv/client secret",
        b"http://127.0.0.1:8080/.well-known/openid-configuration",
    );
    let command = b"'/opt/forgejo' --work-path '/srv/forge-2/forgejo' --config '/srv/forge-2/forgejo/app.ini' admin auth add-oauth --name temper --provider openidConnect --key 'forgejo' --secret \"$(cat '/srv/client secret')\" --auto-discover-url 'http://127.0.0.1:8080/.well-known/openid-configuration'\n";
    assert!(printed.starts_with(command));
    assert!(skein_lib::bytes::find(&printed, secret.expose()).is_none());
    assert!(skein_lib::bytes::find(&printed, crate::sign_in::SETTINGS).is_some());
}
#[test]
fn managed_instructions_use_forgejo_on_the_hosts_path() {
    let printed = instructions(
        b"forgejo",
        None,
        b"git",
        b"/data/secret",
        b"https://temper.example.com/.well-known/openid-configuration",
    );
    assert!(printed.starts_with(b"'forgejo' admin auth add-oauth"));
    assert!(skein_lib::bytes::find(&printed, b"--work-path").is_none());
    assert!(
        skein_lib::bytes::find(&printed, b"'https://temper.example.com/.well-known/openid-configuration'").is_some()
    );
}
#[test]
fn randomness_is_unpadded_base64url_and_shell_words_are_literal() {
    let encoded = secret(&[255; 32]);
    assert_eq!(encoded.expose(), b"__________________________________________8");
    assert_eq!(&*quote(b"/a'$(false)"), b"'/a'\\''$(false)'");
}

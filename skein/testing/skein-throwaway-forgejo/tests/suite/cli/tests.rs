#[test]
fn run_user_matches_the_effective_users_passwd_login() {
    let expected = skein_shell::login_passwd(skein_shell::effective_user()).expect("effective user's passwd login");
    assert_eq!(super::run_user().expect("run user").as_bytes(), expected.as_ref());
}

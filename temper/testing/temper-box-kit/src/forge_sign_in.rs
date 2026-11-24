//! Configuration and commands shared by forge sign-in proofs.
use super::{configuration, write_private};
use std::{
    path::Path,
    process::{Command, Output},
};

/// Add the Google member and the matching confidential Forgejo client.
pub fn configure(directory: &Path, forge: &str, temper: &str, google: &str) {
    let path = directory.join(temper_config_shell::FILE);
    let mut file = super::read_file(&path, temper_config_shell::Shape::Standalone);
    add_to_file(&mut file, forge, temper, google);
    if !directory.join("forge-1/setup").exists() {
        let address: std::net::SocketAddr = forge.parse().expect("fixture forge socket");
        super::owned::setup(
            directory,
            &super::owned::Names {
                forgejo: Path::new("/opt/forgejo"),
                runner: None,
                port: address.port(),
                bot: "temper",
                person: "sam",
            },
        );
    }
    write_private(&directory.join("google.secret"), GOOGLE_SECRET);
    super::write_file(&path, &file, temper_config_shell::Shape::Standalone);
}

pub const GOOGLE_SECRET: &[u8] = b"throwaway-google-secret";

/// Compose the Google member and Forgejo client over a fixture's configuration file.
pub fn add_to_file(file: &mut temper_config_shell::File, forge: &str, temper: &str, google: &str) {
    file.owner = Some("sam@example.com".into());
    file.openid = Some(temper_config_shell::OpenId {
        key: Some("provider.key".into()),
        addresses: Some(vec![3]),
        clients: vec![temper_config_shell::OpenIdClient {
            id: "forgejo".into(),
            secret: Some("forgejo.secret".into()),
            redirect: format!("http://{forge}/user/oauth2/temper/callback"),
        }],
    });
    file.listen = Some(temper.parse().expect("temper listener"));
    file.sign_in = Some(temper_config_shell::SignIn {
        client: "temper-test".into(),
        secret: "google.secret".into(),
        members: vec![temper_config_shell::Member { address: "sam@example.com".into(), login: "sam".into() }],
        forges: vec![temper_config_shell::ForgeSignIn { forge: 1, client: "forgejo".into() }],
    });
    file.test.google = Some(temper_config_shell::TestGoogle {
        issuer: format!("http://{google}"),
        authorization: format!("http://{google}/auth"),
        token: format!("http://{google}/token"),
        keys: format!("http://{google}/keys"),
        transport: temper_config_shell::Transport::Plaintext,
    });
}

#[must_use]
pub fn command(directory: &Path) -> Command {
    let mut command = Command::new(skein_scratch::paths::binary("temper-box"));
    command.args(["forge", "sign-in"]).args(configuration::flags(directory)).arg("1").current_dir(directory);
    command
}

pub fn success(output: &Output) {
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
}

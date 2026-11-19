//! A provider in a scratch deployment; the callback's harness port is learned before its restart.
use crate::{Tier, configuration};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    process::{Command, Stdio},
};

impl Tier {
    #[must_use]
    pub fn authorize(name: &str) -> (Self, String) {
        let tier = Self::reserved(name);
        let key = tier.scratch.path().join("provider.key");
        let status = Command::new("openssl")
            .args(["genpkey", "-algorithm", "RSA", "-pkeyopt", "rsa_keygen_bits:3072", "-out"])
            .arg(&key)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("generate scratch RSA key");
        assert!(status.success(), "scratch RSA key generation succeeds");
        fs::set_permissions(&key, fs::Permissions::from_mode(0o600)).expect("private scratch key");
        configuration::private(&tier.scratch.path().join("provider-client.secret"), b"scratch-provider-client-secret");
        let file = configuration::read_init(tier.scratch.path()).file;
        let origin = format!("http://{}", file.listen.expect("reserved listener"));
        (tier, origin)
    }
    /// Origins in the harness bind dynamic ports. Keep the story on that loop by
    /// writing the exact registered callback before restarting the scratch service.
    pub fn authorize_callback(&self, callback: &str) {
        let mut file = configuration::read_init(self.scratch.path()).file;
        file.openid = Some(temper_config_shell::OpenId {
            key: Some(self.scratch.path().join("provider.key")),
            clients: vec![temper_config_shell::OpenIdClient {
                id: "forgejo".into(),
                secret: Some(self.scratch.path().join("provider-client.secret")),
                redirect: callback.into(),
            }],
            addresses: Some(vec![]),
        });
        configuration::write_file(self.scratch.path(), &file);
    }
}

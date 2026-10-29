//! Held loopback listeners and the configuration documents that name them.
use skein_io::kernel::Fd;
use std::{
    fs,
    net::{SocketAddr, TcpListener},
    os::fd::IntoRawFd,
    path::Path,
};

pub fn listener() -> TcpListener {
    TcpListener::bind("127.0.0.1:0").expect("private fake listener")
}

pub fn descriptor(listener: TcpListener) -> Fd {
    Fd::new(listener.into_raw_fd())
}

pub fn provider(
    scripts: Box<[skein_fake_llm_domain::api::Script]>,
    credential: skein_llm::Credential,
) -> (skein_fake_peers::llm::Peer, SocketAddr, Fd) {
    let listener = listener();
    let address = listener.local_addr().expect("provider address");
    let mut peer = jig_channel_world::fixtures::provider_at(
        address,
        scripts,
        credential,
        skein_lib::Duration::ZERO,
        skein_fake_peers::Transport::Tls,
        skein_fake_llm_protocol::documents::Provider::OpenAi,
    );
    let fd = descriptor(listener);
    peer.adopt_listener(fd, address).expect("provider adopts listener");
    (peer, address, fd)
}

/// Update endpoint records before starting a binary; listeners remain owned by its world.
pub fn configure(directory: &Path, issuer: Option<SocketAddr>, provider: Option<SocketAddr>) {
    let path = directory.join("config.json");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("config")).expect("JSON");
    if let Some(address) = issuer {
        let sign_in = &mut value["engine"]["llm"]["providers"][0]["sources"]["sign_in"];
        sign_in["issuer"] = format!("http://{address}").into();
        for (field, route) in
            [("authorization", "authorize"), ("token", "token"), ("revocation", "revoke"), ("keys", "keys")]
        {
            sign_in[field] = format!("http://{address}/{route}").into();
        }
    }
    if let Some(address) = provider {
        value["engine"]["llm"]["endpoints"][0]["address"] = serde_json::json!({"socket":address.to_string()});
        let agent_path = directory.join("agent.json");
        let mut agent: serde_json::Value =
            serde_json::from_slice(&fs::read(&agent_path).expect("agent config")).expect("JSON");
        agent["endpoints"][0]["address"] = address.to_string().into();
        super::private(&agent_path, &serde_json::to_vec(&agent).expect("agent JSON"));
    }
    super::private(&path, &serde_json::to_vec(&value).expect("config JSON"));
}

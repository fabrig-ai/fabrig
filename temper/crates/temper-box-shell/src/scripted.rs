//! Choose the box's constant before composing its worker (protocol/configuration/providers.md, 4).
use std::net::{SocketAddr, TcpListener};
use temper_service::stock::Given;

pub(crate) const LISTENER: skein_shell::Variable = skein_shell::Variable {
    name: "TEMPER_INTERNAL_SCRIPTED_LISTENER",
    set: skein_shell::Set::Internal,
    form: "SOCKET",
    way: skein_shell::Way::Reads,
    line: "The scripted model's listener retained by the supervising box.",
    absent: "Choose loopback port 8370 when free, otherwise a kernel-assigned port.",
    notes: "The box passes the chosen address before composing its worker and hands the listener to the fake LLM.",
};
pub const ADDRESS: &[u8] = LISTENER.name.as_bytes();
pub(crate) const PREFERRED: SocketAddr =
    SocketAddr::V4(std::net::SocketAddrV4::new(std::net::Ipv4Addr::LOCALHOST, 8370));

pub fn choose() -> Result<TcpListener, String> {
    TcpListener::bind(PREFERRED)
        .or_else(|_| TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)))
        .map_err(|why| format!("the scripted model's listener: {why}"))
}
pub(crate) fn address(environment: &[crate::EnvironmentEntry]) -> Result<SocketAddr, String> {
    if let Some((_, bytes)) = environment.iter().find(|(name, _)| name.as_ref() == ADDRESS) {
        let address: SocketAddr = std::str::from_utf8(bytes)
            .ok()
            .and_then(|value| value.parse().ok())
            .ok_or("the scripted model's listener is not a socket address")?;
        if !address.ip().is_loopback() || address.port() == 0 {
            return Err("the scripted model's listener must be a nonzero loopback port".into());
        }
        return Ok(address);
    }
    // A serve without a supervising box has no kind-slot host to retain a listener.
    choose()?.local_addr().map_err(|why| why.to_string())
}
pub(crate) fn given(directories: &crate::Directories, given: &Given, address: SocketAddr) -> Given {
    given_with(directories, given, address, |path, defaults| {
        temper_service_shell::providers::read(path, defaults, &mut std::io::stderr())
    })
}
pub(crate) fn given_with(
    directories: &crate::Directories,
    given: &Given,
    address: SocketAddr,
    read: impl FnOnce(
        &std::path::Path,
        Box<[jig_protocol_config::ProviderSetting]>,
    ) -> Box<[jig_protocol_config::ProviderSetting]>,
) -> Given {
    let providers = read(&crate::path(&directories.data).join("providers.setting"), given.running());
    given.with_providers(providers, Box::new([temper_service::stock::scripted(address)]))
}

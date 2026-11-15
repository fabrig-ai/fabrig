//! Unresolved GitHub job-log origins (temper/docs/design/protocol/forge/README.md, 2.6).
use skein_forge::github::redirect::Origin;
use skein_forge_connection::settings::LogOrigin;
use skein_lib::List;

/// The twenty results hosts, in resolution order. Only the serving engine resolves them.
#[must_use]
pub fn defaults() -> List<LogOrigin> {
    let mut origins = List::with_capacity(20);
    for number in 0_u32..20 {
        origins
            .push(LogOrigin {
                origin: Origin {
                    tls: true,
                    host: format!("productionresultssa{number}.blob.core.windows.net").into_bytes().into_boxed_slice(),
                    port: 443,
                },
                address: skein_io::Addr::Ip(temper_engine_service::forge::HELD_ADDRESS),
            })
            .expect("twenty default origins");
    }
    origins
}

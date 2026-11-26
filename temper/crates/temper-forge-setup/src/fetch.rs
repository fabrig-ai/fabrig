//! Forgejo's pinned releases (protocol/setup/owned.md, 6).
use crate::{
    check::joined,
    setup::{Origin, Owned},
};
use alloc::boxed::Box;
use skein_forge::forgejo::pin::{self, Binary, Platform};
use skein_lib::List;
pub use temper_fetch::fetch::{Event, Failure, Fetch, Fetched, Named, Pin, Request, Unpacked, hex, step, url};

fn digest(binary: Binary, platform: Platform, arch: &str) -> [u8; 32] {
    let mut result = [0; 32];
    let text = pin::digest(binary, platform, arch).expect("pinned architecture").as_bytes();
    for (slot, digits) in result.iter_mut().zip(text.as_chunks::<2>().0) {
        let mut value = 0_u8;
        for digit in digits {
            let n = if digit.is_ascii_digit() {
                digit.checked_sub(b'0').expect("digit")
            } else {
                digit.checked_sub(b'a').expect("hex digit").checked_add(10).expect("hex digit")
            };
            value = value.checked_mul(16).expect("byte").checked_add(n).expect("byte");
        }
        *slot = value;
    }
    result
}
/// Selects release paths and digests for the host's architecture.
pub fn pins(owned: &Owned, platform: Platform, machine: &str) -> Result<Box<[Pin]>, Failure> {
    let arch = match machine {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        _ => return Err(Failure::Architecture(machine.as_bytes().into())),
    };
    if platform == Platform::Macos && arch != "arm64" {
        return Err(Failure::Architecture(machine.as_bytes().into()));
    }
    let mut pins = List::with_capacity(2);
    for binary in [Binary::Forgejo, Binary::Runner] {
        let (version, name, file) = match binary {
            Binary::Forgejo => {
                if !owned.fetch.forgejo {
                    continue;
                }
                (pin::VERSION, b"forgejo".as_slice(), &owned.forgejo)
            }
            Binary::Runner => {
                if !owned.fetch.runner {
                    continue;
                }
                (
                    pin::RUNNER_VERSION,
                    b"forgejo-runner".as_slice(),
                    &owned.runner.as_ref().expect("fetched runner exists").binary,
                )
            }
        };
        let origin = match &owned.fetch.mirror {
            Some(mirror) => match binary {
                Binary::Forgejo => mirror.forgejo.clone(),
                Binary::Runner => mirror.runner.clone(),
            },
            None => Origin { host: b"fabrig.ai".as_slice().into(), port: 443 },
        };
        let os = match platform {
            Platform::Linux => b"linux-".as_slice(),
            Platform::Macos => b"darwin-".as_slice(),
        };
        let path = joined(&[
            b"/temper/",
            name,
            b"/",
            version.as_bytes(),
            b"/",
            name,
            b"-",
            version.as_bytes(),
            b"-",
            os,
            arch.as_bytes(),
            b".zst",
        ]);
        pins.push(Pin {
            binary: Named::Forge(binary),
            origin,
            path,
            digest: digest(binary, platform, arch),
            file: file.clone(),
            packed: true,
        })
        .expect("two pins");
    }
    Ok(pins.into_boxed())
}

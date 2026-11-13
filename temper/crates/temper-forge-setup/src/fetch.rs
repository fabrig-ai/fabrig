//! Forgejo's pinned releases (protocol/setup/owned.md, 6).
use crate::{
    check::joined,
    setup::{Origin, Owned},
};
use alloc::boxed::Box;
use skein_forge::forgejo::pin::{self, Binary, Form, Platform};
use skein_lib::List;
pub use temper_fetch::fetch::{
    Bottle, Event, Extraction, Failure, Fetch, Fetched, Named, Pin, Request, hex, step, url,
};

fn digest(binary: Binary, platform: Platform, arch: &str, form: Form) -> [u8; 32] {
    let mut result = [0; 32];
    let text = pin::digest(binary, platform, arch, form).expect("pinned architecture").as_bytes();
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
        let (fetch, version, name, default, file) = match binary {
            Binary::Forgejo => {
                (owned.fetch.forgejo, pin::VERSION, b"forgejo".as_slice(), b"codeberg.org".as_slice(), &owned.forgejo)
            }
            Binary::Runner => {
                if !owned.fetch.runner {
                    continue;
                }
                (
                    true,
                    pin::RUNNER_VERSION,
                    b"forgejo-runner".as_slice(),
                    b"code.forgejo.org".as_slice(),
                    &owned.runner.as_ref().expect("fetched runner exists").binary,
                )
            }
        };
        if !fetch {
            continue;
        }
        let project = match binary {
            Binary::Forgejo => b"forgejo".as_slice(),
            Binary::Runner => b"runner".as_slice(),
        };
        let origin = match &owned.fetch.mirror {
            Some(mirror) => match binary {
                Binary::Forgejo => mirror.forgejo.clone(),
                Binary::Runner => mirror.runner.clone(),
            },
            None => Origin {
                host: match platform {
                    Platform::Linux => default.into(),
                    Platform::Macos => b"ghcr.io".as_slice().into(),
                },
                port: 443,
            },
        };
        let linux_path = joined(&[
            b"/forgejo/",
            project,
            b"/releases/download/v",
            version.as_bytes(),
            b"/",
            name,
            b"-",
            version.as_bytes(),
            b"-linux-",
            arch.as_bytes(),
        ]);
        let (path, bottle) = match platform {
            Platform::Linux => (linux_path, None),
            Platform::Macos => {
                let archive = digest(binary, platform, arch, Form::Archive);
                let path = joined(&[b"/v2/homebrew/core/", name, b"/blobs/sha256:", &hex(&archive)]);
                let member = joined(&[name, b"/", version.as_bytes(), b"/bin/", name]);
                (path, Some(Bottle { digest: archive, member, token: owned.fetch.mirror.is_none() }))
            }
        };
        pins.push(Pin {
            binary: Named::Forge(binary),
            origin,
            path,
            digest: digest(binary, platform, arch, Form::Binary),
            file: file.clone(),
            bottle,
        })
        .expect("two pins");
    }
    Ok(pins.into_boxed())
}

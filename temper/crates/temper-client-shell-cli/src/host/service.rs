//! One-time service credentials reach only their selected output (client/ctl/README.md, 7).
use super::OneShot;
use skein_lib::Secret;
use temper_client_domain_forge::sign_in::{CREDENTIAL_BYTES, service_link};
use temper_client_tools::jig::{
    FileEnd, Printed,
    call::{Parsed, Value, value},
    rows,
};

struct Reissue {
    directory: Box<[u8]>,
    json: bool,
}

pub(super) struct ServiceFile {
    pub(super) path: Option<Box<[u8]>>,
    pub(super) checked: bool,
    pub(super) credentials: std::collections::BTreeMap<u64, Secret>,
    reissue: Option<Reissue>,
    printed: Option<Printed>,
}
impl ServiceFile {
    pub(super) fn for_call(parsed: &Parsed, directory: Option<&[u8]>) -> Option<Self> {
        let Parsed::Call { call, json } = parsed else { return None };
        let (index, reissue) = match call.tool {
            rows::PARTY_MAKE_AT => (2, None),
            rows::PARTY_REISSUE_AT => {
                (1, Some(Reissue { directory: directory.unwrap_or_default().into(), json: *json }))
            }
            _ => return None,
        };
        let output = value(call, index)?;
        let path = if let Value::File(path) = output {
            Some(path.clone())
        } else if reissue.is_some() && *output == Value::Stdin {
            None
        } else {
            return None;
        };
        Some(Self {
            checked: path.is_none(),
            path,
            credentials: std::collections::BTreeMap::new(),
            reissue,
            printed: None,
        })
    }
}
impl OneShot {
    pub(super) fn service_print(&mut self, mut printed: Printed) {
        let Some(service) = &mut self.service_file else {
            self.print(printed);
            return;
        };
        if printed.status != 0 {
            self.print(printed);
            return;
        }
        let FileEnd::Service { party } = printed.file else {
            self.fail(b"service answer did not name its party");
            return;
        };
        let credential = service
            .credentials
            .remove(&party)
            .filter(|credential| service.reissue.is_none() || u32::try_from(credential.len()) == Ok(CREDENTIAL_BYTES));
        let Some(credential) = credential else {
            if service.reissue.is_some() {
                self.fail(format!("party {party}'s new credential was not answered; reissue it again").as_bytes());
            } else {
                // A keyed make replay has the party but no one-time credential.
                self.print(printed);
            }
            return;
        };
        let address = self.directory.address.as_deref().expect("signed-in address");
        if let Some(reissue) = &service.reissue {
            let link = service_link(address, &credential);
            let mut bytes = link.expose().to_vec();
            bytes.push(b'\n');
            let file = service.path.as_deref().unwrap_or(b"-");
            if !reissue.json {
                printed.stdout = reissue_advice(&printed.stdout, &reissue.directory, file);
            }
            if service.path.is_none() {
                let mut answer = printed.stdout.into_vec();
                if !answer.ends_with(b"\n") {
                    answer.push(b'\n');
                }
                self.output.push(true, answer.into_boxed_slice());
                printed.stdout = bytes.into_boxed_slice();
                self.print(printed);
            } else {
                service.printed = Some(printed);
                self.directory.store_service_link(&Secret::new(bytes.into_boxed_slice()));
                // The answer is durable; the write now has the file layer's deadline.
                self.until = None;
            }
        } else {
            let mut bytes = address.to_vec();
            bytes.push(b'\n');
            bytes.extend_from_slice(credential.expose());
            service.printed = Some(printed);
            self.directory.store_service(&Secret::new(bytes.into_boxed_slice()));
        }
    }
    pub(super) fn service_recorded(&mut self, result: &Result<(), Box<[u8]>>) {
        let service = self.service_file.as_mut().expect("service output");
        match result {
            Ok(()) => {
                let printed = service.printed.take().expect("durable answer");
                self.print(printed);
            }
            Err(error) if service.reissue.is_some() => {
                let mut line = service.path.as_deref().expect("service output file").to_vec();
                line.extend_from_slice(b" could not be written: ");
                line.extend_from_slice(error);
                self.fail(&line);
            }
            Err(_) => self.fail(b"service file could not be written"),
        }
    }
}

fn reissue_advice(answer: &[u8], directory: &[u8], file: &[u8]) -> Box<[u8]> {
    let mut line = answer.strip_suffix(b"\n").unwrap_or(answer).to_vec();
    line.extend_from_slice(b"; sign its directory in with temper ctl ");
    line.extend_from_slice(directory);
    line.extend_from_slice(b" sign-in ");
    line.extend_from_slice(file);
    line.push(b'\n');
    line.into_boxed_slice()
}

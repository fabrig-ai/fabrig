//! The service credential reaches only its private file (client/ctl/tools.md, 3).
use super::OneShot;
use skein_lib::Secret;
use temper_client_tools::jig::{
    FileEnd, Printed,
    call::{Parsed, Value, Values},
    rows,
};

pub(super) struct ServiceFile {
    pub(super) path: Box<[u8]>,
    pub(super) credentials: std::collections::BTreeMap<u64, Secret>,
    printed: Option<Printed>,
}
impl ServiceFile {
    pub(super) fn for_call(parsed: &Parsed) -> Option<Self> {
        let Parsed::Call { call, .. } = parsed else { return None };
        if call.tool != rows::PARTY_MAKE_AT {
            return None;
        }
        let Some(Some(Values::One(Value::File(path)))) = call.values.get(2) else { return None };
        Some(Self { path: path.clone(), credentials: std::collections::BTreeMap::new(), printed: None })
    }
}
impl OneShot {
    pub(super) fn service_print(&mut self, printed: Printed) {
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
        let Some(credential) = service.credentials.remove(&party) else {
            // A keyed replay has the party but no one-time credential.
            self.print(printed);
            return;
        };
        let mut bytes = self.directory.address.as_deref().expect("signed-in address").to_vec();
        bytes.push(b'\n');
        bytes.extend_from_slice(credential.expose());
        service.printed = Some(printed);
        self.directory.store_service(&Secret::new(bytes.into_boxed_slice()));
    }
    pub(super) fn service_recorded(&mut self, result: &Result<(), Box<[u8]>>) {
        match result {
            Ok(()) => {
                let printed =
                    self.service_file.as_mut().expect("service output").printed.take().expect("durable answer");
                self.print(printed);
            }
            Err(_) => self.fail(b"service file could not be written"),
        }
    }
}

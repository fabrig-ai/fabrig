//! Admission through ctl before starting the worker (testing/qa/deployment.md, 5).
use super::{DOCUMENT_BYTES, Json, configuration};
use crate::drive::{Input, Plan, Program};
use temper_config_shell::{FILE, Shape};

#[derive(Clone, Debug)]
pub struct Worker {
    name: Box<[u8]>,
    digest: Box<[u8]>,
}
impl Worker {
    pub fn read(document: &[u8], digest: &[u8]) -> Result<Self, String> {
        let file = configuration(FILE, document, Shape::WorkerAlone)?;
        let name = file.name.ok_or_else(|| format!("{FILE}: name: required for the deployment's worker"))?;
        let name = name.into_bytes().into();
        let digest = digest.strip_suffix(b"\n").unwrap_or(digest);
        if digest.len() != 64 || !digest.iter().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(b)) {
            return Err("worker.digest: expected 64 lowercase hexadecimal digits".into());
        }
        Ok(Self { name, digest: digest.into() })
    }
    pub fn listed(&self, bytes: &[u8]) -> Result<bool, String> {
        let Json::Object(members) = Json::parse("worker list", bytes, DOCUMENT_BYTES)? else {
            return Err("worker list: expected an object".into());
        };
        let Some((_, Json::Array(rows))) = members.iter().find(|(key, _)| key.as_ref() == b"rows") else {
            return Err("worker list: expected a rows array".into());
        };
        for row in rows {
            if row.member_string("worker list", b"name")? == self.name {
                return Ok(true);
            }
        }
        Ok(false)
    }
    #[must_use]
    pub fn add(&self, plan: &Plan) -> Program {
        plan.ctl(
            &[
                b"worker".as_slice().into(),
                b"add".as_slice().into(),
                self.name.clone(),
                b"--places".as_slice().into(),
                b"2".as_slice().into(),
                b"--digest".as_slice().into(),
                self.digest.clone(),
                b"--key".as_slice().into(),
                b"qa-deployment-worker".as_slice().into(),
            ],
            Input::Null,
        )
    }
}

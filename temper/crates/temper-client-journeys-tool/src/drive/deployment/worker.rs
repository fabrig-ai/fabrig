//! Admission through ctl before starting the worker (testing/qa/deployment.md, 5).
use super::{DOCUMENT_BYTES, Json};
use crate::drive::{Input, Plan, Program};

#[derive(Clone, Debug)]
pub struct Worker {
    name: Box<[u8]>,
    places: u32,
    digest: Box<[u8]>,
}
impl Worker {
    pub fn read(document: &[u8], digest: &[u8]) -> Result<Self, String> {
        let document = Json::parse("worker.json", document, DOCUMENT_BYTES)?;
        let Json::Array(workers) = document.member("worker.json", b"workers")? else {
            return Err("worker.json: workers must be an array".into());
        };
        let [worker] = workers.as_slice() else {
            return Err("worker.json: expected one worker".into());
        };
        let name = worker.member_string("worker.json", b"name")?;
        let Json::Number(places) = worker.member("worker.json", b"slots")? else {
            return Err("worker.json: slots must be a count".into());
        };
        let places = std::str::from_utf8(places)
            .ok()
            .and_then(|n| n.parse::<u32>().ok())
            .filter(|n| *n != 0)
            .ok_or("worker.json: slots must be a positive count")?;
        let digest = digest.strip_suffix(b"\n").unwrap_or(digest);
        if digest.len() != 64 || !digest.iter().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(b)) {
            return Err("worker.digest: expected 64 lowercase hexadecimal digits".into());
        }
        Ok(Self { name, places, digest: digest.into() })
    }
    pub fn listed(&self, bytes: &[u8]) -> Result<bool, String> {
        let Json::Array(rows) = Json::parse("worker list", bytes, DOCUMENT_BYTES)? else {
            return Err("worker list: expected an array".into());
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
                self.places.to_string().into_bytes().into(),
                b"--digest".as_slice().into(),
                self.digest.clone(),
                b"--key".as_slice().into(),
                b"qa-deployment-worker".as_slice().into(),
            ],
            Input::Null,
        )
    }
}

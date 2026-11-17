//! Worker rows and their connected host's runs (client/ctl/workers.md, 1–2).
use super::Text;
use jig_client::contract as c;
use skein_json::writer::Encoder;

fn runs(worker: &c::WorkerState, deployment: &c::Deployment) -> u64 {
    for host in &deployment.hosts {
        if host.connected && host.name.as_deref() == Some(worker.name.as_ref()) {
            return u64::from(host.runs.len());
        }
    }
    0
}
pub(super) fn text(deployment: &c::Deployment, out: &mut Text) {
    for worker in &deployment.worker_settings {
        if worker.withdrawn {
            continue;
        }
        out.put(&worker.name);
        out.put(" · ".as_bytes());
        out.number(u64::from(worker.places));
        out.put(if worker.places == 1 { b" place" } else { b" places" });
        out.put(" · ".as_bytes());
        out.put(if worker.linked { b"up" } else { b"down" });
        if let Some(since) = worker.since {
            out.put(b" since ");
            super::utc(since, out);
        }
        out.put(" · ".as_bytes());
        let count = runs(worker, deployment);
        if count == 0 {
            out.put(b"no runs");
        } else {
            out.number(count);
            out.put(if count == 1 { b" run" } else { b" runs" });
        }
        out.put(b"\n");
    }
}
pub(super) fn encode(deployment: &c::Deployment, out: &mut Encoder) {
    out.array_start();
    for worker in &deployment.worker_settings {
        if worker.withdrawn {
            continue;
        }
        out.object_start();
        out.key(b"number");
        out.unsigned(u64::from(worker.number));
        out.key(b"name");
        out.string(&worker.name);
        out.key(b"places");
        out.unsigned(u64::from(worker.places));
        out.key(b"digest");
        match worker.digest {
            Some(digest) => {
                let encoded = crate::jig::argv::value::encode(
                    &[],
                    crate::jig::table::Kind::Sha256,
                    &crate::jig::call::Value::Sha256(digest),
                )
                .expect("SHA-256 is printable");
                out.string(&encoded);
            }
            None => out.null(),
        }
        out.key(b"withdrawn");
        out.boolean(worker.withdrawn);
        out.key(b"linked");
        out.boolean(worker.linked);
        out.key(b"since");
        match worker.since {
            Some(since) => out.unsigned(since.as_nanos()),
            None => out.null(),
        }
        out.key(b"runs");
        out.unsigned(runs(worker, deployment));
        out.object_end();
    }
    out.array_end();
}

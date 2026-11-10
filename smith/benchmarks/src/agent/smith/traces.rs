//! Program accounts keep host answers separate from the spawned agent's facts.
//! Both captures are bounded by the attempt log ceiling (../../../../docs/design/benchmarks/README.md 6.2/8).

use crate::agent::Traces;
use crate::run::Archive;
use crate::{AttemptResult, Fault, bound, input, io};

impl super::Observing {
    pub(crate) fn archive_traces(
        &mut self,
        archive: Archive<'_>,
        result: &mut AttemptResult,
        maximum: usize,
    ) -> Result<(), Fault> {
        if self.prepared.program_source() {
            let mut path = self
                .prepared
                .trace
                .as_ref()
                .ok_or_else(|| input("program account requires a trace path"))?
                .as_os_str()
                .to_os_string();
            path.push(".agent");
            let captured = capture(std::path::Path::new(&path), maximum);
            if let Ok(bytes) = &captured {
                archive.put(result, "trace.jsonl.agent", &bytes[..bytes.len().min(maximum)])?;
            }
            self.agent_trace(captured.as_deref().map_err(Clone::clone).and_then(|bytes| {
                if bytes.len() > maximum { Err(bound("agent trace exceeds the attempt log bound")) } else { Ok(bytes) }
            }));
        }
        Ok(())
    }

    pub(crate) fn archive_trace(
        &self,
        archive: Archive<'_>,
        result: &mut AttemptResult,
        process: Traces<'_>,
    ) -> Result<(), Fault> {
        if !process.trace.is_empty() {
            archive.put(result, "trace.jsonl", process.trace)?;
        } else if self.prepared.program_source() {
            archive.put(result, "trace.jsonl", process.stdout)?;
        }
        Ok(())
    }
}

fn capture(path: &std::path::Path, maximum: usize) -> Result<Vec<u8>, Fault> {
    use std::io::Read;
    let file = std::fs::File::open(path).map_err(|error| io(format!("agent trace open: {error}")))?;
    let mut bytes = Vec::new();
    file.take(u64::try_from(maximum)?.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| io(format!("agent trace read: {error}")))?;
    Ok::<_, Fault>(bytes)
}

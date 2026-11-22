use sha2::{Digest, Sha256};
use skein_fake_checkout::Checkout;
use skein_lib::{Duration, Time, Wall};
use skein_world::end_to_end::{Counts, Method, PeakScope, TreeStatus};
use smith_bench::{
    Artifact, AttemptResult, End, Exit, Fault, Forced, Measure,
    grade::{Entry, Snapshot},
    parse_result,
    process::ProcessResult,
    render_result,
    run::{Request, scratch_root},
    testing::{
        batch::{DispatchFiles, dispatch_result},
        request,
    },
};
use std::{
    io,
    path::{Path, PathBuf},
};

struct Files(Checkout);

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn key(path: &Path) -> &[u8] {
    path.as_os_str().as_encoded_bytes()
}

impl DispatchFiles for Files {
    fn read_result(&self, file: &Path) -> Result<AttemptResult, Fault> {
        let bytes = self.0.content(key(file)).ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
        let text = std::str::from_utf8(bytes).expect("UTF-8 result");
        parse_result(file, text).map_err(|error| Fault::new(smith_bench::FaultKind::Format, error.to_string()))
    }

    fn exists(&self, path: &Path) -> bool {
        self.0.exists(key(path))
    }

    fn snapshot(&mut self, directory: &Path) -> Result<Snapshot, Fault> {
        Ok(self
            .0
            .tree(key(directory))
            .into_iter()
            .map(|(path, bytes)| {
                let path = PathBuf::from(String::from_utf8(path).expect("UTF-8 fixture path"));
                (path, Entry::File { sha256: digest(&bytes), executable: false })
            })
            .collect())
    }

    fn archive(&mut self, directory: &Path, name: &str, bytes: &[u8]) -> Result<Artifact, Fault> {
        let path = directory.join(name);
        assert!(!self.exists(&path), "archives are new files");
        self.0.write(key(&path), bytes);
        Ok(Artifact { path: name.into(), sha256: digest(bytes) })
    }

    fn remove_scratch(&mut self, path: &Path) -> Result<(), Fault> {
        self.0.remove(key(path));
        Ok(())
    }

    fn replace_result(&mut self, file: &Path, result: &AttemptResult) -> Result<(), Fault> {
        self.0.write(key(file), render_result(result)?.as_bytes());
        Ok(())
    }
}

fn settled() -> ProcessResult {
    ProcessResult {
        exit: Exit::Code(1),
        forced: Forced::No,
        start: Time::ZERO,
        start_wall: Wall::EPOCH,
        end: Time::from_nanos(1),
        answered: None,
        stdout: b"interrupted".to_vec(),
        stderr: Vec::new(),
        trace: Vec::new(),
        observing: None,
        error: None,
        counts: Counts {
            method: Method::Walk,
            user: Duration::from_nanos(1_000_000),
            system: Duration::ZERO,
            peak_rss_bytes: 4096,
            peak_scope: PeakScope::LargestProcess { baseline_bytes: 0 },
        },
        tree: TreeStatus {
            live_pids: Vec::new(),
            live_at_leader_exit: Some(Vec::new()),
            interrupted_pids: Vec::new(),
            terminated_pids: Vec::new(),
            killed_pids: Vec::new(),
            stopped_pids: Vec::new(),
            continued_pids: Vec::new(),
            forced_cleanup: false,
        },
    }
}

#[test]
fn missing_result_recovery_archives_evidence_and_removes_settled_scratch() {
    let mut identity = parse_result(
        Path::new("tests/recorded/results/attempt.json"),
        include_str!("../../recorded/results/attempt.json"),
    )
    .expect("result fixture")
    .identity;
    identity.arm = "a".into();
    identity.attempt = "a-0".into();
    let request = Request { settlement_ms: 30_000, ..request("tests/recorded/tasks/valid/task.toml".into(), identity) };
    let state = Path::new("state");
    let scratch = scratch_root(state, &request).expect("planned scratch");
    let directory = state.join("runs").join(&request.identity.run).join(&request.identity.attempt);
    let prepared = serde_json::to_vec_pretty(&request).expect("prepared attempt");
    let mut files = Files(Checkout::new());
    files.0.write(key(&directory.join("prepared.json")), &prepared);
    files.0.write(key(&scratch.join("transient")), b"transient");

    let recovered = dispatch_result(&mut files, state, &request, &settled(), false).expect("settled missing result");
    assert!(matches!(recovered.outcome.end, End::HarnessError { .. }));
    assert!(matches!(recovered.resources.cpu_ms, Measure::Unavailable { .. }));
    assert_eq!(
        files.0.content(key(&directory.join("dispatcher-stdout.log"))).expect("retained output"),
        b"interrupted"
    );
    assert!(!files.exists(&scratch));
    assert!(recovered.artifacts.iter().any(|artifact| {
        artifact.path == Path::new("prepared.json")
            && artifact.sha256 == digest(files.0.content(key(&directory.join(&artifact.path))).expect("archived bytes"))
    }));
}

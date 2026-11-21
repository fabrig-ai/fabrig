use sha2::{Digest, Sha256};
use smith_bench::RunSummary;
pub use smith_bench_fixtures::parity::*;

pub fn named_change(name: &str) -> (RunSummary, skein_scratch::Scratch) {
    let scratch = skein_scratch::Scratch::new("bench-parity");
    let directory = scratch.path().to_path_buf().join(format!("smith-change-{name}-{}", std::process::id()));
    std::fs::create_dir(&directory).expect("temporary input directory");
    let baseline = directory.join("baseline.json");
    smith_bench::write_summary(&baseline, &change_baseline()).expect("baseline input");
    let mut summary = change_summary();
    let report = summary.change.as_mut().expect("recorded section");
    report.baseline_run = Some(smith_bench::parity::change::Input {
        sha256: Some(format!("{:x}", Sha256::digest(std::fs::read(&baseline).expect("baseline bytes")))),
        path: baseline,
    });
    for projection in &mut report.projection {
        projection.baseline_interval = (0.9, 1.1);
    }
    (summary, scratch)
}

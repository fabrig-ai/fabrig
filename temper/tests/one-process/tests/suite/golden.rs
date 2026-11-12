//! The baseline's making story and its permanent upgrade reader.
#[test]
fn baseline_making_story_stops_a_whole_deployment() {
    let directory = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("baseline-making-story");
    if directory.exists() {
        std::fs::remove_dir_all(&directory).expect("previous temporary export");
    }
    temper_one_process_world::golden::make(&directory);
    let archive = directory.with_extension("tar.gz");
    assert!(std::process::Command::new("tar").args(["-czf"]).arg(&archive).args(["-C"]).arg(&directory).arg(".").status().expect("tar").success());
    let sha = std::process::Command::new("sha256sum").arg(&archive).output().expect("sha256sum");
    println!("ARCHIVE_SHA {}", String::from_utf8_lossy(&sha.stdout));
    let bytes = std::process::Command::new("base64").arg(&archive).output().expect("base64");
    println!("ARCHIVE_BEGIN\n{}ARCHIVE_END", String::from_utf8_lossy(&bytes.stdout));
}

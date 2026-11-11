//! The baseline's making story and its permanent upgrade reader.
#[test]
fn baseline_making_story_stops_a_whole_deployment() {
    let directory = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("baseline-making-story");
    if directory.exists() {
        std::fs::remove_dir_all(&directory).expect("previous temporary export");
    }
    temper_one_process_world::golden::make(&directory);
}

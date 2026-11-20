//! The floor's making story and the permanent deployment readers.
#[test]
fn dev_image_opens_and_serves_on_this_build() {
    temper_one_process_world::golden::read_dev(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("golden/dev"));
}

#[test]
fn floor_golden_store_opens_and_serves_the_kept_person() {
    temper_one_process_world::golden::read(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("golden/floor"));
}

#[test]
fn floor_making_story_stops_a_whole_deployment() {
    let directory = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("floor-making-story");
    if directory.exists() {
        std::fs::remove_dir_all(&directory).expect("previous temporary export");
    }
    temper_one_process_world::golden::make(&directory);
}

#[test]
fn floorless_golden_store_is_refused_without_writing() {
    temper_one_process_world::golden::refuse(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("golden/floor"));
}

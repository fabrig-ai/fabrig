//! Landed QA's real-file, socket, lock and process checks (qa/README.md, 6).
// APFS, macOS's file system, holds no file name that is not UTF-8 (temper/docs/design/testing/macos.md, 3).
#[cfg(target_os = "linux")]
mod build_files;
mod builds;
mod events;
mod hold;
mod place;
mod run;
mod run_passes;
mod world;

use temper_qa_tool::testing::passes;

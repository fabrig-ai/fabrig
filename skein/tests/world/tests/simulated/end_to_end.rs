use skein_io::kernel::{Exit, Signal};
use skein_lib::{Duration, Time};
use skein_world::Referee;
use skein_world::end_to_end::Cleanup;
use skein_world_tests::supervised::{SupervisedJudge, SupervisedSnapshot, SupervisedTrigger};

#[cfg(target_os = "linux")]
#[path = "../suite/supervised_referee.rs"]
mod referee_steps;

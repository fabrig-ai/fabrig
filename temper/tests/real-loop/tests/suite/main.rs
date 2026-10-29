//! Real-loop integration scenarios (temper/docs/design/domain/testing.md, 2.4).
use skein_world::domain::heap::Counting;

#[global_allocator]
static HEAP: Counting = Counting;

mod account_key;
mod case_5_3;
mod case_traffic;
mod cluster;
mod forge;
mod git;
mod github;
mod github_land;
mod landing;
mod landing_retry;
mod lifecycle;
mod lock;
mod lost;
mod peers;
mod restart;
mod sign_in;
mod template_bounds;
mod terminal;
mod tools;

mod fuzzy {
    mod case2_8_1;
    mod case2_hour_0;
    mod case_4_1_4_3;
    mod case_4_4;
    mod case_4_5_limited;
    mod case_4_5_refused;
    mod case_4_6;
    mod case_5_1;
    mod case_5_2;
    mod case_5_4;
    mod case_5_5;
    mod case_5_6;
    mod case_6_1;
    mod case_6_2;
    mod case_6_4;
    mod case_6_5;
    mod case_6_6;
    mod case_6_7;
    mod case_6_8;
    mod green_light;
    mod landing;
    mod program_landing;
    mod repair;
}

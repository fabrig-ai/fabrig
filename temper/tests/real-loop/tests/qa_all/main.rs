//! QA real-loop scenarios (temper/docs/design/domain/testing.md, 2.4).
use skein_world::domain::heap::Counting;

#[global_allocator]
static HEAP: Counting = Counting;

mod qa_added_deps;
mod qa_adoption_hold;
mod qa_case_4_1;
mod qa_case_act2;
mod qa_case_act3;
mod qa_github_connect;
mod qa_github_land;
mod qa_github_loop;
mod qa_live_text;
mod qa_plan_chat;
mod qa_sign_in_paste;

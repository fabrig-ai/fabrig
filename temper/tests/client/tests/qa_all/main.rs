use skein_world::domain::heap::Counting;

#[global_allocator]
static HEAP: Counting = Counting;

mod qa_accounts;
mod qa_change_page;
mod qa_change_panel;
mod qa_connect_model;
mod qa_first_repository;
mod qa_first_run_hooks;
mod qa_forges;
mod qa_github_connect;
mod qa_live_text;
#[path = "../qa_model/mod.rs"]
mod qa_model;
mod qa_question;
mod qa_sign_in;
mod qa_sign_in_paste;
mod qa_task_pages;
mod qa_welcome;
mod qa_welcome_m3;
mod qa_welcome_m3_owned;
mod qa_welcome_m4;

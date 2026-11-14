mod census;
mod lists;
mod notes;
mod reads;
mod turn;

mod acts;
mod conditions;

mod digest;

mod pending;
mod wait;

mod account_waits;
mod accounts;
mod catalogue_windows;

mod account_acts;

mod choices;

mod chat_template;

mod amend_choice;

mod task_words;

mod amend_controls;

mod settings;

mod task_settings;

mod projects;

mod replays;

mod specification;

mod goal_template;
mod party;
mod restore;
mod role;

mod project_delete;
mod visibility;

fn malformed_setting(
    setting: jig_client::contract::Setting,
    fault: jig_client::contract::SettingFault,
) -> jig_client::contract::Malformed {
    jig_client::contract::Malformed::Settings(jig_client::contract::MalformedSettings { setting, fault, detail: None })
}

mod need;

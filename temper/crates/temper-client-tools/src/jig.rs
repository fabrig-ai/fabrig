//! The generic tools vocabulary.
pub mod key;
pub mod table;
pub use table::Limits;
pub mod argv;
pub mod call;
pub mod json;
pub mod render;
pub mod rows;
pub mod runner;
pub mod subject;
pub use runner::{FileEnd, Printed};

pub mod conditions;
pub mod digest;
pub mod help;

pub mod wait;

mod settings_adapter;
#[must_use]
pub fn form_setting_value(value: &jig_client::contract::SettingValue) -> settings_form::view::SettingValue {
    settings_adapter::form_setting_value(value)
}
pub(crate) use settings_adapter::limits as form_limits;
pub mod settings_file;

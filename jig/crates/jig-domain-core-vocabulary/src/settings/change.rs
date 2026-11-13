//! Settings edit shape admission (domain/engine/settings.md, 1–2).
use super::{Edit, Failure, Fault, Item, Setting};
use crate::Limits;

pub fn change_shape(items: &[Item], limits: &Limits) -> Result<(), Failure> {
    let bound = super::capacity(limits)
        .expect("settings capacity")
        .checked_mul(2)
        .expect("item union")
        .checked_add(1)
        .expect("one project policy");
    let mut jig = 0_u32;
    for item in items {
        if !super::application::is_application(item.setting) {
            jig = jig.checked_add(1).expect("bounded item count");
            if jig > bound {
                return Err(Failure { setting: item.setting, fault: Fault::Count, detail: None });
            }
        }
    }
    let mut counts = [0_u32; 3];
    let mut project_policy = false;
    for (at, item) in items.iter().enumerate() {
        match item.setting {
            Setting::ProjectPolicy { project } => {
                if project == 0 || project_policy {
                    return Err(Failure { setting: item.setting, fault: Fault::Shape, detail: None });
                }
                match &item.edit {
                    Edit::Set(super::Value::Policy(_)) => {}
                    Edit::Set(_) | Edit::Removed | Edit::Reset => {
                        return Err(Failure { setting: item.setting, fault: Fault::Shape, detail: None });
                    }
                }
                project_policy = true;
            }
            Setting::Model(0)
            | Setting::Charter(0)
            | Setting::Template(0)
            | Setting::Policy(0)
            | Setting::Application { number: 0, .. } => {
                return Err(Failure { setting: item.setting, fault: Fault::Shape, detail: None });
            }
            Setting::Model(number) if number > jig_domain_tasks::MODELS => {
                return Err(Failure { setting: item.setting, fault: Fault::Shape, detail: None });
            }
            Setting::Model(_)
            | Setting::Charter(_)
            | Setting::Template(_)
            | Setting::Policy(_)
            | Setting::Rules
            | Setting::Application { .. } => {}
        }
        for earlier in items.iter().take(at) {
            if item.setting == earlier.setting {
                return Err(Failure { setting: item.setting, fault: Fault::Shape, detail: None });
            }
        }
        match &item.edit {
            Edit::Set(value) => {
                super::check::shape(item.setting, value, limits)?;
                super::check::count(item.setting, &mut counts, limits)?;
            }
            Edit::Removed if item.setting == Setting::Rules => {
                return Err(Failure { setting: item.setting, fault: Fault::Shape, detail: None });
            }
            Edit::Removed | Edit::Reset => {}
        }
    }
    Ok(())
}

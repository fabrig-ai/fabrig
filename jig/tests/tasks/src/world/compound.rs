use super::World;
use jig_domain_tasks::Operation;
use jig_domain_tasks::{self as tasks, Cancel, Release};
use skein_lib::List;
impl World {
    pub(super) fn remember_compound(&mut self, person: u64, period: u64, operations: &List<Operation>) {
        for operation in operations {
            match operation {
                Operation::Move(tasks::Move { task, .. }) => self.accounting_referee.moving(*task, person, period),
                Operation::Cancel(Cancel { message: Some(number), .. })
                | Operation::Release(Release { message: Some(number), .. })
                | Operation::Amend(tasks::Amend { message: number, .. }) => self.message = self.message.max(*number),
                Operation::Message(tasks::Message { word, .. }) => self.message = self.message.max(word.number),
                Operation::Stop { .. }
                | Operation::Cancel(Cancel { message: None, .. })
                | Operation::Release(Release { message: None, .. }) => {}
            }
        }
    }
}

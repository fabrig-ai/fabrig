//! Judging, keeping and holding the forge's kinds
//! (temper/docs/design/protocol/configuration.md, 13.2;
//! temper/docs/design/domain/forge.md, 8.9 and section 16).
use alloc::boxed::Box;
use skein_lib::{Env, List, Map, Queue};

use crate::settings::{self, Edit, SettingKey, SettingRow, SettingSlot, Value};
use crate::{Domain, ForgeCharters, Key, Limits, Request, Stored};

impl Domain {
    #[must_use]
    #[expect(clippy::manual_map, reason = "the step subset uses exhaustive matches rather than closures")]
    pub fn judge_settings(&self, bounds: &settings::Bounds, items: &[settings::Asked]) -> Box<[settings::Judged]> {
        let mut judged = List::uncapped();
        for item in items {
            let key = SettingKey { kind: item.kind, scope: item.scope, number: item.number, slot: SettingSlot::Change };
            let value = match &item.edit {
                Edit::Set(value) => Some(value),
                Edit::Remove => None,
                Edit::Reset => self.setting_default(item.kind, item.scope, item.number),
            };
            let check = settings::Item {
                kind: item.kind,
                scope: item.scope,
                number: item.number,
                value: match value {
                    Some(value) => Some(value.value.clone()),
                    None => None,
                },
            };
            let answer = match settings::judge(&check, bounds) {
                Err(refusal) => settings::Judged::Refuse(refusal),
                Ok(()) => {
                    let same = match &item.edit {
                        Edit::Set(_) | Edit::Remove => self.setting_rows.get(&key) == Some(&item.edit),
                        Edit::Reset => !self.setting_rows.contains_key(&key),
                    };
                    if same {
                        settings::Judged::Same
                    } else {
                        let (size, charter) = value_state(value);
                        settings::Judged::Accept { size, charter }
                    }
                }
            };
            judged.push(answer).expect("one judgement per item");
        }
        judged.into_boxed()
    }

    pub fn keep_settings(
        &mut self,
        env: &Env<Limits>,
        revision: u64,
        proposal: Option<u64>,
        items: &[settings::Asked],
        out: &mut Queue<Request>,
    ) {
        let before = core::mem::replace(
            &mut self.held_rules,
            settings::HeldRules { deployment: Box::new([]), projects: Map::with_capacity(env.limits.judge_projects) },
        );
        for item in items {
            let key = SettingKey { kind: item.kind, scope: item.scope, number: item.number, slot: SettingSlot::Change };
            match self.setting_rows.get(&key) {
                Some(Edit::Set(value)) => {
                    let edit = Edit::Set(value.clone());
                    self.save_setting(SettingKey { slot: SettingSlot::Replaced(revision), ..key }, edit, out);
                }
                Some(Edit::Remove | Edit::Reset) | None => {}
            }
            let edit = match proposal {
                Some(proposal) => self.setting_rows.get(&SettingKey { slot: SettingSlot::Proposal(proposal), ..key }),
                None => None,
            };
            let edit = edit.unwrap_or(&item.edit).clone();
            match edit {
                Edit::Set(_) | Edit::Remove => self.save_setting(key, edit, out),
                Edit::Reset => self.erase_setting(key, out),
            }
        }
        if let Some(proposal) = proposal {
            self.drop_proposal_settings(proposal, out);
        }
        self.rebuild_settings();
        let deployment_changed = before.deployment != self.held_rules.deployment;
        let mut wake = List::with_capacity(self.changes.capacity());
        for (task, row) in &self.changes {
            if row.terminal() {
                continue;
            }
            let Some(repository) = self.repositories.get(&row.repository) else { continue };
            if deployment_changed
                || before.projects.get(&repository.project) != self.held_rules.projects.get(&repository.project)
            {
                wake.push(*task).expect("one wake per live change");
            }
        }
        for task in &wake {
            self.wait_change(env, *task, Some(env.wall));
        }
    }

    pub fn keep_proposal_settings(&mut self, proposal: u64, items: &[settings::Asked], out: &mut Queue<Request>) {
        for item in items {
            let key = SettingKey {
                kind: item.kind,
                scope: item.scope,
                number: item.number,
                slot: SettingSlot::Proposal(proposal),
            };
            self.save_setting(key, item.edit.clone(), out);
        }
        self.rebuild_settings();
    }

    pub fn drop_proposal_settings(&mut self, proposal: u64, out: &mut Queue<Request>) {
        for (key, _) in &self.proposal_settings(proposal) {
            self.erase_setting(*key, out);
        }
        self.rebuild_settings();
    }

    pub fn give_settings_defaults(&mut self, stock: Box<[settings::Stock]>) -> Box<[settings::DefaultState]> {
        let mut states = List::uncapped();
        for item in &stock {
            let key =
                SettingKey { kind: item.kind, scope: item.scope, number: item.number, slot: SettingSlot::Default };
            let (size, charter) = value_state(Some(&item.value));
            states
                .push(settings::DefaultState {
                    kind: item.kind,
                    scope: item.scope,
                    number: item.number,
                    size,
                    charter,
                    differs: self.setting_rows.get(&key) != Some(&Edit::Set(item.value.clone())),
                })
                .expect("one state per stock entry");
        }
        self.setting_stock = Some(stock);
        self.rebuild_settings();
        states.into_boxed()
    }

    pub fn save_settings_defaults(&mut self, out: &mut Queue<Request>) {
        let mut saves = List::with_capacity(self.setting_rows.capacity());
        if let Some(stock) = &self.setting_stock {
            for item in stock {
                let key =
                    SettingKey { kind: item.kind, scope: item.scope, number: item.number, slot: SettingSlot::Default };
                let edit = Edit::Set(item.value.clone());
                if self.setting_rows.get(&key) != Some(&edit) {
                    saves.push((key, edit)).expect("stock within setting bounds");
                }
            }
        }
        let mut erases = List::with_capacity(self.setting_rows.capacity());
        for (key, _) in &self.setting_rows {
            if key.slot == SettingSlot::Default && self.setting_default(key.kind, key.scope, key.number).is_none() {
                erases.push(*key).expect("bounded default rows");
            }
        }
        for key in &erases {
            self.erase_setting(*key, out);
        }
        for (key, edit) in saves.into_boxed() {
            self.save_setting(key, edit, out);
        }
        self.rebuild_settings();
    }

    pub fn erase_project_settings(&mut self, project: u32, out: &mut Queue<Request>) {
        let mut erases = List::with_capacity(self.setting_rows.capacity());
        for (key, _) in &self.setting_rows {
            if key.kind == 2 && key.scope == project {
                erases.push(*key).expect("bounded project setting rows");
            }
        }
        for key in &erases {
            self.erase_setting(*key, out);
        }
        self.rebuild_settings();
        self.held_rules.projects.remove(&project);
    }

    #[must_use]
    pub const fn charters(&self) -> ForgeCharters {
        self.held_charters
    }

    #[must_use]
    pub fn landing_rules(&self, project: u32) -> (&[settings::Held], &[settings::Held]) {
        let project = match self.held_rules.projects.get(&project) {
            Some(rules) => rules.as_ref(),
            None => &[],
        };
        (&self.held_rules.deployment, project)
    }

    #[must_use]
    pub fn setting(&self, kind: u32, scope: u32, number: u32) -> Option<&settings::SizedValue> {
        let key = SettingKey { kind, scope, number, slot: SettingSlot::Change };
        match self.setting_rows.get(&key) {
            Some(Edit::Set(value)) => Some(value),
            Some(Edit::Remove | Edit::Reset) => None,
            None => self.setting_default(kind, scope, number),
        }
    }

    #[must_use]
    pub fn proposal_settings(&self, proposal: u64) -> Box<[(SettingKey, Edit)]> {
        let mut items = List::with_capacity(self.setting_rows.capacity());
        for (key, edit) in &self.setting_rows {
            if key.slot == SettingSlot::Proposal(proposal) {
                items.push((*key, edit.clone())).expect("bounded proposal rows");
            }
        }
        items.into_boxed()
    }

    fn setting_default(&self, kind: u32, scope: u32, number: u32) -> Option<&settings::SizedValue> {
        match &self.setting_stock {
            Some(stock) => {
                for item in stock {
                    if (item.kind, item.scope, item.number) == (kind, scope, number) {
                        return Some(&item.value);
                    }
                }
                None
            }
            None => {
                let key = SettingKey { kind, scope, number, slot: SettingSlot::Default };
                match self.setting_rows.get(&key) {
                    Some(Edit::Set(value)) => Some(value),
                    Some(Edit::Remove | Edit::Reset) | None => None,
                }
            }
        }
    }

    fn save_setting(&mut self, key: SettingKey, edit: Edit, out: &mut Queue<Request>) {
        self.setting_rows.insert(key, edit.clone()).expect("settings checked by jig fit the forge's rows");
        out.push(Request::Save { record: Stored::Setting(SettingRow { key, edit }), release: None, read_afresh: None });
    }

    fn erase_setting(&mut self, key: SettingKey, out: &mut Queue<Request>) {
        if self.setting_rows.remove(&key).is_some() {
            out.push(Request::Erase { key: Key::Setting(key) });
        }
    }

    pub(crate) fn rebuild_settings(&mut self) {
        let mut keys = Map::with_capacity(self.setting_rows.capacity());
        for (key, _) in &self.setting_rows {
            match key.slot {
                SettingSlot::Default | SettingSlot::Change => {
                    keys.insert((key.kind, key.scope, key.number), ()).expect("bounded current settings");
                }
                SettingSlot::Proposal(_) | SettingSlot::Replaced(_) => {}
            }
        }
        if let Some(stock) = &self.setting_stock {
            for item in stock {
                keys.insert((item.kind, item.scope, item.number), ()).expect("stock within setting bounds");
            }
        }
        let mut deployment = List::uncapped();
        let mut projects = Map::<u32, List<settings::Held>>::with_capacity(self.held_rules.projects.capacity());
        for ((kind, scope, number), ()) in &keys {
            let Some(value) = self.setting(*kind, *scope, *number) else { continue };
            if let Value::Rule(rule) = &value.value {
                let held = settings::Held { scope: *scope, number: *number, rule: rule.clone() };
                match *kind {
                    1 => deployment.push(held).expect("bounded deployment rules"),
                    2 => {
                        if !projects.contains_key(scope) {
                            projects.insert(*scope, List::uncapped()).expect("project rules checked by jig fit");
                        }
                        projects.get_mut(scope).expect("project inserted").push(held).expect("bounded project rules");
                    }
                    _ => unreachable!("rule belongs to kind 1 or 2"),
                }
            }
        }
        self.held_rules.deployment = deployment.into_boxed();
        self.held_rules.projects = Map::with_capacity(projects.capacity());
        for _ in 0..projects.len() {
            let project = *projects.first().expect("counted project").0;
            let rules = projects.remove(&project).expect("counted project rules");
            self.held_rules.projects.insert(project, rules.into_boxed()).expect("bounded project rules");
        }
        self.held_charters = ForgeCharters { change: self.setting_charter(1), review: self.setting_charter(2) };
    }

    fn setting_charter(&self, number: u32) -> u32 {
        match self.setting(3, 0, number) {
            Some(settings::SizedValue { value: Value::Charter(charter), .. }) => *charter,
            Some(settings::SizedValue { value: Value::Rule(_), .. }) | None => 0,
        }
    }
}

fn value_state(value: Option<&settings::SizedValue>) -> (u32, Option<u32>) {
    match value {
        Some(settings::SizedValue { value: Value::Charter(charter), size }) => (*size, Some(*charter)),
        Some(settings::SizedValue { value: Value::Rule(_), size }) => (*size, None),
        None => (0, None),
    }
}

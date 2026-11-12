//! Judging, keeping and holding the forge's kinds
//! (temper/docs/design/protocol/configuration/settings.md, 2;
//! temper/docs/design/domain/forge/change/gates.md, 3, and temper/docs/design/domain/forge/README.md, 9).
use alloc::boxed::Box;
use jig_domain_core::settings::application::{Restored, RestoredEdit};
use skein_lib::{Env, List, Map, Queue};

use crate::settings::{self, Edit, SettingKey, SettingRow, SettingSlot, TemplateGoal, Value};
use crate::{Domain, ForgeCharters, Key, Limits, Request, Stored};

impl Domain {
    #[must_use]
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
        let mut before_classes = Map::with_capacity(env.limits.judge_projects);
        for item in items {
            if item.kind == 8 && !before_classes.contains_key(&item.scope) {
                before_classes.insert(item.scope, self.project_classes(item.scope).sets).expect("bounded projects");
            }
        }
        let before = core::mem::replace(
            &mut self.held_rules,
            settings::HeldRules {
                deployment: Box::new([]),
                every_project: Box::new([]),
                projects: Map::with_capacity(env.limits.judge_projects),
            },
        );
        for item in items {
            self.keep_setting(revision, proposal, item, out);
        }
        if let Some(proposal) = proposal {
            self.drop_proposal_settings(proposal, out);
        }
        self.rebuild_settings();
        for item in items {
            if item.kind == 13 {
                out.push(Request::Marked { tasks: Box::new([crate::follows_subject(item.scope)]) });
            }
            if item.kind == 12 {
                crate::follows::keep(self, item.scope, item.number, out);
            }
        }
        let mut wake = List::with_capacity(self.changes.capacity());
        for (task, row) in &self.changes {
            if row.terminal() {
                continue;
            }
            let Some(repository) = self.repositories.get(&row.repository) else { continue };
            let rules = self.landing_rules(repository.project);
            let classes_changed = match before_classes.get(&repository.project) {
                Some(sets) => *sets != self.project_classes(repository.project).sets,
                None => false,
            };
            let mut by_path = false;
            if classes_changed {
                for held in &rules {
                    by_path |=
                        !held.rule.sets.is_empty() && crate::covers_branch(&held.rule.pattern, repository, &row.base);
                }
            }
            if held_for_project(&before, repository.project) != rules || by_path {
                wake.push(*task).expect("one wake per live change");
            }
        }
        for task in &wake {
            self.wait_change(env, *task, Some(env.wall));
        }
    }

    fn keep_setting(&mut self, revision: u64, proposal: Option<u64>, item: &settings::Asked, out: &mut Queue<Request>) {
        let key = SettingKey { kind: item.kind, scope: item.scope, number: item.number, slot: SettingSlot::Change };
        let before = match self.setting_rows.get(&key) {
            Some(Edit::Set(value)) => Some(value),
            Some(Edit::Remove | Edit::Reset) => None,
            None => self.setting_default(item.kind, item.scope, item.number),
        };
        if let Some(value) = before {
            let edit = Edit::Set(value.clone());
            self.save_setting(SettingKey { slot: SettingSlot::Replaced(revision), ..key }, edit, out);
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

    #[must_use]
    pub fn restore_settings(
        &mut self,
        bounds: &settings::Bounds,
        proposal: u64,
        items: &[Restored],
        out: &mut Queue<Request>,
    ) -> Box<[settings::Judged]> {
        let mut judged = List::uncapped();
        let mut asked = List::uncapped();
        for item in items {
            let edit = match item.edit {
                RestoredEdit::Set { revision } => {
                    let key = SettingKey {
                        kind: item.kind,
                        scope: item.scope,
                        number: item.number,
                        slot: SettingSlot::Replaced(revision),
                    };
                    if self.refused_settings.contains_key(&key) {
                        judged
                            .push(settings::Judged::Refuse(settings::Refusal::Retired))
                            .expect("one judgement per item");
                        continue;
                    }
                    match self.setting_rows.get(&key) {
                        Some(Edit::Set(value)) => Edit::Set(value.clone()),
                        Some(Edit::Remove | Edit::Reset) | None => {
                            judged
                                .push(settings::Judged::Refuse(settings::Refusal::Replaced))
                                .expect("one judgement per item");
                            continue;
                        }
                    }
                }
                RestoredEdit::Remove => Edit::Remove,
                RestoredEdit::Reset => Edit::Reset,
            };
            let value = match &edit {
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
                Err(why) => settings::Judged::Refuse(why),
                Ok(()) => {
                    let (size, charter) = value_state(value);
                    asked
                        .push(settings::Asked { kind: item.kind, scope: item.scope, number: item.number, edit })
                        .expect("one kept edit per accepted item");
                    settings::Judged::Accept { size, charter }
                }
            };
            judged.push(answer).expect("one judgement per item");
        }
        self.keep_proposal_settings(proposal, &asked.into_boxed(), out);
        judged.into_boxed()
    }

    pub fn forget_replaced(&mut self, before: u64, out: &mut Queue<Request>) {
        let mut erases = List::with_capacity(self.setting_rows.capacity());
        for (key, _) in &self.setting_rows {
            if let SettingSlot::Replaced(revision) = key.slot
                && revision < before
            {
                erases.push(*key).expect("bounded replaced rows");
            }
        }
        for (key, ()) in &self.refused_settings {
            if let SettingSlot::Replaced(revision) = key.slot
                && revision < before
            {
                erases.push(*key).expect("bounded refused setting identities");
            }
        }
        for key in &erases {
            self.erase_setting(*key, out);
        }
    }

    pub fn drop_proposal_settings(&mut self, proposal: u64, out: &mut Queue<Request>) {
        for (key, _) in &self.proposal_settings(proposal) {
            self.erase_setting(*key, out);
        }
        let mut refused = List::with_capacity(self.refused_settings.capacity());
        for (key, ()) in &self.refused_settings {
            if key.slot == SettingSlot::Proposal(proposal) {
                refused.push(*key).expect("bounded refused proposal identities");
            }
        }
        for key in &refused {
            self.erase_setting(*key, out);
        }
        self.rebuild_settings();
    }

    pub fn give_settings_defaults(&mut self, stock: Box<[settings::Stock]>) -> Box<[settings::DefaultState]> {
        let mut states = List::uncapped();
        for item in &stock {
            if item.kind == 2 && item.scope == 0 {
                continue;
            }
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

    /// The stored defaults the stock last given no longer holds, kind 2's
    /// every-project entries (scope 0) left out as `give_settings_defaults`
    /// leaves them out (temper/docs/design/protocol/configuration/stock.md).
    #[must_use]
    pub fn dropped_settings_defaults(&self) -> Box<[(u32, u32, u32)]> {
        let mut dropped = List::with_capacity(self.setting_rows.capacity());
        for (key, _) in &self.setting_rows {
            if key.slot == SettingSlot::Default
                && !(key.kind == 2 && key.scope == 0)
                && self.setting_default(key.kind, key.scope, key.number).is_none()
            {
                dropped.push((key.kind, key.scope, key.number)).expect("one name per stored default");
            }
        }
        dropped.into_boxed()
    }

    /// Preserve changed defaults before a release replaces them (temper/docs/design/client/forge/served.md).
    pub fn remember_settings_defaults(&mut self, revision: u64, out: &mut Queue<Request>) {
        let mut replaced = List::uncapped();
        for (key, edit) in &self.setting_rows {
            if key.slot != SettingSlot::Default {
                continue;
            }
            let change = SettingKey { slot: SettingSlot::Change, ..*key };
            if !self.setting_rows.contains_key(&change)
                && let Edit::Set(value) = edit
                && self.setting_default(key.kind, key.scope, key.number) != Some(value)
            {
                replaced
                    .push((SettingKey { slot: SettingSlot::Replaced(revision), ..*key }, Edit::Set(value.clone())))
                    .expect("one replaced value per default setting");
            }
        }
        for (key, edit) in replaced.into_boxed() {
            self.save_setting(key, edit, out);
        }
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
            if (key.kind == 2
                || key.kind == 6
                || key.kind == 7
                || key.kind == 8
                || key.kind == 10
                || key.kind == 12
                || key.kind == 13)
                && key.scope == project
            {
                erases.push(*key).expect("bounded project setting rows");
            }
        }
        for key in &erases {
            self.erase_setting(*key, out);
        }
        self.rebuild_settings();
        self.held_rules.projects.remove(&project);
    }

    /// Kind 6 of a project, or no route with none stored
    /// (temper/docs/design/protocol/configuration/settings.md, 2 and temper/docs/design/protocol/configuration/stock.md).
    #[must_use]
    pub fn project_questions(&self, project: u32) -> temper_engine_domain_forge_change::Route {
        use temper_engine_domain_forge_change::Route;
        match self.setting(6, project, 1) {
            Some(settings::SizedValue { value: Value::Questions(route), .. }) => *route,
            Some(settings::SizedValue {
                value:
                    Value::Goal(_)
                    | Value::Dependency(_)
                    | Value::Follow(_)
                    | Value::Rule(_)
                    | Value::Charter(_)
                    | Value::Ways(_)
                    | Value::CharterWords(_)
                    | Value::Classes(_)
                    | Value::ProjectWords(_),
                ..
            })
            | None => Route::None,
        }
    }

    /// Kind 8 of a project, or Agents review's stock table with none held
    /// (temper/docs/design/protocol/configuration/settings.md, 2 and temper/docs/design/protocol/configuration/stock.md).
    #[must_use]
    pub fn project_classes(&self, project: u32) -> temper_protocol_reaches::Classes {
        match self.setting(8, project, 1) {
            Some(settings::SizedValue { value: Value::Classes(classes), .. }) => classes.clone(),
            Some(settings::SizedValue {
                value:
                    Value::Dependency(_)
                    | Value::Follow(_)
                    | Value::Rule(_)
                    | Value::Charter(_)
                    | Value::Questions(_)
                    | Value::Ways(_)
                    | Value::Goal(_)
                    | Value::CharterWords(_)
                    | Value::ProjectWords(_),
                ..
            })
            | None => temper_protocol_reaches::Classes::stock(temper_protocol_reaches::Way::AgentsReview),
        }
    }

    /// A charter's words as held: its change, else the stock's; none where removed or never given
    /// (temper/docs/design/protocol/configuration/settings.md, 2).
    #[must_use]
    pub fn charter_words(&self, charter: u32) -> Option<&[u8]> {
        match self.setting(9, 0, charter) {
            Some(settings::SizedValue { value: Value::CharterWords(line), .. }) => Some(line),
            Some(settings::SizedValue {
                value:
                    Value::Goal(_)
                    | Value::Dependency(_)
                    | Value::Follow(_)
                    | Value::Rule(_)
                    | Value::Charter(_)
                    | Value::Questions(_)
                    | Value::Ways(_)
                    | Value::Classes(_)
                    | Value::ProjectWords(_),
                ..
            })
            | None => None,
        }
    }

    /// A project's words as held, none with none stored (temper/docs/design/protocol/configuration/settings.md, 2).
    #[must_use]
    pub fn project_words(&self, project: u32) -> Option<&settings::ProjectWords> {
        match self.setting(10, project, 1) {
            Some(settings::SizedValue { value: Value::ProjectWords(words), .. }) => Some(words),
            Some(settings::SizedValue {
                value:
                    Value::Goal(_)
                    | Value::Dependency(_)
                    | Value::Follow(_)
                    | Value::Rule(_)
                    | Value::Charter(_)
                    | Value::Questions(_)
                    | Value::Ways(_)
                    | Value::CharterWords(_)
                    | Value::Classes(_),
                ..
            })
            | None => None,
        }
    }

    /// A goal template's work as held: its change, else the stock's; none where removed
    /// or never given (temper/docs/design/protocol/configuration/settings.md, 2).
    #[must_use]
    pub fn template_goal(&self, template: u32) -> Option<TemplateGoal> {
        match self.setting(11, 0, template) {
            Some(settings::SizedValue { value: Value::Goal(goal), .. }) => Some(*goal),
            Some(settings::SizedValue {
                value:
                    Value::Dependency(_)
                    | Value::Follow(_)
                    | Value::Rule(_)
                    | Value::Charter(_)
                    | Value::Questions(_)
                    | Value::Ways(_)
                    | Value::CharterWords(_)
                    | Value::Classes(_)
                    | Value::ProjectWords(_),
                ..
            })
            | None => None,
        }
    }

    /// Current dependencies in number order (temper/docs/design/domain/forge/dependencies.md, 4).
    #[must_use]
    pub fn dependencies(&self, project: u32) -> Box<[(u32, &settings::Dependency)]> {
        let mut dependencies = List::uncapped();
        for (key, edit) in &self.setting_rows {
            if key.kind == 13
                && key.scope == project
                && key.slot == SettingSlot::Change
                && let Edit::Set(settings::SizedValue { value: Value::Dependency(value), .. }) = edit
            {
                dependencies.push((key.number, value)).expect("one current value per dependency");
            }
        }
        dependencies.into_boxed()
    }

    /// Current kind 12, with no stock default (temper/docs/design/domain/forge/follows.md, 2).
    #[must_use]
    pub fn follow(&self, project: u32, number: u32) -> Option<&settings::Follow> {
        let key = SettingKey { kind: 12, scope: project, number, slot: SettingSlot::Change };
        if let Some(Edit::Set(settings::SizedValue { value: Value::Follow(follow), .. })) = self.setting_rows.get(&key)
        {
            Some(follow)
        } else {
            None
        }
    }

    /// Current follows in number order, excluding proposals and replaced values
    /// (temper/docs/design/domain/forge/follows.md, 2).
    #[must_use]
    pub fn follows(&self, project: u32) -> Box<[(u32, &settings::Follow)]> {
        let mut follows = List::uncapped();
        for (key, edit) in &self.setting_rows {
            if key.kind == 12
                && key.scope == project
                && key.slot == SettingSlot::Change
                && let Edit::Set(settings::SizedValue { value: Value::Follow(follow), .. }) = edit
            {
                follows.push((key.number, follow)).expect("one value per current follow");
            }
        }
        follows.into_boxed()
    }

    #[must_use]
    pub const fn charters(&self) -> ForgeCharters {
        self.held_charters
    }

    /// Given rules used when a project's hypothetical edit removes its own rule
    /// (temper/docs/design/protocol/configuration/stock.md).
    #[must_use]
    pub fn every_project_rules(&self) -> &[settings::Held] {
        &self.held_rules.every_project
    }

    /// Every rule a merge into a repository of `project` meets: the deployment's,
    /// then the project's by number, its rule N the every-project rule N while
    /// the project has no value of its own set (temper/docs/design/protocol/configuration/stock.md).
    #[must_use]
    pub fn landing_rules(&self, project: u32) -> Box<[settings::Held]> {
        held_for_project(&self.held_rules, project)
    }

    #[must_use]
    pub fn setting(&self, kind: u32, scope: u32, number: u32) -> Option<&settings::SizedValue> {
        let key = SettingKey { kind, scope, number, slot: SettingSlot::Change };
        match self.setting_rows.get(&key) {
            Some(Edit::Set(value)) => Some(value),
            Some(Edit::Remove | Edit::Reset) => self.every_project_default(kind, scope, number),
            None => self.setting_default(kind, scope, number),
        }
    }

    /// Read a set item's value before its next overwrite (temper/docs/design/client/forge/served.md).
    /// The caller verifies that the retained revision set this item.
    #[must_use]
    pub fn revision_setting(&self, kind: u32, scope: u32, number: u32, revision: u32) -> Option<&settings::SizedValue> {
        let mut next = None;
        for (key, edit) in &self.setting_rows {
            if (key.kind, key.scope, key.number) != (kind, scope, number) {
                continue;
            }
            if let SettingSlot::Replaced(at) = key.slot
                && at > u64::from(revision)
                && match next {
                    Some((before, _)) => at < before,
                    None => true,
                }
                && let Edit::Set(value) = edit
            {
                next = Some((at, value));
            }
        }
        match next {
            Some((_, value)) => Some(value),
            None => {
                let key = SettingKey { kind, scope, number, slot: SettingSlot::Change };
                match self.setting_rows.get(&key) {
                    Some(Edit::Set(value)) => Some(value),
                    Some(Edit::Remove | Edit::Reset) => None,
                    None => self.setting_default(kind, scope, number),
                }
            }
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
        match self.scope_default(kind, scope, number) {
            Some(value) => Some(value),
            None => self.every_project_default(kind, scope, number),
        }
    }

    fn every_project_default(&self, kind: u32, scope: u32, number: u32) -> Option<&settings::SizedValue> {
        if kind == 2 && scope != 0 { self.scope_default(2, 0, number) } else { None }
    }

    fn scope_default(&self, kind: u32, scope: u32, number: u32) -> Option<&settings::SizedValue> {
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
        self.refused_settings.remove(&key);
        self.setting_rows.insert(key, edit.clone()).expect("settings checked by jig fit the forge's rows");
        out.push(Request::Save { record: Stored::Setting(SettingRow { key, edit }), release: None, read_afresh: None });
    }

    fn erase_setting(&mut self, key: SettingKey, out: &mut Queue<Request>) {
        let refused = self.refused_settings.remove(&key).is_some();
        if self.setting_rows.remove(&key).is_some() || refused {
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
        let mut every_project = List::uncapped();
        let mut projects = Map::<u32, List<settings::Held>>::with_capacity(self.held_rules.projects.capacity());
        for ((kind, scope, number), ()) in &keys {
            let Some(value) = self.setting(*kind, *scope, *number) else { continue };
            if let Value::Rule(rule) = &value.value {
                let held = settings::Held { scope: *scope, number: *number, rule: rule.clone() };
                match *kind {
                    1 => deployment.push(held).expect("bounded deployment rules"),
                    2 if *scope == 0 => every_project.push(held).expect("bounded given rules"),
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
        self.held_rules.every_project = every_project.into_boxed();
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
            Some(settings::SizedValue {
                value:
                    Value::Dependency(_)
                    | Value::Follow(_)
                    | Value::Rule(_)
                    | Value::Goal(_)
                    | Value::Questions(_)
                    | Value::Ways(_)
                    | Value::CharterWords(_)
                    | Value::Classes(_)
                    | Value::ProjectWords(_),
                ..
            })
            | None => 0,
        }
    }
}

fn held_for_project(held: &settings::HeldRules, project: u32) -> Box<[settings::Held]> {
    let mut result = List::uncapped();
    for rule in &held.deployment {
        result.push(rule.clone()).expect("bounded deployment rules");
    }
    let own_len = match held.projects.get(&project) {
        Some(rules) => rules.len(),
        None => 0,
    };
    let count = held.every_project.len().checked_add(own_len).expect("bounded held rules");
    let mut rules = Map::with_capacity(u32::try_from(count).expect("bounded held rules"));
    if project != 0 {
        for rule in &held.every_project {
            rules.insert(rule.number, settings::Held { scope: project, ..rule.clone() }).expect("bounded given rules");
        }
        if let Some(own) = held.projects.get(&project) {
            for rule in own {
                rules.insert(rule.number, rule.clone()).expect("bounded project rules");
            }
        }
    }
    for (_, rule) in &rules {
        result.push(rule.clone()).expect("bounded held rules");
    }
    result.into_boxed()
}

fn value_state(value: Option<&settings::SizedValue>) -> (u32, Option<u32>) {
    match value {
        Some(settings::SizedValue { value: Value::Follow(follow), size }) => {
            let charter = match &follow.producer {
                Some(producer) => settings::producer_charter(producer).flatten(),
                None => None,
            };
            (*size, charter)
        }

        Some(settings::SizedValue { value: Value::Charter(charter), size }) => (*size, Some(*charter)),
        Some(settings::SizedValue {
            value:
                Value::Dependency(_)
                | Value::Rule(_)
                | Value::Goal(_)
                | Value::Questions(_)
                | Value::Ways(_)
                | Value::CharterWords(_)
                | Value::Classes(_)
                | Value::ProjectWords(_),
            size,
        }) => (*size, None),
        None => (0, None),
    }
}

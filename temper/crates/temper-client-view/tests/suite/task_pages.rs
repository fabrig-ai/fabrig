//! adversarial task-page behaviour from design and public events.
#[cfg(test)]
#[expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    clippy::wildcard_enum_match_arm,
    reason = "this is an ordinary Rust driver, not step code; selectors collect only the requests each assertion reads"
)]
mod tests {
    use jig_client::contract as c;
    use skein_lib::{Duration, Env, List, Queue, RequestLimits, Time, Token, Wall};
    use temper_client_domain::{self as domain, Domain, Event, Limits, Request, forge};
    use temper_client_protocol_wire as wire;
    const TASK: u64 = 9;
    const FORGE: u16 = wire::FORGE;
    #[expect(clippy::arithmetic_side_effects, reason = "the request count is the nonzero constant declared here")]
    fn domain_limits() -> Limits {
        {
            let requests = RequestLimits {
                requests: 4,
                bytes: 262_144,
                out: 16,
                first: Duration::from_millis(10),
                most: Duration::from_millis(80),
                margin: Duration::from_secs(1),
            };
            let client = jig_client::Limits {
                domain: jig_client::DomainLimits {
                    accounts: jig_client::AccountCounts { accounts: 16, sources: 8, sign_ins: 4 },
                    owners: &[jig_client::Spoken {
                        family: temper_client_protocol_wire::FORGE,
                        version: temper_client_protocol_wire::CLIENT_VERSION,
                    }],
                    requests,
                    ask_bytes: requests.bytes / u64::from(requests.requests),
                    // Page handles, trials and a running trial task.
                    handles: 16, // Four findings subjects and one live-goals menu.
                    watches: 16, // Four findings subjects and one live-goals menu.
                    turn_bytes: 65536,
                    backoff: jig_client::Backoff { first: Duration::from_millis(10), most: Duration::from_millis(80) },
                    authority: 12,
                    heartbeat: Duration::from_millis(100),
                    reads: 8,
                    parts: 8,
                    delegates: 2,
                    document_bytes: 65536,
                    records: 2,
                    kept: 4,
                    projects: 2,
                    text: 32,
                    long_text: 32,
                    turn_text: 32,
                    rows: 2,
                    fields: 4,
                    segments: 9,
                },
                exchanges: 8,
            };

            Limits { forge: forge::Limits { field_bytes: 256, checks: 8 }, client, row_uses: 8, window_bytes: None }
        }
    }
    fn fixture_me(project_name: &[u8]) -> c::Me {
        use jig_client::contract::{self, DocumentText, HeldRole, Me, MeParty, MeProject, Needs, PartyKind};
        let mut projects = List::with_capacity(2);
        projects
            .push(MeProject {
                left_out: 0,
                defaults: List::with_capacity(0),
                public: None,
                project: 1,
                name: DocumentText::complete(
                    contract::Text::new(skein_lib::bytes::copy_of(project_name), 32).expect("name fits"),
                ),
                role: HeldRole::Policy { role: 1 },
                pool: None,
                templates: List::with_capacity(0),
                creating: None,
                models: List::with_capacity(0),
            })
            .expect("one project");
        let mut second = projects.as_slice()[0].clone();
        second.project = 2;
        second.name = DocumentText::complete(contract::Text::new(Box::from(*b"Second"), 32).expect("name fits"));
        projects.push(second).expect("second project");
        Me {
            providers: List::with_capacity(0),
            providers_cut: false,
            key_bytes: 256,
            redirect_bytes: 256,
            words_bytes: 8192,
            party: MeParty {
                number: 73,
                kind: PartyKind::Person,
                login: DocumentText::complete(contract::Text::new(Box::from(*b"pat"), 32).expect("login fits")),
                name: DocumentText::complete(contract::Text::new(Box::from(*b"Pat"), 32).expect("name fits")),
            },
            projects,
            needs: Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
            needs_total: 0,
            updates: contract::Count { value: 0, limited: false },
            results_seen: 0,
            cut: false,
            retention: Duration::from_secs(1000),
            deployment_role: None,
        }
    }

    fn document() -> c::Task {
        let mut parts = jig_client_test_records::task_record().into_parts();
        parts.chip = jig_client_test_records::owner_waiting_chip_record();
        let record = jig_client_protocol_wire::v19::Task::new(&jig_client_protocol_wire::v19::CEILINGS, parts)
            .expect("checked task record");
        let limits = domain_limits();
        let link =
            jig_client_protocol_link::Limits::new(limits.client.domain, limits.client.exchanges).expect("link limits");
        let mut task = jig_client_protocol_link::translate::documents::task(record, &link).expect("translated task");
        task.chip.task = TASK;
        task.project = 1;
        task.chip.executor = c::ChipExecutor::Procedure { connector: FORGE, kind: 2 };
        task.priority = None;
        task.goal = None;
        task.chip.phase = c::Phase::Active;
        let c::ChipPhaseFacts { phase, details, .. } = task.chip.details.as_mut();
        *phase = c::ChipPhase::Active(c::RunStage::Idle);
        details.project = 1;
        details.executor = c::Executor::Procedure { connector: FORGE, code: 2 };
        details.procedure =
            Some(c::Part { connector: FORGE, kind: c::PartKind::Procedure, key: c::PartKey::Number(TASK) });
        details.observation = None;
        details.ending = None;
        details.live_delegates = 0;
        task.acts = List::with_capacity(0);
        task.waits_on = c::WaitsOn::Nothing;
        task
    }
    fn root_event(root: &mut Domain, event: Event) -> Vec<jig_client::Down> {
        let limits = domain_limits();
        let mut out = Queue::uncapped();
        domain::step(root, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits }, event, &mut out);
        let mut down = Vec::new();
        while let Some(request) = out.pop() {
            match request {
                Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: value } } => down.push(value),
                Request::Show { address, .. } => {
                    down.extend(root_event(root, Event::Went { place: domain::Place::Address(address) }));
                }
                _ => {}
            }
        }
        down
    }
    fn up(root: &mut Domain, up: jig_client::Up) -> Vec<jig_client::Down> {
        root_event(root, Event::Client { event: jig_client::Event::Up { up } })
    }
    fn deliver(root: &mut Domain, watch: Token, value: c::Delivery) -> Vec<jig_client::Down> {
        up(root, jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(value) })
    }
    fn opened(document: c::Task) -> (Domain, Token, Token, Vec<jig_client::Down>) {
        opened_with_tree(document, true)
    }
    fn opened_with_tree(document: c::Task, supply_tree: bool) -> (Domain, Token, Token, Vec<jig_client::Down>) {
        let mut root = domain::unlinked(&domain_limits(), 11);
        let mut requests = root_event(
            &mut root,
            Event::Start { address: domain::Address::Task { task: TASK }, kept: List::with_capacity(0), with: None },
        );
        for event in [
            jig_client::Event::StartUnlinked { kept: List::with_capacity(0) },
            jig_client::Event::SignIn { with: jig_client::With::Transport },
            jig_client::Event::Up { up: jig_client::Up::Served { served: true, versions: None } },
        ] {
            requests.extend(root_event(&mut root, Event::Client { event }));
        }
        let mut task_watch = None;
        let mut tree_watch = None;
        let mut me_watch = None;
        for request in &requests {
            if let jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(subject) } = request {
                match subject {
                    c::Subject::Me => me_watch = Some(*watch),
                    c::Subject::Task { task: TASK } => task_watch = Some(*watch),
                    c::Subject::Tree { .. } => tree_watch = Some(*watch),
                    _ => (),
                }
            }
        }
        let mut after = deliver(&mut root, me_watch.expect("Me watch"), c::Delivery::Me(Box::new(fixture_me(b"Shop"))));
        let task_watch = task_watch.expect("task watch");
        after.extend(deliver(&mut root, task_watch, c::Delivery::Task(Box::new(document.clone()))));
        let tree_watch = tree_watch.expect("tree watch");
        if supply_tree {
            after.extend(deliver(&mut root, tree_watch, tree(&document)));
        }
        (root, task_watch, tree_watch, after)
    }
    fn tree(document: &c::Task) -> c::Delivery {
        c::Delivery::Tree(Box::new(c::Tree {
            root: document.chip.clone(),
            rows: List::with_capacity(0),
            folded: List::with_capacity(0),
            held_below: 0,
            next: None,
        }))
    }
    fn ended_reads(requests: &[jig_client::Down]) -> Vec<Token> {
        requests
            .iter()
            .filter_map(|request| match request {
                jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Tasks { query, .. }) }
                    if query.parent == Some(TASK) =>
                {
                    Some(*read)
                }
                _ => None,
            })
            .collect()
    }
    fn ended_answer(root: &mut Domain, read: Token, page: c::TaskPage) -> Vec<jig_client::Down> {
        up(
            root,
            jig_client::Up::ReadDone {
                read,
                result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Tasks(Box::new(page)))),
            },
        )
    }
    fn empty_page() -> c::TaskPage {
        c::TaskPage {
            live: List::with_capacity(0),
            live_after: None,
            ended: c::EndedGroup::Rows { rows: List::with_capacity(0), after: None, cut: false },
        }
    }
    fn answer_all_initial_reads(root: &mut Domain, requests: &[jig_client::Down], page: &c::TaskPage) {
        let reads = ended_reads(requests);
        assert_eq!(reads.len(), 1, "one initial read");
        let requests = ended_answer(root, *reads.first().expect("initial read"), page.clone());
        let reads = ended_reads(&requests);
        assert!(reads.len() <= 1, "one coalesced refresh");
        if let Some(read) = reads.first() {
            let requests = ended_answer(root, *read, page.clone());
            assert!(ended_reads(&requests).is_empty(), "no older page is read");
        }
    }
    fn ended_line(number: u64, parent: u64) -> c::TaskLine {
        let mut task = document();
        end(&mut task);
        task.chip.task = number;
        task.chip.title = c::DocumentText::complete(
            c::Text::new(format!("ended child {number}").into_bytes().into(), 32).expect("title"),
        );
        task.chip.executor = c::ChipExecutor::Agent { charter: 1 };
        let c::ChipPhaseFacts { details, .. } = task.chip.details.as_mut();
        details.executor = c::Executor::Agent { charter: 1 };
        details.procedure = None;
        c::TaskLine {
            chip: task.chip,
            project: 1,
            requester: c::Requester::Task(parent),
            tracked: None,
            label: task.label,
            flags: task.flags,
            changed: Wall::EPOCH,
            last_activity: Wall::EPOCH,
            ended_at: Some(Wall::EPOCH),
            budget: task.budget,
            waits_on: c::WaitsOn::Nothing,
            phases: c::PhaseCounts::ZERO,
            delegates_done: 0,
            delegates_total: 0,
            held_below: 0,
            conversation: None,
        }
    }

    fn words(bytes: &[u8]) -> c::DocumentText {
        c::DocumentText::complete(c::Text::new(bytes.into(), 32).expect("fixture words fit"))
    }
    fn phase(task: &mut c::Task, value: c::ChipPhase) {
        task.chip.phase = value.compact();
        let c::ChipPhaseFacts { phase, .. } = task.chip.details.as_mut();
        *phase = value;
    }
    fn details(task: &mut c::Task) -> &mut c::ChipDetails {
        let c::ChipPhaseFacts { details, .. } = task.chip.details.as_mut();
        details
    }
    fn agent() -> c::Task {
        let mut task = document();
        phase(&mut task, c::ChipPhase::Active(c::RunStage::Idle));
        task.chip.executor = c::ChipExecutor::Agent { charter: 1 };
        details(&mut task).executor = c::Executor::Agent { charter: 1 };
        details(&mut task).procedure = None;
        details(&mut task).attempt = None;
        details(&mut task).run_since = None;
        task.chip.title = words(b"QA agent brief");
        task.spec.specification.words = words(b"QA agent specification");
        task.label = 0;
        task.lineage_chips = List::with_capacity(0);
        task.dependencies = List::with_capacity(0);
        task.dependency_facts = List::with_capacity(0);
        task.hold = None;
        task.result = None;
        task
    }
    fn goal() -> c::Task {
        let mut task = agent();
        task.priority = Some(1);
        task.goal = Some(TASK);
        details(&mut task).live_delegates = 17;
        task
    }
    fn end(task: &mut c::Task) {
        finish(task, c::ChipEnding::Failed(words(b"ended")));
    }
    fn finish(task: &mut c::Task, ending: c::ChipEnding) {
        phase(task, c::ChipPhase::Ended);
        details(task).ending = Some(ending.clone());
        details(task).ended_at = Some(Wall::EPOCH);
        task.result = Some(c::TaskResult::Ending(Box::new(ending)));
    }
    fn view(root: &Domain) -> temper_client_view::View {
        let words = temper_client_view::words::Words::english(FORGE, 64, temper_client_view::words::Shell::Browser)
            .expect("English");
        // The allowed fixture envelope, never raised when a case exhausts it.
        let tree = skein_ui::Limits {
            nodes: 192,
            depth: 32,
            text_per_node: 512,
            text_bytes: 98304,
            drawing_bytes: 0,
            scene_marks: 0,
            scene_bytes: 0,
            styles_per_node: 2,
            patches: 960,
            bindings: 128,
            slots: 32,
            markdown_depth: 3,
        };
        let limits = temper_client_view::Limits { tree };
        let mut view = temper_client_view::View::new(&limits, &domain_limits(), words, FORGE);
        temper_client_view::build(&mut view, root, Wall::EPOCH, &mut Queue::with_capacity(tree.patches));
        view
    }
    fn text(nodes: &[skein_ui::Node]) -> String {
        let mut bytes = Vec::new();
        for node in nodes {
            for run in &node.text {
                bytes.extend_from_slice(&run.text);
            }
        }
        String::from_utf8(bytes).expect("visible UTF-8")
    }
    fn find(view: &temper_client_view::View, role: skein_ui::Role, name: &[u8]) -> skein_ui::NodeId {
        use skein_fake_person::{Face, Find, Found, TreeFace};
        let Found::Node(node) = TreeFace::new(view.tree()).find(&Find::exactly_named(role, name)) else {
            panic!("missing {:?}: {}", String::from_utf8_lossy(name), text(view.tree().nodes()));
        };
        node
    }

    fn page(rows: &[u64], cut: bool) -> c::TaskPage {
        let mut page = empty_page();
        let mut ended = List::with_capacity(domain_limits().client.domain.rows);
        for number in rows {
            ended.push(ended_line(*number, TASK)).expect("ended rows fit fixture");
        }
        page.ended = c::EndedGroup::Rows { rows: ended, after: None, cut };
        page
    }

    // client/pages/task.md 2: with no lineage, "parent is its requester's Task N link
    // when a task asked it"; otherwise no parent.
    #[test]
    fn agent_lineage_falls_back_only_to_a_task_requester() {
        for (requester, parent) in [
            (c::Requester::Task(12), Some("Task 12")),
            (c::Requester::Person(73), None),
            (c::Requester::Deployment { project: 1 }, None),
        ] {
            let mut task = agent();
            task.requester = requester;
            let (root, _, _, _) = opened(task);
            let mut view = view(&root);
            let nodes = view.tree().nodes();
            let index = nodes
                .iter()
                .position(|n| n.styles.as_slice().contains(&temper_client_view::look::AGENT_LINEAGE))
                .expect("lineage bar");
            let end = index.checked_add(usize::try_from(nodes[index].size).expect("size")).expect("end");
            let links: Vec<_> = nodes[index..end].iter().filter(|n| n.role == skein_ui::Role::Link).collect();
            assert_eq!(links.len(), usize::from(parent.is_some()), "{requester:?}");
            if let Some(parent) = parent {
                let node = find(&view, skein_ui::Role::Link, parent.as_bytes());
                assert_eq!(temper_client_view::href(&mut view, node).as_deref(), Some(b"/tasks/12".as_slice()));
            }
        }
    }

    // client/pages/task.md 2: claimed, running and closing run are "live now";
    // a live task with no run out says "waiting", and run_since is only shown there.
    #[test]
    fn agent_run_headings_follow_phase_even_without_observed_work() {
        let since = Wall::from_nanos(1_792_640_880_000_000_000);
        for (value, expected) in [
            (c::ChipPhase::Active(c::RunStage::Claimed { attempt: 2 }), "Run 2 · live now"),
            (c::ChipPhase::Active(c::RunStage::Running { attempt: 2 }), "Run 2 · live now"),
            (c::ChipPhase::Closing(c::ClosingStage::Run { attempt: 2 }), "Run 2 · live now"),
            (c::ChipPhase::Closing(c::ClosingStage::Delegates), "Run 2 · waiting"),
            (c::ChipPhase::Active(c::RunStage::BackingOff { until: since }), "Run 2 · waiting"),
        ] {
            let mut task = agent();
            phase(&mut task, value.clone());
            details(&mut task).attempt = Some(2);
            details(&mut task).run_since = Some(since);
            let (root, _, _, _) = opened(task);
            let drawing = text(view(&root).tree().nodes());
            assert!(drawing.contains(expected), "{value:?}: {drawing}");
            assert!(drawing.contains("Started 10-22 03:48Z"), "run placement: {drawing}");
            assert!(!drawing.contains("Result"), "live task has no result: {drawing}");
        }
    }

    // client/pages/task.md 2: "no heading once it has ended without [a run]";
    // result uses "Completed", "Failed", "Cancelled" and the result's words.
    #[test]
    fn never_run_agent_endings_show_result_without_a_run_heading() {
        for (ending, status, said) in [
            (c::ChipEnding::Done(c::ResultValue::Report(words(b"Complete report"))), "Completed", "Complete report"),
            (c::ChipEnding::Failed(words(b"Failure reason")), "Failed", "Failure reason"),
            (
                c::ChipEnding::Cancelled {
                    reason: words(b"Cancellation reason"),
                    result: None,
                    by: c::Requester::Person(73),
                    at: Wall::EPOCH,
                },
                "Cancelled",
                "Cancellation reason",
            ),
        ] {
            let mut task = agent();
            finish(&mut task, ending);
            let (root, _, _, _) = opened(task);
            let view = view(&root);
            find(&view, skein_ui::Role::Region, b"Result");
            let drawing = text(view.tree().nodes());
            assert!(drawing.contains(status) && drawing.contains(said), "{status}: {drawing}");
            assert!(!drawing.contains("No run now") && !drawing.contains("Run 0"), "no invented run: {drawing}");
        }
    }

    fn region_text(root: &Domain, name: &[u8]) -> String {
        let view = view(root);
        let id = find(&view, skein_ui::Role::Region, name);
        let nodes = view.tree().nodes();
        let index = nodes.iter().position(|n| n.id == id).expect("region");
        let end = index.checked_add(usize::try_from(nodes[index].size).expect("size")).expect("end");
        text(&nodes[index..end])
    }

    // client/pages/task.md 2: brief dependencies are "in order" with added marks
    // from "dependency_facts"; document cut "does not say which ... was cut",
    // so the line has no invented last item (5.8 marks match by number).
    #[test]
    fn agent_dependency_marks_match_numbers_and_do_not_invent_cut_items() {
        let mut task = agent();
        task.cut = true;
        task.dependencies = List::with_capacity(2);
        task.dependencies
            .push(c::TaskLink::Stored { number: 21, status: Some(c::TaskStatus::Done) })
            .expect("dependency");
        task.dependencies.push(c::TaskLink::Stored { number: 22, status: None }).expect("dependency");
        task.dependency_facts = List::with_capacity(2);
        task.dependency_facts
            .push(c::Dependency {
                number: 22,
                pending: true,
                added: Some(c::DependencyAdded { at: Wall::EPOCH, by: c::Requester::Person(73) }),
                keeps: true,
                status: None,
            })
            .expect("facts");
        task.dependency_facts
            .push(c::Dependency {
                number: 21,
                pending: false,
                added: Some(c::DependencyAdded { at: Wall::EPOCH, by: c::Requester::Task(12) }),
                keeps: false,
                status: Some(c::TaskStatus::Done),
            })
            .expect("facts");
        let (root, _, _, _) = opened(task);
        let drawing = region_text(&root, b"Task brief");
        let first = drawing.find("Task 21").expect("first dependency");
        let second = drawing.find("Task 22").expect("second dependency");
        assert!(first < second, "dependency order: {drawing}");
        assert!(drawing.contains("Task 21CompletedAdded by task 12"), "facts matched by number: {drawing}");
        assert!(drawing.contains("Task 22Added by you · keeps task 9 from ending"), "viewer and keeps: {drawing}");
        assert!(
            !drawing.contains("More dependencies not shown."),
            "aggregate cut identifies no lost dependency: {drawing}"
        );
    }

    // client/pages/task.md 4: "An act the document does not offer is not drawn,
    // not even disabled"; offered Message alone controls the agent's composer.
    #[test]
    fn agent_composer_disappears_when_the_offer_is_removed() {
        let mut task = agent();
        task.acts = List::with_capacity(1);
        task.acts
            .push(c::Offered { act: c::ActKind::Message, admission: c::Admission::Allowed, effect: None })
            .expect("message offered");
        let (mut root, watch, _, _) = opened(task.clone());
        // Keep typed steering words while a new whole document removes Message.
        drop(root_event(
            &mut root,
            Event::Act {
                action: domain::Action::Pages(domain::pages::Act::MessageWords {
                    words: b"A private draft".as_slice().into(),
                }),
            },
        ));
        task.acts = List::with_capacity(0);
        drop(deliver(&mut root, watch, c::Delivery::Task(Box::new(task))));
        let view = view(&root);
        assert!(
            !view
                .tree()
                .nodes()
                .iter()
                .any(|node| (node.role == skein_ui::Role::TextArea || node.role == skein_ui::Role::Button)
                    && (node.name.as_deref() == Some(b"Message") || node.name.as_deref() == Some(b"Send"))),
            "removed Message hides controls: {}",
            text(view.tree().nodes())
        );
    }

    // client/pages/task.md 2: Agent is "an agent's executor, no priority, not the
    // chat label"; 5.11 allows untracking a goal. Its resulting page has a brief
    // and conversation, with no inherited ended-delegate query (5.8).
    #[test]
    fn untracking_a_goal_switches_to_the_agent_page() {
        let (mut root, watch, _, requests) = opened(goal());
        answer_all_initial_reads(&mut root, &requests, &page(&[21], false));
        drop(deliver(&mut root, watch, c::Delivery::Task(Box::new(agent()))));
        let view = view(&root);
        find(&view, skein_ui::Role::Region, b"Task brief");
        let drawing = text(view.tree().nodes());
        assert!(!drawing.contains("ended child 21"), "goal's ended rows leave with its tracked shape: {drawing}");
        assert!(!drawing.contains("Its delegates:"), "agent has no goal plan counts: {drawing}");
    }
    // client/pages/task.md, 4: needs are bands; held tasks have cards on their own pages.
    #[test]
    fn a_tracked_page_reads_no_own_or_delegate_held_cards() {
        let mut task = goal();
        phase(
            &mut task,
            c::ChipPhase::Held { reason: c::HoldCategory::Failures { class: c::HoldFailure::Run }, revision: 7 },
        );
        let (mut root, _, tree_watch, mut requests) = opened(task.clone());
        let mut child = agent();
        child.chip.task = 21;
        phase(
            &mut child,
            c::ChipPhase::Held { reason: c::HoldCategory::Failures { class: c::HoldFailure::Run }, revision: 7 },
        );
        let mut rows = List::with_capacity(1);
        rows.push(c::TreeRow {
            chip: child.chip,
            parent: TASK,
            depth: 1,
            proposals: List::with_capacity(0),
            spent: 0,
            dependencies: List::with_capacity(0),
            cut: false,
        })
        .expect("held delegate");
        requests.extend(deliver(
            &mut root,
            tree_watch,
            c::Delivery::Tree(Box::new(c::Tree {
                root: task.chip,
                rows,
                folded: List::with_capacity(0),
                held_below: 0,
                next: None,
            })),
        ));
        assert!(root.pages().task().expect("tracked page").cards.is_empty());
        for request in &requests {
            if let jig_client::Down::Read { what: jig_client::Readable::Jig(c::Read::Task { .. }), .. } = request {
                panic!("tracked page must not read task cards: {requests:?}");
            }
        }
    }
}

//! Linked-root cold opens, not a driver that injects already translated page facts.
#[cfg(test)]
#[expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary Rust domain world inspects typed wire records and semantic trees"
)]
mod tests {
    use jig_client_protocol_wire as w;
    use skein_lib::{Duration, Env, List, Queue, Reader, Time, Wall};
    use temper_client_domain::{
        self as domain, Address, Event, GoalsFilter, Request, WorkFilter,
        pages::{Budget, work::BUDGET},
    };
    use temper_client_view_kit::{
        self as kit,
        engine::{Engine, page_requests},
    };
    mod fixture;
    use fixture::{NOW, PROJECT, Project};

    fn env() -> Env<domain::Limits> {
        Env { now: Time::ZERO, wall: NOW, limits: fixture::limits() }
    }
    fn take(out: &mut Queue<Request>) -> Vec<jig_client::Request> {
        let mut sent = Vec::new();
        while let Some(request) = out.pop() {
            match request {
                Request::Client { request: jig_client::Request::Save { .. } | jig_client::Request::Erase { .. } } => {}
                Request::Client { request } => sent.push(request),
                other => panic!("unexpected root request: {other:?}"),
            }
        }
        sent
    }
    fn contains(haystack: &[u8], needle: impl AsRef<[u8]>) {
        let needle = needle.as_ref();
        assert!(
            skein_lib::bytes::find(haystack, needle).is_some(),
            "missing {:?} in {:?}",
            String::from_utf8_lossy(needle),
            String::from_utf8_lossy(haystack)
        );
    }
    fn page_members(batch: &w::Batch) -> Vec<String> {
        let mut order = Vec::new();
        for member in batch.members() {
            let (subject, watch) = match member {
                w::Member::Open(open) => (open.subject(), true),
                w::Member::Read(read) => (read, false),
                w::Member::Close(_) => panic!("no cold-open closing"),
            };
            match subject.owner() {
                w::Owner::Jig if watch => {
                    if let w::Subject::Tasks(tasks) =
                        w::Subject::decode(&w::CEILINGS, &mut Reader::new(subject.record())).expect("subject")
                        && tasks.query().label() != &Some(3)
                    {
                        order.push(format!("watch {}", tasks.query().label().expect("page label")));
                    }
                }
                w::Owner::Jig => match w::Read::decode(&w::CEILINGS, &mut Reader::new(subject.record())).expect("read")
                {
                    w::Read::Tasks(tasks) => order.push(format!("read {}", tasks.query().label().expect("page label"))),
                    w::Read::Inbox(_) => order.push("needs".into()),
                    _ => {}
                },
                w::Owner::Family(_) if watch => {
                    use temper_client_protocol_wire::{self as wire, v1 as f};
                    let envelope = w::FamilyRecord::decode(&w::CEILINGS, &mut Reader::new(subject.record()))
                        .expect("family scope and window");
                    match wire::decode_watching(envelope.record(), &f::CEILINGS).expect("watch").watch() {
                        f::Watch::Changes(_) => order.push("changes".into()),
                        f::Watch::Drafts(_) => order.push("drafts".into()),
                        _ => {}
                    }
                }
                w::Owner::Family(_) => {}
            }
        }
        order
    }
    #[expect(
        clippy::too_many_lines,
        reason = "the final tree's fixture expectations stay together across Work's filters"
    )]
    fn drawn(tree: &skein_ui::Tree, project: &Project, address: &Address) {
        let text = kit::text(tree);
        let main = kit::main_text(tree);
        contains(&text, "Shop");
        // The rail's Needs badge comes from the hello's me, not an extra read.
        assert!(tree.nodes().iter().any(|node| node.name.as_deref() == Some(b"Needs you 1")), "rail's Needs count");
        match address {
            Address::Work { filter, .. } => {
                if *filter == WorkFilter::All {
                    for (n, _) in project.goals.iter().enumerate() {
                        contains(&main, format!("Live goal {n}"));
                        contains(&main, format!("Headline live {n}"));
                    }
                    for (n, _) in project.milestones.iter().enumerate() {
                        contains(&main, format!("Live milestone {n}"));
                        contains(&main, format!("Nested goal {n}"));
                        let title = format!("Live milestone {n}");
                        let link_index = tree
                            .nodes()
                            .iter()
                            .position(|node| {
                                node.role == skein_ui::Role::Link && node.name.as_deref() == Some(title.as_bytes())
                            })
                            .expect("each milestone's card");
                        let (card_index, _) = tree
                            .nodes()
                            .iter()
                            .enumerate()
                            .find(|(index, node)| {
                                node.styles.as_slice().contains(&temper_client_view::look::MS_CARD)
                                    && *index <= link_index
                                    && index
                                        .checked_add(usize::try_from(node.size).expect("card size"))
                                        .expect("card end")
                                        > link_index
                            })
                            .expect("milestone card contains its title");
                        let card = kit::node_text(tree, card_index);
                        contains(&card, format!("Nested headline {n}"));
                        contains(&card, "3 goals");
                        contains(&card, "31 spent of 100");
                        contains(&card, "2 of 3 goals done.");
                    }
                    for queue in project.changes.queues() {
                        for change in queue.changes() {
                            contains(&main, change.title());
                        }
                    }
                    for (n, landed) in project.changes.landed_default().iter().enumerate() {
                        contains(&main, landed.title());
                        let signers = format!("signed by you and Signer {n}");
                        assert!(
                            tree.nodes()
                                .iter()
                                .any(|node| node.name.as_ref().is_some_and(|name| name.ends_with(signers.as_bytes()))),
                            "every landed row's signers: {signers}"
                        );
                    }
                }
                // The chat's draft stays listed even when its change is in flight.
                for n in 0..project.chats.len() {
                    contains(&main, format!("Draft chat {n}"));
                    contains(&main, format!("draft-document-{n}.md"));
                }
                for draft in project.drafts.drafts() {
                    contains(&main, draft.title());
                    contains(&main, draft.path());
                }
                let heading = tree
                    .nodes()
                    .iter()
                    .find(|node| node.role == skein_ui::Role::Heading { level: 1 })
                    .expect("whole heading");
                let name = heading.name.as_ref().expect("counted heading");
                assert_eq!(
                    name.as_ref(),
                    if project.goals.len() == 2 {
                        b"Five in flight, three drafts, two landed this week.".as_slice()
                    } else {
                        b"63 in flight, 80 drafts, 32 landed this week.".as_slice()
                    },
                    "the heading counts the fixture's cards and landings"
                );
            }
            Address::Goals { filter, .. } => {
                for (ended, lines) in [(false, &project.goals), (true, &project.ended)] {
                    for (n, line) in lines.iter().enumerate() {
                        let shown = match filter {
                            GoalsFilter::All => true,
                            GoalsFilter::Needs => !ended && n == 0,
                            GoalsFilter::Live => !ended && n != 0,
                            GoalsFilter::Ended => ended,
                        };
                        let prefix = [line.chip.title.bytes(), " · ".as_bytes()].concat();
                        let row = tree.nodes().iter().enumerate().find(|(_, node)| {
                            node.styles.as_slice().contains(&temper_client_view::look::GOALS_ROW)
                                && node.name.as_ref().is_some_and(|name| name.starts_with(&prefix))
                        });
                        assert_eq!(
                            row.is_some(),
                            shown,
                            "filter {filter:?}: {}",
                            String::from_utf8_lossy(line.chip.title.bytes())
                        );
                        if let Some((index, _)) = row {
                            let row_text = kit::node_text(tree, index);
                            contains(&row_text, format!("Headline {} {n}", if ended { "ended" } else { "live" }));
                            contains(
                                &row_text,
                                if ended {
                                    "landed"
                                } else if n == 0 {
                                    "needs you"
                                } else {
                                    "running"
                                },
                            );
                            contains(&row_text, "2 / 3");
                        }
                    }
                }
            }
            Address::Changes { .. } => {
                contains(&main, "Queue");
                contains(&main, "on the way to main.");
                for queue in project.changes.queues() {
                    for change in queue.changes() {
                        contains(&main, change.title());
                    }
                }
                for change in project.changes.landed() {
                    contains(&main, change.title());
                }
            }
            _ => panic!("Work's family only"),
        }
    }
    fn open(address: Address, bounded: bool, kept: List<jig_client::Record>) -> Budget {
        let reload = !kept.is_empty();
        let project = Project::new(bounded);
        let limits = fixture::limits();
        let mut root = domain::new(&limits, 11);
        let words = kit::browser_words();
        let view_limits = temper_client_view::Limits::of(&limits, &words).expect("stock view limits");
        let mut view = temper_client_view::View::new(&view_limits, &limits, words, kit::FORGE);
        let mut out = Queue::uncapped();
        domain::step(
            &mut root,
            &env(),
            Event::Start { address: address.clone(), kept, with: Some(jig_client::With::Transport) },
            &mut out,
        );
        domain::settle(&mut root, &env(), &mut out);
        let mut engine = Engine::new(fixture::me());
        let mut cost = Budget { waves: 0, requests: 0 };
        loop {
            let sent = take(&mut out);
            let count = page_requests(&sent);
            if count == 0 {
                assert!(sent.is_empty(), "the whole page sends nothing more: {sent:?}");
                break;
            }
            cost.waves = cost.waves.checked_add(1).expect("waves");
            cost.requests = cost.requests.checked_add(count).expect("requests");
            let openings = sent
                .iter()
                .filter_map(|request| {
                    if let jig_client::Request::Exchange { message: jig_client::Message::Watch(bytes), .. } = request {
                        Some(w::Open::decode(&w::CEILINGS, &mut Reader::new(bytes)).expect("opening"))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            assert!(
                cost.waves <= BUDGET.waves && cost.requests <= BUDGET.requests,
                "{address:?}, bounded={bounded}: {cost:?}, sent {sent:?}, openings {openings:?}"
            );
            if cost.waves == 1 {
                assert_eq!(count, 1);
                let [jig_client::Request::OpenStream { .. }] = sent.as_slice() else {
                    panic!("stream alone before the hello: {sent:?}")
                };
            } else {
                assert_eq!(count, 1);
            }
            for event in engine.wave(&sent, |subject, watch| project.answer(subject, watch)) {
                domain::step(&mut root, &env(), Event::Client { event }, &mut out);
            }
            assert_eq!(
                engine.keyed.len(),
                if reload && cost.waves == 2 { 2 } else { 0 },
                "kept routes wait for the hello's party confirmation, then leave beside the batch"
            );
            domain::settle(&mut root, &env(), &mut out);
            domain::take(&mut root, &env(), &mut out);
            temper_client_view::build(&mut view, &root, NOW, &mut Queue::uncapped());
        }
        assert_eq!(cost, BUDGET);
        assert_eq!(engine.batches.len(), 1);
        assert_eq!(engine.replaces, [if reload { Some([7; 16]) } else { None }]);
        if reload {
            assert_eq!(
                engine.keyed.iter().map(|request| *request.key()).collect::<Vec<_>>(),
                [[11; 16], [12; 16]],
                "each retained key keeps its route and identity"
            );
        }
        assert_eq!(
            page_members(&engine.batches[0]),
            ["watch 8", "read 8", "needs", "watch 9", "read 9", "watch 7", "read 7", "changes", "drafts"]
        );
        assert!(
            root.pages().goals().expect("goals").known(root.client()),
            "goals answered: {:?}",
            root.pages().goals()
        );
        assert!(
            root.pages().milestones().expect("milestones").known(root.client()),
            "milestones answered: {:?}",
            root.pages().milestones()
        );
        assert!(
            root.pages().chats().expect("chats").known(root.client()),
            "chats answered: {:?}",
            root.pages().chats()
        );
        drawn(view.tree(), &project, &address);
        cost
    }
    fn both(address: Address) {
        let small = open(address.clone(), false, List::with_capacity(0));
        let bounded = open(address, true, List::with_capacity(0));
        assert_eq!(small, bounded);
    }
    fn kept() -> List<jig_client::Record> {
        let limits = fixture::limits();
        let mut client = jig_client::Client::new(&limits.client, 11).expect("linked client");
        let mut out = Queue::uncapped();
        for event in [
            jig_client::Event::Start { kept: List::with_capacity(0) },
            jig_client::Event::SignIn { with: jig_client::With::Transport },
            Engine::new(fixture::me()).hello(),
            jig_client::Event::Send {
                key: jig_client::Key { asker: domain::owner::Owner::Pages.asker(), number: 11 },
                name: Some([11; 16]),
                request: jig_client::Ask::Jig(Box::new(jig_client::contract::Ask::Stop {
                    project: PROJECT,
                    task: 9001,
                })),
            },
            jig_client::Event::Send {
                key: jig_client::Key { asker: domain::owner::Owner::Pages.asker(), number: 12 },
                name: Some([12; 16]),
                request: jig_client::Ask::Jig(Box::new(jig_client::contract::Ask::Stop {
                    project: PROJECT,
                    task: 9002,
                })),
            },
        ] {
            client.step(Time::ZERO, NOW, event, &mut out);
            client.settle(Time::ZERO, NOW, &mut out);
        }
        let mut records = List::uncapped();
        while let Some(request) = out.pop() {
            if let jig_client::Request::Save { records: saved } = request {
                for record in saved.into_boxed() {
                    records.push(record).expect("saved records");
                }
            }
        }
        assert_eq!(records.len(), 3, "stream identity and two requests are kept");
        records
    }
    #[test]
    fn reload_does_not_wait_for_kept_keyed_requests() {
        open(Address::Work { project: PROJECT, filter: WorkFilter::All }, false, kept());
    }
    #[test]
    fn work_all() {
        both(Address::Work { project: PROJECT, filter: WorkFilter::All });
    }
    #[test]
    fn work_drafts() {
        both(Address::Work { project: PROJECT, filter: WorkFilter::Drafts });
    }
    #[test]
    fn goals_all() {
        both(Address::Goals { project: PROJECT, filter: GoalsFilter::All });
    }
    #[test]
    fn goals_needs() {
        both(Address::Goals { project: PROJECT, filter: GoalsFilter::Needs });
    }
    #[test]
    fn goals_live() {
        both(Address::Goals { project: PROJECT, filter: GoalsFilter::Live });
    }
    #[test]
    fn goals_ended() {
        both(Address::Goals { project: PROJECT, filter: GoalsFilter::Ended });
    }
    #[test]
    fn changes() {
        both(Address::Changes { project: PROJECT });
    }
}

//! One inventory drives both profile checks and mutations (testing/settings-tests.md, 2).
use super::{Kind, Profile, Relation, Small, derived, limits, value};
use skein_lib::List;
type Carrier = (&'static str, &'static [&'static str], fn(&Profile) -> u64);
type Count = (&'static str, fn(&Profile) -> u64);
type Pair = (&'static str, &'static str, &'static [&'static str], fn(&Profile) -> u64);

fn row(
    name: &'static str,
    kind: Kind,
    design: &'static str,
    figure: &'static str,
    sources: &'static [&'static str],
    formula: fn(&Profile) -> u64,
) -> Relation {
    Relation {
        name,
        kind,
        design,
        figure,
        sources,
        formula,
        small: Small::Same,
        minimum: false,
        worker: false,
        overload: None,
        explicit: None,
        implicit: false,
        full_start: false,
    }
}
fn implicit(mut relation: Relation) -> Relation {
    relation.implicit = true;
    relation
}
// These relations also need checks after configuration is composed.
fn full_start(mut relation: Relation) -> Relation {
    relation.full_start = true;
    relation
}
fn worker(mut relation: Relation) -> Relation {
    relation.worker = true;
    relation
}
fn ceiling(mut relation: Relation, overload: fn(&Profile) -> super::Override) -> Relation {
    relation.overload = Some(overload);
    relation
}
fn explicit(mut relation: Relation, value: u64) -> Relation {
    relation.explicit = Some(value);
    relation
}
fn minimum(mut relation: Relation) -> Relation {
    relation.minimum = true;
    relation
}
fn add(rows: &mut List<Relation>, relation: Relation) {
    rows.push(relation).expect("bounded relation inventory");
}
fn scaled(p: &Profile, name: &str, cap: u64) -> u64 {
    let base = Profile { jig: jig_protocol_config::stock(), own: limits::stock(), slots: 0 };
    value(&base, name)
        .checked_mul(u64::from(p.own.live_goals))
        .expect("u32 product")
        .div_ceil(u64::from(derived::LIVE_GOALS))
        .min(cap)
}
fn loaded() -> u64 {
    u64::from(derived::STOCK.turn)
        .checked_add(u64::try_from(size_of::<temper_engine_domain::Record>()).expect("record size"))
        .expect("loaded bytes")
}
fn payload(p: &Profile) -> u32 {
    p.own.tool_payload.max(derived::STOCK.payload)
}
fn message(p: &Profile) -> u64 {
    u64::from(derived::message(payload(p)).expect("payload separator"))
}
fn twice(n: u64) -> u64 {
    n.checked_mul(2).expect("bounded figure")
}

#[expect(clippy::too_many_lines, reason = "one relation inventory drives both profile checks and generated mutations")]
pub(super) fn relations() -> List<Relation> {
    use Kind::{CheckedTogether, Floor, Follows, HoldsAll, MayRefuseBatch, SetByTemper};
    let mut rows = List::with_capacity(100);
    let lifetime = "configuration/limits.md, the forge client's lifetime";
    add(
        &mut rows,
        explicit(
            row(
                "forge component deadline",
                Follows,
                lifetime,
                "forge_component.deadline_ms",
                &["forge.client.lifetime_ms"],
                |p| p.own.forge.client.lifetime.as_millis().min(270_000),
            ),
            400_000,
        ),
    );
    add(
        &mut rows,
        ceiling(
            full_start(row(
                "forge deadline within lifetime",
                HoldsAll,
                lifetime,
                "forge.client.lifetime_ms",
                &["forge_component.deadline_ms"],
                |p| p.own.forge_component.deadline.as_millis(),
            )),
            |p| {
                super::item(
                    "forge_component.deadline_ms",
                    p.own.forge.client.lifetime.as_millis().checked_add(1).expect("deadline past lifetime"),
                )
            },
        ),
    );
    let together = "configuration/limits.md, limits checked against each other";
    add(
        &mut rows,
        full_start(row(
            "forge kept answers",
            SetByTemper,
            together,
            "forge_component.kept_bytes",
            &["forge.client.calls", "forge.client.answer_bytes"],
            |p| {
                u64::from(p.own.forge.client.calls)
                    .checked_mul(u64::from(p.own.forge.client.answer_bytes))
                    .expect("u32 product")
            },
        )),
    );
    add(
        &mut rows,
        full_start(row(
            "forge connections and calls",
            CheckedTogether,
            together,
            "forge_component.connections",
            &["forge.client.calls"],
            |p| u64::from(p.own.forge.client.calls),
        )),
    );
    add(
        &mut rows,
        implicit(row(
            "forge write and marker",
            Floor,
            "configuration/derived.md, 5.2",
            "forge_component.body_bytes",
            &["forge.client.op_bytes"],
            |p| u64::from(derived::forge_body(p.own.forge.client.op_bytes).expect("marker room")),
        )),
    );
    add(
        &mut rows,
        row(
            "forge component repositories",
            Follows,
            "configuration/derived.md, 5.2",
            "forge_component.repositories",
            &["forge.repositories"],
            |p| u64::from(p.own.forge.repositories),
        ),
    );
    add(
        &mut rows,
        minimum(row(
            "forge component string carrier",
            CheckedTogether,
            together,
            "forge_component.string_bytes",
            &["forge.name_bytes"],
            |p| u64::from(p.own.forge.name_bytes),
        )),
    );
    // forge::stock derives the configured-forge table, before any overrides.
    add(
        &mut rows,
        row(
            "forge component forges",
            Follows,
            "protocol/forge/README.md, 6",
            "forge_component.forges",
            &["forge.forges"],
            |p| u64::from(p.own.forge.forges),
        ),
    );

    add(
        &mut rows,
        worker(row(
            "worker workspaces",
            Follows,
            "configuration/derived.md, 5.2",
            "checkout.workspaces",
            &["worker.slots"],
            |p| u64::from(p.slots),
        )),
    );
    add(
        &mut rows,
        worker(row(
            "worker git operations",
            Follows,
            "configuration/derived.md, 5.2",
            "git_component.operations",
            &["checkout.workspaces"],
            |p| u64::from(p.own.checkout.workspaces),
        )),
    );
    let carriers = "configuration/derived.md, 5.1";
    let start = rows.len();
    let carriers_table: &[Carrier] = &[
        ("checkout.message_bytes", &["tool_payload"][..], (|p: &Profile| u64::from(payload(p)))),
        ("forge.client.op_bytes", &["tool_payload", "tasks.result_bytes"][..], |p: &Profile| {
            message(p).max(u64::from(p.jig.core.tasks.result_bytes))
        }),
        (
            "forge.client.answer_bytes",
            &["forge_component.body_bytes", "forge_component.string_bytes"][..],
            |p: &Profile| {
                u64::from(
                    derived::forge_answer(&p.own.forge_component, limits::stock().forge.client.answer_bytes)
                        .expect("largest listing row"),
                )
            },
        ),
        ("root.journal.transcript_bytes", &["agent.turn", "core.resume_bytes"][..], |p: &Profile| {
            u64::from(derived::resume(derived::STOCK.turn).expect("resume")).max(u64::from(p.jig.core.resume_bytes))
        }),
        ("root.loads.transcript_bytes", &["agent.turn"][..], |_: &Profile| u64::from(derived::STOCK.turn)),
        ("root.loads.bytes", &["agent.turn"][..], |_: &Profile| loaded()),
        ("root.loads.reply_bytes", &["agent.turn"][..], |_: &Profile| loaded()),
        ("root.loads.read_bytes", &["agent.turn"][..], |_: &Profile| {
            derived::read(
                derived::STOCK.turn,
                u64::try_from(size_of::<jig_domain_core::reads::Row>()).expect("row size"),
                u64::try_from(size_of::<jig_domain_core::Record>()).expect("record size"),
            )
            .expect("read room")
        }),
        ("tasks.message_bytes", &["tool_payload"][..], message),
        ("tasks.result_bytes", &["tool_payload"][..], message),
        ("tasks.inbox_bytes", &["tool_payload", "agent.inbox"][..], |p: &Profile| {
            u64::from(derived::inbox(payload(p), derived::STOCK.inbox).expect("inbox room"))
        }),
        ("hub.event_bytes", &["tool_payload"][..], message),
        ("hub.turn_bytes", &["agent.turn"][..], |_: &Profile| u64::from(derived::STOCK.turn)),
        ("core.resume_bytes", &["agent.turn"][..], |_: &Profile| {
            u64::from(derived::resume(derived::STOCK.turn).expect("resume"))
        }),
        ("hub.transcript_bytes", &["agent.turn"][..], |_: &Profile| {
            derived::transcript(derived::resume(derived::STOCK.turn).expect("resume")).expect("transcript")
        }),
        ("hub.turn_queue_bytes", &["agent.turn", "agent.unacknowledged"][..], |_: &Profile| {
            derived::turn_queue(derived::STOCK.turn, derived::STOCK.unacknowledged).expect("turn queue")
        }),
        ("hub.outcome_bytes", &["tool_payload"][..], |p: &Profile| {
            derived::outcome(payload(p), jig_protocol_config::stock().hub.outcome_bytes)
        }),
        ("core.call_answer_bytes", &["tool_payload"][..], |p: &Profile| {
            u64::from(derived::call_answer(payload(p), jig_protocol_config::stock().core.call_answer_bytes))
        }),
        ("io.output", &["agent.turn"][..], |p: &Profile| {
            u64::from(derived::output(&p.jig).expect("frame room").max(derived::OUTPUT_FLOOR))
        }),
    ];
    for &(name, sources, formula) in carriers_table {
        add(&mut rows, row(name, Floor, carriers, name, sources, formula));
    }
    for entry in rows.iter_mut().skip(usize::try_from(start).expect("table offset")) {
        entry.small = Small::Carriers;
    }

    let counts = "configuration/derived.md, 5.4";
    let start = rows.len();
    let counts_table: &[Count] = &[
        ("tasks.tasks", (|p: &Profile| scaled(p, "tasks.tasks", u64::MAX))),
        ("tasks.project_tasks", |p: &Profile| scaled(p, "tasks.project_tasks", u64::MAX)),
        ("tasks.project_goals", |p: &Profile| scaled(p, "tasks.project_goals", u64::MAX)),
        ("tasks.placed_holds", |p: &Profile| scaled(p, "tasks.placed_holds", u64::MAX)),
        ("tasks.project_placed_holds", |p: &Profile| scaled(p, "tasks.project_placed_holds", u64::MAX)),
        ("tasks.party_placed_holds", |p: &Profile| scaled(p, "tasks.party_placed_holds", u64::MAX)),
        ("tasks.funders", |p: &Profile| scaled(p, "tasks.funders", u64::MAX)),
        ("tasks.pools", |p: &Profile| scaled(p, "tasks.pools", u64::MAX)),
        ("tasks.hold_waiters", |p: &Profile| scaled(p, "tasks.hold_waiters", u64::MAX)),
        ("forge.tasks", |p: &Profile| scaled(p, "forge.tasks", 1024)),
        ("forge.holds", |p: &Profile| scaled(p, "forge.holds", 256)),
        ("forge.subscriptions", |p: &Profile| scaled(p, "forge.subscriptions", u64::MAX)),
        ("forge.entries", |p: &Profile| scaled(p, "forge.entries", u64::MAX)),
        ("forge.client.entries", |p: &Profile| scaled(p, "forge.client.entries", u64::MAX)),
        ("forge.client.resources", |p: &Profile| scaled(p, "forge.client.resources", u64::MAX)),
        ("forge.issues", |p: &Profile| scaled(p, "forge.issues", u64::MAX)),
    ];
    for &(name, formula) in counts_table {
        add(&mut rows, row(name, Follows, counts, name, &["live_goals"], formula));
    }
    add(
        &mut rows,
        row("landing members", Follows, counts, "forge.landing_members", &["forge.holds"], |p| {
            u64::from(limits::stock().forge.landing_members.min(p.own.forge.holds))
        }),
    );
    add(
        &mut rows,
        full_start(row(
            "live landings",
            Follows,
            counts,
            "forge.landings",
            &["live_goals", "forge.holds", "forge.landing_members"],
            |p| {
                scaled(
                    p,
                    "forge.landings",
                    u64::from(
                        p.own.forge.holds.checked_div(p.own.forge.landing_members).expect("positive landing members"),
                    ),
                )
            },
        )),
    );
    add(
        &mut rows,
        row("delegates", Follows, counts, "tasks.delegates", &["tasks.tasks"], |p| {
            u64::from(jig_protocol_config::stock().core.tasks.delegates.min(p.jig.core.tasks.tasks.saturating_sub(1)))
        }),
    );
    add(
        &mut rows,
        row(
            "people operations",
            Follows,
            counts,
            "people.operations",
            &["tasks.delegates", "tasks.project_goals", "tasks.tasks"],
            |p| {
                u64::from(
                    p.jig
                        .core
                        .tasks
                        .delegates
                        .saturating_add(1)
                        .max(p.jig.core.tasks.project_goals)
                        .min(p.jig.core.tasks.tasks),
                )
            },
        ),
    );
    add(
        &mut rows,
        row("inbox tasks", Follows, counts, "people.inbox_tasks", &["tasks.tasks"], |p| {
            twice(u64::from(p.jig.core.tasks.tasks))
        }),
    );
    add(
        &mut rows,
        row("exemptions", Follows, counts, "tasks.exemptions", &["live_goals", "tasks.tasks"], |p| {
            scaled(p, "tasks.exemptions", 4096).min(u64::from(p.jig.core.tasks.tasks))
        }),
    );
    for entry in rows.iter_mut().skip(usize::try_from(start).expect("table offset")) {
        entry.small = Small::Counts;
    }

    add(
        &mut rows,
        row(
            "brief read holds task section",
            Floor,
            "configuration/limits.md, the brief's task section",
            "root.brief.read_bytes",
            &["root.brief.budgets.task"],
            |p| u64::from(p.own.root.brief.budgets.task),
        ),
    );
    add(
        &mut rows,
        row("forge brief holds a read", Floor, together, "forge.brief_bytes", &["root.brief.read_bytes"], |p| {
            u64::from(p.own.root.brief.read_bytes)
        }),
    );
    add(
        &mut rows,
        row(
            "journal holds the brief",
            Floor,
            together,
            "root.journal.result_bytes",
            &["root.brief.brief_bytes"],
            |p| u64::from(p.own.root.brief.brief_bytes),
        ),
    );
    add(
        &mut rows,
        row(
            "forge stop bound",
            Follows,
            "configuration/derived.md, 5.6",
            "stop_bound_ms",
            &["forge.client.lifetime_ms", "forge.client.clock_margin_ms"],
            |p| {
                twice(
                    p.own
                        .forge
                        .client
                        .lifetime
                        .as_millis()
                        .checked_add(p.own.forge.client.clock_margin.as_millis())
                        .expect("stop sum"),
                )
            },
        ),
    );

    add(
        &mut rows,
        row(
            "landing holds may refuse a batch",
            MayRefuseBatch,
            "domain/landing.md, 12",
            "forge.holds",
            &["forge.landings", "forge.landing_members"],
            |p| {
                u64::from(p.own.forge.landings)
                    .checked_mul(u64::from(p.own.forge.landing_members))
                    .expect("hold product")
            },
        ),
    );
    add(
        &mut rows,
        row(
            "client resources may refuse a batch",
            MayRefuseBatch,
            "domain/forge/keeping-up.md, 1",
            "forge.client.resources",
            &["forge.tasks", "forge.resources_per_task"],
            |p| {
                u64::from(p.own.forge.tasks)
                    .checked_mul(u64::from(p.own.forge.resources_per_task))
                    .expect("resource product")
            },
        ),
    );
    let client_pairs: &[Pair] = &[
        (
            "client entries hold forge entries",
            "forge.client.entries",
            &["forge.entries"][..],
            (|p: &Profile| u64::from(p.own.forge.entries)),
        ),
        (
            "client repositories hold forge repositories",
            "forge.client.repositories",
            &["forge.repositories"][..],
            |p: &Profile| u64::from(p.own.forge.repositories),
        ),
        ("client resources hold forge holds", "forge.client.resources", &["forge.holds"][..], |p: &Profile| {
            u64::from(p.own.forge.holds)
        }),
    ];
    for &(name, figure, sources, formula) in client_pairs {
        add(&mut rows, minimum(row(name, CheckedTogether, together, figure, sources, formula)));
    }
    add(
        &mut rows,
        ceiling(
            row("store tier rows hold changes", HoldsAll, together, "store.tier_landings", &["forge.changes"], |p| {
                twice(u64::from(p.own.forge.changes)).checked_add(1).expect("tier rows")
            }),
            |p| {
                super::item(
                    "forge.changes",
                    value(p, "store.tier_landings")
                        .checked_div(2)
                        .expect("two tier lists")
                        .checked_add(1)
                        .expect("past tier ceiling"),
                )
            },
        ),
    );
    add(
        &mut rows,
        ceiling(
            row(
                "store landing log",
                HoldsAll,
                together,
                "store.landing_log",
                &["forge.landings_kept", "forge.landings_pending"],
                |p| {
                    u64::from(p.own.forge.landings_kept)
                        .checked_add(u64::from(p.own.forge.landings_pending))
                        .expect("landing log")
                },
            ),
            |p| {
                super::item(
                    "forge.landings_pending",
                    value(p, "store.landing_log")
                        .checked_sub(u64::from(p.own.forge.landings_kept))
                        .expect("kept within log")
                        .checked_add(1)
                        .expect("past log ceiling"),
                )
            },
        ),
    );
    add(
        &mut rows,
        ceiling(
            row(
                "store posted comments",
                HoldsAll,
                together,
                "store.posted_comments",
                &["forge.landings_kept", "forge.landings_pending", "forge.landing_members", "forge.changes"],
                |p| {
                    u64::from(p.own.forge.landings_kept)
                        .checked_add(u64::from(p.own.forge.landings_pending))
                        .expect("landing log")
                        .checked_mul(u64::from(p.own.forge.landing_members))
                        .expect("landing members")
                        .checked_add(u64::from(p.own.forge.changes))
                        .expect("posted comments")
                },
            ),
            |p| {
                super::item(
                    "forge.landing_members",
                    value(p, "store.posted_comments")
                        .checked_sub(u64::from(p.own.forge.changes))
                        .expect("changes within comments")
                        .checked_div(
                            u64::from(p.own.forge.landings_kept)
                                .checked_add(u64::from(p.own.forge.landings_pending))
                                .expect("log sum"),
                        )
                        .expect("nonempty log")
                        .checked_add(1)
                        .expect("past comments ceiling"),
                )
            },
        ),
    );
    add(
        &mut rows,
        full_start(row(
            "git output holds conflict paths",
            HoldsAll,
            together,
            "git_component.stdout_bytes",
            &["checkout.conflicts", "checkout.path_bytes"],
            |p| {
                u64::from(p.own.checkout.conflicts)
                    .checked_mul(u64::from(p.own.checkout.path_bytes).checked_add(1).expect("NUL"))
                    .expect("conflict output")
            },
        )),
    );
    // These same-set safety relations explain companion mutations without
    // pretending that a rejected source produced a resolved figure.
    add(
        &mut rows,
        minimum(row(
            "message holds result",
            CheckedTogether,
            "jig protocol/limits.md, message bounds",
            "tasks.message_bytes",
            &["tasks.result_bytes"],
            |p| u64::from(p.jig.core.tasks.result_bytes),
        )),
    );
    add(
        &mut rows,
        minimum(row(
            "loads reply holds loaded bytes",
            CheckedTogether,
            "domain/root.md, loads",
            "root.loads.reply_bytes",
            &["root.loads.bytes"],
            |p| u64::from(p.own.root.loads.bytes),
        )),
    );
    add(
        &mut rows,
        minimum(row(
            "transcript holds resume window",
            CheckedTogether,
            "configuration/derived.md, 5.1",
            "hub.transcript_bytes",
            &["core.resume_bytes"],
            |p| derived::transcript(p.jig.core.resume_bytes).expect("resume window"),
        )),
    );
    add(
        &mut rows,
        ceiling(
            row(
                "store answer ceiling",
                HoldsAll,
                "protocol/store-versions.md, 1; v25 AnswerName",
                "store.answer_bytes",
                &["forge.client.answer_bytes"],
                |p| u64::from(p.own.forge.client.answer_bytes),
            ),
            |p| {
                super::item(
                    "forge.client.answer_bytes",
                    value(p, "store.answer_bytes").checked_add(1).expect("past ceiling"),
                )
            },
        ),
    );
    add(
        &mut rows,
        ceiling(
            row(
                "store tier goal ceiling",
                HoldsAll,
                "protocol/store-versions.md, 1; v25 TierRow.goals",
                "store.goals",
                &["forge.holds"],
                |p| u64::from(p.own.forge.holds),
            ),
            |p| super::item("forge.holds", value(p, "store.goals").checked_add(1).expect("past ceiling")),
        ),
    );
    add(
        &mut rows,
        ceiling(
            full_start(row(
                "worker landing member ceiling",
                HoldsAll,
                "configuration/limits.md, landing members; startup::checks::root",
                "wire.landing_members",
                &["forge.landing_members"],
                |p| u64::from(p.own.forge.landing_members),
            )),
            |p| {
                super::item(
                    "forge.landing_members",
                    value(p, "wire.landing_members").checked_add(1).expect("past ceiling"),
                )
            },
        ),
    );
    add(
        &mut rows,
        ceiling(
            row(
                "store key envelope",
                HoldsAll,
                "configuration/limit-settings.md, 3; store keys::largest",
                "store.key_bytes",
                &["forge.name_bytes"],
                |p| {
                    u64::from(temper_protocol_store::keys::largest(&p.own.forge).expect("key bound"))
                        .checked_add(1)
                        .expect("application prefix")
                },
            ),
            |p| {
                super::item(
                    "forge.name_bytes",
                    value(p, "store.key_bytes")
                        .saturating_sub(14)
                        .checked_div(4)
                        .expect("four name fields")
                        .checked_add(1)
                        .expect("past key envelope"),
                )
            },
        ),
    );
    rows
}

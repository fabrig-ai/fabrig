//! The address table and its reference (temper/docs/design/domain/agent/ask-temper.md, 1;
//! temper/docs/design/protocol/documentation.md, 8).

use skein_lib::{Overflow, Writer};

/// The compact line an unknown address answers with, held to this table and ask.txt by a step test.
pub const ADDRESSES: &str = "me · project · goals [ended] · tasks · task N [tree | activity | turn A.T] · change N · changes · inbox · ways · settings [KIND N | rules | landing-rule N [project] | landing | gates OWNER/NAME | project-ways | project-questions | project-classes | charter-words N | project-words] · revisions · revision N · proposal tN/K or pN/K · help [NAME | search WORDS] · file OWNER/NAME PATH [BRANCH] · more TOKEN";

pub(crate) struct Group {
    pub form: &'static str,
    pub rows: &'static [Address],
}

pub(crate) struct Address {
    pub form: &'static str,
    pub line: &'static str,
    pub next: &'static str,
}

// Groups keep the instructions' order and compact forms; their rows expand each address on the page.
pub(crate) const GROUPS: &[Group] = &[
    Group {
        form: "me",
        rows: &[Address {
            form: "me",
            line: "Your name, project role and budget; what waits on you in this project, counted by kind and oldest first.",
            next: "inbox · project · goals · tasks",
        }],
    },
    Group {
        form: "project",
        rows: &[Address {
            form: "project",
            line: "The project, its repositories and people; this period and its budget, reserved, spent and left; held tasks and what they wait on.",
            next: "goals · tasks · changes · inbox · settings",
        }],
    },
    Group {
        form: "goals [ended]",
        rows: &[
            Address {
                form: "goals",
                line: "Live goals by priority: number, title, priority, state, what each waits on and its report headline.",
                next: "task N · task N tree · goals ended",
            },
            Address {
                form: "goals ended",
                line: "Ended goals, latest ended first: number, title, priority, state and report headline.",
                next: "task N · goals",
            },
        ],
    },
    Group {
        form: "tasks",
        rows: &[Address {
            form: "tasks",
            line: "Live tasks, latest active first: number, title, kind, executor, state, what each waits on and last activity time.",
            next: "task N · goals · inbox",
        }],
    },
    Group {
        form: "task N [tree | activity | turn A.T]",
        rows: &[
            Address {
                form: "task N",
                line: "The task, requester, executor, state, hold, spend, budget, delegate count and first asked lines; a goal report or ended result whole.",
                next: "task N tree · task N activity · change N · tasks",
            },
            Address {
                form: "task N tree",
                line: "Delegates, live then ended: number, title, executor, state, spend and how many delegates each has.",
                next: "task N · task N activity · task N tree",
            },
            Address {
                form: "task N activity",
                line: "Latest events, oldest first: time, actor, words, runs and turns with a line per call; whether earlier events were not read.",
                next: "task N turn A.T · task N · task N tree",
            },
            Address {
                form: "task N turn A.T",
                line: "One turn whole: brief headings, words, calls with inputs and results; a turn past the read bound by its size.",
                next: "task N activity · task N",
            },
        ],
    },
    Group {
        form: "change N",
        rows: &[Address {
            form: "change N",
            line: "The change: repository, branch, landing branch, pull request, step, checks, gates, approvals, repairs, questions, holds and queue or turn place.",
            next: "task N · task N activity · changes · settings gates OWNER/NAME",
        }],
    },
    Group {
        form: "changes",
        rows: &[Address {
            form: "changes",
            line: "Landing queues in order, the owning change first; landings by turn under way; latest landed changes, a line each.",
            next: "change N · settings landing · ways",
        }],
    },
    Group {
        form: "inbox",
        rows: &[Address {
            form: "inbox",
            line: "What waits on you in this project, oldest first: kind, task number and title, first line; whether more wait. Answer each on Needs you.",
            next: "task N · me",
        }],
    },
    Group {
        form: "ways",
        rows: &[Address {
            form: "ways",
            line: "Stock ways over this project: name, current mark, headline, line and path; your own path where no stock way runs; whether you may change settings.",
            next: "settings rules · settings landing · settings project-ways · settings project-words · settings charter-words N · help settings",
        }],
    },
    Group {
        form: "settings [KIND N | rules | landing-rule N [project] | landing | gates OWNER/NAME | project-ways | project-questions | project-classes | charter-words N | project-words]",
        rows: &[
            Address {
                form: "settings",
                line: "Newest revision; settings and their last revisions, held revisions, waiting proposals; project repositories, people, forge accounts and offer.",
                next: "settings charter N · settings template N · settings policy N · settings model N · settings forge-charter N · settings rules · settings landing-rule N [project] · settings landing · settings gates OWNER/NAME · settings project-ways · settings project-questions · settings project-classes · revisions · help settings",
            },
            Address {
                form: "settings charter N",
                line: "The charter whole, at the newest revision.",
                next: "settings · revisions · help settings-fields",
            },
            Address {
                form: "settings template N",
                line: "The template whole, at the newest revision.",
                next: "settings · revisions · help settings-fields",
            },
            Address {
                form: "settings policy N",
                line: "The policy whole, at the newest revision.",
                next: "settings · revisions · help settings-fields",
            },
            Address {
                form: "settings model N",
                line: "The model whole, at the newest revision.",
                next: "settings · revisions · help settings-fields",
            },
            Address {
                form: "settings forge-charter N",
                line: "The forge charter whole, at the newest revision.",
                next: "settings · revisions · help settings-fields",
            },
            Address {
                form: "settings dependency N",
                line: "A dependency of this project, whole, at the newest revision; no stock default.",
                next: "settings · revisions · help settings-fields",
            },
            Address {
                form: "settings follow N",
                line: "A follow of this project, whole, at the newest revision; no stock default.",
                next: "settings · revisions · help settings-fields",
            },
            Address {
                form: "settings template-goal N",
                line: "Template work: whether its coordinator designs first and makes a close-out; with none stored, stock planning and no close-out, marked so.",
                next: "settings template N · ways · help settings-fields",
            },
            Address {
                form: "settings rules",
                line: "The rules whole, at the newest revision.",
                next: "settings · revisions · help settings-fields",
            },
            Address {
                form: "settings landing-rule N [project]",
                line: "A landing rule whole: the deployment's, or, with project, this project's.",
                next: "settings landing · settings · help landing-rule",
            },
            Address {
                form: "settings landing",
                line: "Project landing: queue or turn, merge job, after-landing retries and bound, repository gate and tier counts; set in configuration, never by proposal.",
                next: "settings gates OWNER/NAME · settings landing-rule N [project] · help landing",
            },
            Address {
                form: "settings gates OWNER/NAME",
                line: "Repository gates and tiers: workflows, ignored paths, blocking, artifacts, followed branches and descriptions; set in configuration, never by proposal.",
                next: "settings landing · file OWNER/NAME PATH",
            },
            Address {
                form: "settings project-ways",
                line: "This project's ways; stock where none is stored, marked so.",
                next: "settings · revisions · help settings-fields",
            },
            Address {
                form: "settings project-questions",
                line: "This project's questions; stock where none is stored, marked so.",
                next: "settings · revisions · help settings-fields",
            },
            Address {
                form: "settings project-classes",
                line: "This project's classes; stock where none is stored, marked so.",
                next: "settings · revisions · help settings-fields",
            },
            Address {
                form: "settings charter-words N",
                line: "A charter's words; stock where none is stored, marked so, or that it has none.",
                next: "settings · revisions · help settings-fields",
            },
            Address {
                form: "settings project-words",
                line: "This project's words, or that it has none.",
                next: "settings · revisions · help settings-fields",
            },
        ],
    },
    Group {
        form: "revisions",
        rows: &[Address {
            form: "revisions",
            line: "The newest history page: each revision's decider, time, source proposal and items without values.",
            next: "revision N · settings · help revision",
        }],
    },
    Group {
        form: "revision N",
        rows: &[Address {
            form: "revision N",
            line: "One revision: decider, time, source proposal, whether running work moved, and items with values or sizes.",
            next: "revisions · proposal tN/K · proposal pN/K · settings",
        }],
    },
    Group {
        form: "proposal tN/K or pN/K",
        rows: &[Address {
            form: "proposal tN/K or pN/K",
            line: "A task's or party's settings proposal: reason, items, review against newest settings while waiting, state and outcome.",
            next: "revision N · revisions · help proposal",
        }],
    },
    Group {
        form: "help [NAME | search WORDS]",
        rows: &[
            Address { form: "help", line: "The index of temper's pages.", next: "help NAME · help search WORDS" },
            Address {
                form: "help NAME",
                line: "The page whole, name and title first; for an unknown name, the sections holding its words.",
                next: "help · help search WORDS",
            },
            Address {
                form: "help search WORDS",
                line: "Sections holding every word, a line each with page, heading and text; nothing, saying so; more than eight, asking for narrower words.",
                next: "help NAME",
            },
        ],
    },
    Group {
        form: "file OWNER/NAME PATH [BRANCH]",
        rows: &[Address {
            form: "file OWNER/NAME PATH [BRANCH]",
            line: "File bytes or directory entries, head first, in a readable project repository; the branch defaults to the repository's default branch.",
            next: "file OWNER/NAME PATH [BRANCH] · settings gates OWNER/NAME",
        }],
    },
    Group {
        form: "more TOKEN",
        rows: &[Address {
            form: "more TOKEN",
            line: "The next part of an answer, reading its address again at the token's byte offset.",
            next: "The address's next addresses.",
        }],
    },
];

pub(crate) fn forms(out: &mut Writer) -> Result<(), Overflow> {
    for (at, group) in GROUPS.iter().enumerate() {
        if at != 0 {
            out.put(" · ".as_bytes())?;
        }
        out.put(group.form.as_bytes())?;
    }
    Ok(())
}

pub(super) fn following(form: &str) -> &'static [u8] {
    for group in GROUPS {
        for row in group.rows {
            if row.form == form {
                return row.next.as_bytes();
            }
        }
    }
    unreachable!("each answer names a row of the address table")
}

/// The reference page `see`, whole (protocol/documentation.md, 8).
pub fn see_reference(out: &mut Writer) -> Result<(), Overflow> {
    out.put(b"# Ask temper's addresses\n\nKind: reference\n\nRead temper with see, by address, in your project. Each answer ends with what to read next. When a part shows more: TOKEN, read see more TOKEN for the next part.\n")?;
    for group in GROUPS {
        for row in group.rows {
            out.put(b"\n## ")?;
            out.put(row.form.as_bytes())?;
            out.put(b"\n\n")?;
            out.put(row.line.as_bytes())?;
            out.put(b"\n\nNext: ")?;
            out.put(row.next.as_bytes())?;
            out.put(b"\n")?;
        }
    }
    Ok(())
}

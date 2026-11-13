//! The segments in order (temper/docs/design/domain/forge/change/branch-path.md).
use super::{Actor, Inputs, Opens, STOPS, Start, Step, Stop};
use super::{
    approvals,
    cards::{agent_card, append, card, coordinator_card, fact, number, stop, text},
    rules,
    scopes::Coverage,
};
use alloc::boxed::Box;
use skein_lib::{List, bytes};
use temper_engine_domain_forge_change::Way;
use temper_engine_domain_forge_vocabulary::{Follows, settings::Held};

const CODER: &[u8] = b"the coding agent";
const REPAIR: &[u8] = "an agent · repairs what a check or a review found, then goes back through them".as_bytes();

/// `A, B and C`: the parts joined by ", ", the last by " and "; none for no part.
fn joined(parts: &[&[u8]]) -> Option<Box<[u8]>> {
    let (last, preceding) = parts.split_last()?;
    let mut words = List::uncapped();
    let mut separator = b"".as_slice();
    for part in preceding {
        words.push(separator).expect("card words fit memory");
        words.push(*part).expect("card words fit memory");
        separator = b", ";
    }
    if !preceding.is_empty() {
        words.push(b" and ").expect("card words fit memory");
    }
    words.push(*last).expect("card words fit memory");
    Some(bytes::concat(words.as_slice()))
}

struct Drawing {
    stops: List<Stop>,
    repaired: bool,
}

impl Drawing {
    fn push(&mut self, stop: Stop) {
        if self.stops.as_slice().len() < STOPS.saturating_sub(1) {
            self.stops.push(stop).expect("stops before the final bar");
        }
    }

    fn agent(&mut self, inputs: &Inputs, charter: u32, step: Step, words: &[u8], who: &[u8], name: &[u8]) {
        let card = agent_card(inputs, charter, who, name);
        let needed = step == Step::Repair || step == Step::Resolve;
        let mut drawn = stop(Actor::Agent, step, words, Opens::Charter(charter), card);
        drawn.needed = needed;
        self.push(drawn);
    }

    fn repair(&mut self, inputs: &Inputs, tests: bool, review: bool) -> bool {
        if self.repaired {
            return false;
        }
        let card = agent_card(inputs, inputs.producer, REPAIR, CODER);
        let mut drawn = stop(Actor::Agent, Step::Repair, b"repair", Opens::Charter(inputs.producer), card);
        drawn.needed = true;
        drawn.over = repair_over(tests, review).into();
        self.push(drawn);
        self.repaired = true;
        true
    }

    fn ci(&mut self, inputs: &Inputs, held: &Held, coverage: &Coverage) {
        let mut workflows = List::uncapped();
        let mut contexts = List::uncapped();
        if let Some(repository) = &inputs.repository {
            for workflow in &repository.workflows {
                if bytes::is_utf8(workflow) {
                    workflows.push(workflow.as_ref()).expect("repository workflows fit memory");
                }
            }
            for context in &repository.contexts {
                if bytes::is_utf8(context) {
                    contexts.push(context.as_ref()).expect("required checks fit memory");
                }
            }
        }
        let checks = match joined(contexts.as_slice()) {
            Some(contexts) => bytes::concat(&[b"the CI workflows' checks ", &contexts, b", at the change's head"]),
            None => b"every check the repository's CI workflows report, at the change's head".as_slice().into(),
        };
        let mut facts = List::uncapped();
        if let Some(workflows) = joined(workflows.as_slice()) {
            facts.push(fact(b"Workflows", &workflows)).expect("two CI facts at most");
        }
        facts.push(fact(b"Checks", &checks)).expect("two CI facts at most");
        let card = card("code · holds a change until CI passes".as_bytes(), b"CI", facts.into_boxed());
        let mut drawn = stop(Actor::Code, Step::Check, b"check", opens_rule(held), card);
        drawn.over = coverage.over(inputs);
        self.push(drawn);
    }

    fn code(&mut self, step: Step, words: &[u8], opens: Opens, who: &[u8], name: &[u8], does: &[u8]) {
        self.push(stop(Actor::Code, step, words, opens, card(who, name, Box::new([fact(b"Does", does)]))));
    }

    fn resolve(&mut self, inputs: &Inputs) {
        self.agent(
            inputs,
            inputs.producer,
            Step::Resolve,
            b"resolve",
            "an agent · settles a conflict with its branch, then goes back through the checks".as_bytes(),
            CODER,
        );
    }

    fn segment(&mut self, inputs: &Inputs, branch: &[u8], by: Way, goal: bool) {
        let held = rules::covering(inputs, branch);
        let mut ci = None;
        let mut ci_coverage = Coverage::new();
        let mut update = false;
        for held in held.as_slice() {
            if held.rule.enforce {
                if held.rule.ci {
                    ci_coverage.add(inputs, &held.rule);
                    if ci.is_none() {
                        ci = Some(held);
                    }
                }
                update |= held.rule.up_to_date;
            }
        }
        let head_ci = match by {
            Way::Queue => match &inputs.repository {
                Some(repository) => repository.ci,
                None => false,
            },
            Way::Turn => false,
        };
        if !head_ci {
            ci = None;
        }
        if let Some(held) = ci {
            self.ci(inputs, held, &ci_coverage);
        }
        let gates = rules::gates(inputs, held.as_slice());
        let mut tests = ci.is_some();
        let mut review = false;
        for gate in gates.as_slice() {
            tests |= gate.check;
            review |= !gate.check;
            let (charter, step, words, who, name) = if gate.check {
                let who = bytes::concat(&["an agent · runs the repository's check ".as_bytes(), &number(gate.number)]);
                (inputs.producer, Step::Check, b"check".as_slice(), who, CODER)
            } else {
                let who = if gate.blocking {
                    "an agent · holds a change until it approves"
                } else {
                    "an agent · advises; holds nothing"
                };
                (inputs.reviewer, Step::Review, b"review".as_slice(), who.as_bytes().into(), b"the reviewer".as_slice())
            };
            let card = agent_card(inputs, charter, &who, name);
            let mut drawn = stop(Actor::Agent, step, words, Opens::Gate { number: gate.number, charter }, card);
            drawn.over = gate.coverage.over(inputs);
            self.push(drawn);
        }
        if (ci.is_some() || !gates.is_empty())
            && self.repair(inputs, tests, review)
            && let Some(held) = ci
        {
            self.ci(inputs, held, &ci_coverage);
        }
        self.approvals(inputs, held.as_slice());
        let mut checked = tests;
        if goal && let Some(repository) = &inputs.repository {
            for tier in &repository.tiers {
                checked |= follows(&tier.follows, branch, goal);
            }
        }
        let close_charter = match inputs.start {
            Start::Goal { charter, close_out, .. } => {
                if !goal && close_out {
                    Some(charter)
                } else {
                    None
                }
            }
            Start::Chat => None,
        };
        let preceding =
            if close_charter.is_some() { Some(core::mem::replace(&mut self.stops, List::uncapped())) } else { None };
        self.land(inputs, branch, by, update, goal, checked, review);
        if let Some(mut preceding) = preceding {
            // Keep the landing and its close-out before the final bar even when earlier stops fill the path.
            let room = STOPS.saturating_sub(self.stops.as_slice().len()).saturating_sub(2);
            let keep = preceding.as_slice().len().min(room);
            preceding.truncate(u32::try_from(keep).expect("path stop bound fits u32"));
            let landing = core::mem::replace(&mut self.stops, preceding);
            for stop in landing.into_boxed() {
                self.push(stop);
            }
        }
        if let Some(charter) = close_charter {
            let lands = bytes::concat(&[b"as one change into ", branch, b", by ", branch, b"'s rules"]);
            let card = coordinator_card(
                inputs,
                charter,
                "an agent · after the landing, brings what the goal leaves up to date".as_bytes(),
                fact(b"Lands", &lands),
            );
            self.push(stop(Actor::Agent, Step::CloseOut, b"close-out", Opens::Charter(charter), card));
        }
        self.tiers(inputs, branch, goal);
        if goal && inputs.merge_job {
            self.catch_up(inputs);
        }
    }

    #[expect(clippy::too_many_arguments, reason = "a segment carries its branch, landing way and preceding checks")]
    #[expect(
        clippy::fn_params_excessive_bools,
        reason = "the segment names independent update, goal, test and review facts"
    )]
    fn land(&mut self, inputs: &Inputs, branch: &[u8], by: Way, update: bool, goal: bool, tests: bool, review: bool) {
        match by {
            Way::Queue => {
                if update {
                    let change = if !goal && inputs.start != Start::Chat {
                        b"the goal's branch".as_slice()
                    } else {
                        b"the change".as_slice()
                    };
                    let who = bytes::concat(&[
                        "code · ".as_bytes(),
                        branch,
                        b" has moved in a way that clashes with ",
                        change,
                    ]);
                    let when = bytes::concat(&[
                        change,
                        b" reaches the front of the queue: ",
                        branch,
                        b" is merged into it, never by an agent",
                    ]);
                    let clean =
                        if tests { b"it is checked again and lands".as_slice() } else { b"it lands".as_slice() };
                    let card = card(
                        &who,
                        b"the queue",
                        Box::new([
                            fact(b"When", &when),
                            fact(b"Merged cleanly", clean),
                            fact(b"A clash", b"it leaves the queue and goes to resolve"),
                        ]),
                    );
                    let mut drawn = stop(Actor::Code, Step::Conflict, b"conflict", Opens::Queue, card);
                    drawn.needed = true;
                    self.push(drawn);
                    self.resolve(inputs);
                }
                let who = if goal {
                    "code · lands each change into the goal's branch".as_bytes().into()
                } else if inputs.start != Start::Chat {
                    bytes::concat(&["code · lands the goal on ".as_bytes(), branch])
                } else {
                    Box::from("code · lands the change".as_bytes())
                };
                let card =
                    card(&who, b"the queue", Box::new([fact(b"Does", b"merges exactly the commit its checks saw")]));
                let mut drawn = stop(Actor::Code, Step::Merge, b"merge", Opens::Queue, card);
                if goal {
                    drawn.over = b"into goal".as_slice().into();
                }
                self.push(drawn);
            }
            Way::Turn => {
                self.code(
                    Step::Prepare,
                    b"prepare",
                    Opens::Turn,
                    "code · puts the change on its branch's tip".as_bytes(),
                    b"the turn",
                    b"rebases the change onto its branch's tip",
                );
                self.resolve(inputs);
                if let Some(repository) = &inputs.repository
                    && !repository.gates.is_empty()
                {
                    let mut names = List::uncapped();
                    for name in &repository.gates {
                        if !names.is_empty() {
                            append(&mut names, b", ");
                        }
                        append(&mut names, name);
                    }
                    let card = card(
                        "code · runs the test gates at the candidate".as_bytes(),
                        b"the turn",
                        Box::new([fact(b"Gates", names.as_slice())]),
                    );
                    self.push(stop(Actor::Code, Step::Prove, b"prove", Opens::Turn, card));
                    self.repair(inputs, true, review);
                }
                self.code(
                    Step::Turn,
                    b"turn",
                    Opens::Turn,
                    "code · lands the change".as_bytes(),
                    b"the turn",
                    b"moves the branch to exactly the proved candidate",
                );
            }
        }
    }

    fn catch_up(&mut self, inputs: &Inputs) {
        let branch = inputs.branch.as_ref();
        let who = bytes::concat(&["code · takes ".as_bytes(), branch, b" into the goal's branch"]);
        let does = bytes::concat(&[
            b"merges ",
            branch,
            b"'s tip into a branch made at the goal's tip, landing as a change into it",
        ]);
        let when = bytes::concat(&[b"the coordinator's call, as ", branch, b"'s landings touch the goal's open work"]);
        let card = card(
            &who,
            b"the merge job",
            Box::new([fact(b"Does", &does), fact(b"When", &when), fact(b"A conflict", b"goes to resolve")]),
        );
        let mut drawn = stop(Actor::Code, Step::CatchUp, b"catch up", Opens::Nothing, card);
        drawn.needed = true;
        self.push(drawn);
    }

    fn approvals(&mut self, inputs: &Inputs, held: &[Held]) {
        for approval in &approvals::collect(inputs, held, false) {
            let words = approvals::approval_words(&approval.role, approval.count, !approval.coverage.all);
            let card = approvals::approval_card(inputs, held, approval);
            let mut drawn = stop(Actor::Person, Step::Approve, &words, opens_rule(&approval.first), card);
            drawn.over = approval.coverage.over(inputs);
            self.push(drawn);
        }
    }

    fn tiers(&mut self, inputs: &Inputs, branch: &[u8], goal: bool) {
        if let Some(repository) = &inputs.repository {
            for tier in &repository.tiers {
                if !follows(&tier.follows, branch, goal) {
                    continue;
                }
                let (step, words, who, when) = if goal {
                    (
                        Step::Check,
                        b"check".as_slice(),
                        "code · checks the goal's branch",
                        b" on the goal's branch".as_slice(),
                    )
                } else {
                    (Step::Verify, b"verify".as_slice(), "code · checks what landed", b" after each landing".as_slice())
                };
                let runs = bytes::concat(&[b"the tier ", &text(&tier.name, super::VALUE), when]);
                let mut facts = List::uncapped();
                facts.push(fact(b"Runs", &runs)).expect("tier runs");
                if !tier.description.is_empty() {
                    facts.push(fact(b"Checks", &tier.description)).expect("tier description");
                }
                let card = card(who.as_bytes(), &tier.name, facts.into_boxed());
                self.push(stop(Actor::Code, step, words, Opens::Tier(tier.name.clone()), card));
                if goal && !self.repaired {
                    match inputs.start {
                        Start::Goal { charter, .. } => {
                            let card = agent_card(
                                inputs,
                                charter,
                                "an agent · repairs a red on the goal's branch".as_bytes(),
                                b"the coordinator",
                            );
                            let mut drawn = stop(Actor::Agent, Step::Repair, b"repair", Opens::Charter(charter), card);
                            drawn.needed = true;
                            drawn.over = b"tests".as_slice().into();
                            self.push(drawn);
                            self.repaired = true;
                        }
                        Start::Chat => {}
                    }
                }
            }
        }
    }
}

fn follows(follows: &Follows, branch: &[u8], goal: bool) -> bool {
    match follows {
        Follows::Landing => !goal,
        Follows::Branch(name) => name.as_ref() == branch,
        Follows::Goals => goal,
    }
}

fn repair_over(tests: bool, review: bool) -> &'static [u8] {
    match (tests, review) {
        (true, true) => b"tests, review",
        (true, false) => b"tests",
        (false, true) => b"review",
        (false, false) => b"",
    }
}

fn opens_rule(held: &Held) -> Opens {
    Opens::Rule { kind: if held.scope == 0 { 1 } else { 2 }, scope: held.scope, number: held.number }
}

fn bar(branch: &[u8], goal: bool) -> Stop {
    let who = if goal {
        "a branch · holds the goal's changes until it lands whole"
    } else {
        "a branch · where the change lands"
    };
    stop(Actor::Bar, Step::Branch, branch, Opens::Nothing, card(who.as_bytes(), branch, Box::default()))
}

pub(super) fn stops(inputs: &Inputs) -> Box<[Stop]> {
    let mut drawing = Drawing { stops: List::uncapped(), repaired: false };
    match inputs.start {
        Start::Chat => {
            let card =
                card("a person · asks for the change".as_bytes(), b"you", Box::new([fact(b"Starts", b"a chat")]));
            drawing.push(stop(Actor::Person, Step::Ask, b"you ask", Opens::Nothing, card));
            drawing.agent(
                inputs,
                inputs.producer,
                Step::Produce,
                b"produce",
                "an agent · makes the change".as_bytes(),
                CODER,
            );
        }
        Start::Goal { charter, design, .. } => {
            let card = card(
                "a person · writes the draft with an agent".as_bytes(),
                b"you",
                Box::new([fact(b"Starts", b"a goal for a draft")]),
            );
            drawing.push(stop(Actor::Person, Step::Draft, b"draft", Opens::Nothing, card));
            if design {
                let card = coordinator_card(
                    inputs,
                    charter,
                    "an agent · designs the goal, then cuts it into changes".as_bytes(),
                    fact(
                        b"Its order",
                        b"the design is delivered first, as a change into the goal's branch; the changes start from it and merge after it",
                    ),
                );
                drawing.push(stop(Actor::Agent, Step::Design, b"design", Opens::Charter(charter), card));
            } else {
                drawing.agent(
                    inputs,
                    charter,
                    Step::Plan,
                    b"plan",
                    "an agent · cuts the draft into changes".as_bytes(),
                    b"the coordinator",
                );
            }
            drawing.agent(
                inputs,
                inputs.producer,
                Step::ProduceMany,
                "produce ×n".as_bytes(),
                "agents · make the goal's changes, in parallel".as_bytes(),
                CODER,
            );
            drawing.segment(inputs, b"temper/0/g0", Way::Queue, true);
            drawing.push(bar(b"goal branch", true));
        }
    }
    drawing.segment(inputs, &inputs.branch, inputs.by, false);
    drawing.stops.push(bar(&inputs.branch, false)).expect("room reserved for the landing branch's bar");
    drawing.stops.into_boxed()
}

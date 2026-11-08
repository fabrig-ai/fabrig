//! The segments in order (temper/docs/design/domain/forge.md, 8.13).
use super::{Actor, Inputs, Opens, STOPS, Start, Step, Stop};
use super::{
    cards::{agent_card, append, card, fact, number, stop, text},
    rules,
};
use alloc::boxed::Box;
use skein_lib::{List, bytes};
use temper_engine_domain_forge::{Follows, settings::Held};
use temper_engine_domain_forge_change::Way;

const CODER: &[u8] = b"the coding agent";
const REPAIR: &[u8] = "an agent · repairs what a check or a review found, then goes back through them".as_bytes();

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
        self.push(stop(Actor::Agent, step, words, Opens::Charter(charter), card));
    }

    fn repair(&mut self, inputs: &Inputs) -> bool {
        if self.repaired {
            return false;
        }
        self.agent(inputs, inputs.producer, Step::Repair, b"repair", REPAIR, CODER);
        self.repaired = true;
        true
    }

    fn ci(&mut self, inputs: &Inputs, held: &Held) {
        let mut contexts = List::uncapped();
        if let Some(repository) = &inputs.repository {
            for context in &repository.contexts {
                if bytes::is_utf8(context) {
                    contexts.push(context.as_ref()).expect("required checks fit memory");
                }
            }
        }
        let checks = match contexts.as_slice().split_last() {
            Some((last, preceding)) => {
                let mut parts = List::uncapped();
                parts.push(b"the CI workflows' checks ".as_slice()).expect("card words fit memory");
                let mut separator = b"".as_slice();
                for context in preceding {
                    parts.push(separator).expect("card words fit memory");
                    parts.push(*context).expect("card words fit memory");
                    separator = b", ";
                }
                if !preceding.is_empty() {
                    parts.push(b" and ").expect("card words fit memory");
                }
                parts.push(*last).expect("card words fit memory");
                parts.push(b", at the change's head").expect("card words fit memory");
                bytes::concat(parts.as_slice())
            }
            None => b"every check the repository's CI workflows report, at the change's head".as_slice().into(),
        };
        let card =
            card("code · holds a change until CI passes".as_bytes(), b"CI", Box::new([fact(b"Checks", &checks)]));
        self.push(stop(Actor::Code, Step::Check, b"check", opens_rule(held), card));
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
            "an agent · settles a conflict with its branch".as_bytes(),
            CODER,
        );
    }

    fn segment(&mut self, inputs: &Inputs, branch: &[u8], by: Way, goal: bool) {
        let held = rules::covering(inputs, branch);
        let mut ci = None;
        let mut update = false;
        for held in held.as_slice() {
            if held.rule.enforce {
                if held.rule.ci && ci.is_none() {
                    ci = Some(held);
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
            self.ci(inputs, held);
        }
        let gates = rules::gates(inputs, held.as_slice());
        for gate in gates.as_slice() {
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
            self.push(stop(Actor::Agent, step, words, Opens::Gate { number: gate.number, charter }, card));
        }
        if (ci.is_some() || !gates.is_empty())
            && self.repair(inputs)
            && let Some(held) = ci
        {
            self.ci(inputs, held);
        }
        for held in held.as_slice() {
            if held.rule.enforce {
                self.approvals(inputs, held);
            }
        }
        self.land(inputs, by, update);
        self.tiers(inputs, branch, goal);
    }

    fn land(&mut self, inputs: &Inputs, by: Way, update: bool) {
        match by {
            Way::Queue => {
                if update {
                    self.code(
                        Step::Update,
                        b"update",
                        Opens::Queue,
                        "code · brings the change up to date with its branch".as_bytes(),
                        b"the queue",
                        b"merges the branch's tip into the change, never by an agent",
                    );
                    self.resolve(inputs);
                }
                self.code(
                    Step::Merge,
                    b"merge",
                    Opens::Queue,
                    "code · lands the change".as_bytes(),
                    b"the queue",
                    b"merges exactly the commit its checks saw",
                );
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
                    self.repair(inputs);
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

    fn approvals(&mut self, inputs: &Inputs, held: &Held) {
        for approval in &held.rule.approvals {
            let mut name = bytes::concat(&[b"role ", &number(approval.role)]);
            let mut yours = false;
            for role in &inputs.roles {
                if role.number == approval.role {
                    name.clone_from(&role.name);
                    yours = role.yours;
                    break;
                }
            }
            let mut words = if yours {
                Box::from(b"you approve".as_slice())
            } else {
                let article = match name.first() {
                    Some(b'a' | b'e' | b'i' | b'o' | b'u' | b'A' | b'E' | b'I' | b'O' | b'U') => b"an ".as_slice(),
                    Some(_) | None => b"a ".as_slice(),
                };
                bytes::concat(&[article, &text(&name, super::WORDS), b" approves"])
            };
            if approval.people > 1 {
                words = bytes::concat(&[&words, " ×".as_bytes(), &number(approval.people)]);
            }
            let asks = bytes::concat(&[&number(approval.people), b" of them"]);
            let card = card(
                "a person · decides whether it lands".as_bytes(),
                &name,
                Box::new([fact(b"Role", &name), fact(b"Asks", &asks)]),
            );
            self.push(stop(Actor::Person, Step::Approve, &words, opens_rule(held), card));
        }
    }

    fn tiers(&mut self, inputs: &Inputs, branch: &[u8], goal: bool) {
        if let Some(repository) = &inputs.repository {
            for tier in &repository.tiers {
                let follows = match &tier.follows {
                    Follows::Landing => !goal,
                    Follows::Branch(name) => name.as_ref() == branch,
                    Follows::Goals => goal,
                };
                if !follows {
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
                let card = card(who.as_bytes(), &tier.name, Box::new([fact(b"Runs", &runs)]));
                self.push(stop(Actor::Code, step, words, Opens::Tier(tier.name.clone()), card));
                if goal && !self.repaired {
                    match inputs.start {
                        Start::Goal { charter } => {
                            self.agent(
                                inputs,
                                charter,
                                Step::Repair,
                                b"repair",
                                "an agent · repairs a red on the goal's branch".as_bytes(),
                                b"the coordinator",
                            );
                            self.repaired = true;
                        }
                        Start::Chat => {}
                    }
                }
            }
        }
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
        Start::Goal { charter } => {
            let card = card(
                "a person · writes the draft with an agent".as_bytes(),
                b"you",
                Box::new([fact(b"Starts", b"a goal for a draft")]),
            );
            drawing.push(stop(Actor::Person, Step::Draft, b"draft", Opens::Nothing, card));
            drawing.agent(
                inputs,
                charter,
                Step::Plan,
                b"plan",
                "an agent · cuts the draft into changes".as_bytes(),
                b"the coordinator",
            );
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

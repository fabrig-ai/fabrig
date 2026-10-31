//! Exact policy edit review fragments; client/domain/view.md, 7.
//! Numeric flags and literal segments remain supplied facts, not reconstructed policy.
use crate::words::Words;
use crate::words::phrases::jig::JigPhrase;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role, Run};

use jig_client::contract::{OwnerPattern, OwnerPatternEnd, PolicyEdit, PolicyGuard};

fn pattern(b: &mut Builder, value: &OwnerPattern, words: &Words, text_limit: u32) {
    let (phrase, bytes) = match &value.last {
        OwnerPatternEnd::Exact(bytes) => (crate::words::authority_words::AuthorityPhrase::Exact, bytes),
        OwnerPatternEnd::Open(bytes) => (crate::words::authority_words::AuthorityPhrase::Open, bytes),
    };
    let label = words.book().say(words.language(), phrase.id(), &[]).expect("checked terminal label").text;
    let reserved = u32::try_from(label.len())
        .expect("bounded ending")
        .checked_add(crate::words::literal_segment::display_bytes(bytes).expect("bounded segment"))
        .expect("bounded path words");
    crate::words::literal_segment::path(
        b,
        value.segments.as_slice(),
        words,
        crate::words::literal_segment::PathDisplay { text: text_limit, reserved, keep_last: false },
    );
    b.text(Run::Plain, &label);
    crate::words::literal_segment::render(b, bytes);
}
fn effect(b: &mut Builder, connector: u16, kind: u16, words: &Words) {
    words.text(b, JigPhrase::PolicyEffect, &[Hole::Number(u64::from(connector)), Hole::Number(u64::from(kind))]);
}
/// Display one whole edit without inventing role names or parsing resource names.
pub fn render(b: &mut Builder, value: &PolicyEdit, words: &Words, text_limit: u32, rows: u32) {
    let name = match value {
        PolicyEdit::Templates { .. } => return,
        PolicyEdit::ProjectSpend { period_spend } | PolicyEdit::RoleSpend { period_spend, .. } => {
            words.say(JigPhrase::PolicyPeriodSpend, &[Hole::Amount(*period_spend)])
        }
        PolicyEdit::Role(role) => words.say(JigPhrase::PolicyEditRole, &[Hole::Number(u64::from(role.number))]),
        PolicyEdit::Requirements { .. } => words.say(JigPhrase::PolicyRequirements, &[]),
        PolicyEdit::Projections { .. } => words.say(JigPhrase::PolicyProjections, &[]),
        PolicyEdit::Permissions { .. } => words.say(JigPhrase::PolicyPermissions, &[]),
    };
    b.open(Role::Region);
    b.name(&name);
    match value {
        PolicyEdit::Templates { .. } => {}
        PolicyEdit::ProjectSpend { period_spend } => {
            words.text(b, JigPhrase::PolicyPeriodSpend, &[Hole::Amount(*period_spend)]);
        }
        PolicyEdit::RoleSpend { role, period_spend } => {
            words.text(b, JigPhrase::RoleNumber, &[Hole::Number(u64::from(*role))]);
            words.text(b, JigPhrase::PolicyPeriodSpend, &[Hole::Amount(*period_spend)]);
        }
        PolicyEdit::Role(role) => {
            words.text(b, JigPhrase::PolicyEditRole, &[Hole::Number(u64::from(role.number))]);
            crate::tree::fragments::authority::render_words(b, &role.authority, words, text_limit, rows);
            words.text(b, JigPhrase::PolicyPeriodSpend, &[Hole::Amount(role.period_spend)]);
            words.text(b, JigPhrase::PolicyRequests, &[Hole::Number(u64::from(role.requests))]);
            words.text(b, JigPhrase::PolicyDecides, &[Hole::Number(u64::from(role.decides))]);
        }
        PolicyEdit::Requirements { requirements } => {
            words.text(b, JigPhrase::PolicyRequirements, &[]);
            for requirement in requirements {
                effect(b, requirement.connector, requirement.kind, words);
                pattern(b, &requirement.pattern, words, text_limit);
                words.text(
                    b,
                    JigPhrase::PolicyJudge,
                    &[
                        Hole::Number(u64::from(requirement.judge.connector)),
                        Hole::Number(u64::from(requirement.judge.requirement)),
                        Hole::Number(u64::from(requirement.judge.parameters)),
                    ],
                );
                match requirement.guard {
                    PolicyGuard::Guarded => words.text(b, JigPhrase::PolicyGuarded, &[]),
                    PolicyGuard::Observed { freshness } => {
                        words.text(b, JigPhrase::PolicyObserved, &[Hole::Number(freshness.as_nanos())]);
                    }
                }
                words.text(
                    b,
                    if requirement.must_be_guarded {
                        JigPhrase::PolicyMustGuard
                    } else {
                        JigPhrase::PolicyGuardNotRequired
                    },
                    &[],
                );
            }
        }
        PolicyEdit::Projections { grants } => {
            words.text(b, JigPhrase::PolicyProjections, &[]);
            for grant in grants {
                effect(b, grant.connector, grant.kind, words);
                pattern(b, &grant.pattern, words, text_limit);
            }
        }
        PolicyEdit::Permissions { mappings } => {
            words.text(b, JigPhrase::PolicyPermissions, &[]);
            for mapping in mappings {
                words.text(
                    b,
                    JigPhrase::PolicyPermission,
                    &[
                        Hole::Number(u64::from(mapping.connector)),
                        Hole::Number(u64::from(mapping.permission)),
                        Hole::Number(u64::from(mapping.role)),
                    ],
                );
            }
        }
    }
    b.close();
}

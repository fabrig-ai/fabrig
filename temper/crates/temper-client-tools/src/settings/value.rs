//! Temper's inline values (temper/docs/design/client/ctl/tools.md, 2 and 3).
use skein_json::writer::Encoder;
use skein_lib::Reader;
use temper_client_protocol_wire::v1 as w;
pub(crate) fn render(kind: u32, bytes: &[u8], out: &mut Encoder) -> bool {
    let mut reader = Reader::new(bytes);
    match kind {
        1 | 2 => {
            let Ok(value) = w::LandingRule::decode(&w::CEILINGS, &mut reader) else { return false };
            if !reader.is_empty() {
                return false;
            }
            render_rule(&value, out);
        }
        3 => {
            let Ok(value) = w::ForgeCharter::decode(&w::CEILINGS, &mut reader) else { return false };
            if !reader.is_empty() {
                return false;
            }
            out.unsigned(u64::from(value.charter()));
        }
        11 => {
            let Ok(value) = w::TemplateGoal::decode(&w::CEILINGS, &mut reader) else { return false };
            if !reader.is_empty() {
                return false;
            }
            out.object_start();
            out.key(b"design");
            out.boolean(value.design());
            out.key(b"close_out");
            out.boolean(value.close_out());
            out.object_end();
        }
        6 => {
            let Ok(value) = w::ProjectQuestions::decode(&w::CEILINGS, &mut reader) else { return false };
            if !reader.is_empty() {
                return false;
            }
            out.string(match value.route() {
                w::QuestionsRoute::None => b"none",
                w::QuestionsRoute::Foreman => b"foreman",
                w::QuestionsRoute::Up => b"up",
            });
        }
        7 => {
            let Ok(value) = w::ProjectWays::decode(&w::CEILINGS, &mut reader) else { return false };
            if !reader.is_empty() {
                return false;
            }
            out.object_start();
            out.key(b"drafts");
            out.boolean(value.drafts());
            out.key(b"goals");
            out.boolean(value.goals());
            out.key(b"changes");
            out.boolean(value.changes());
            out.key(b"way");
            out.unsigned(u64::from(value.way()));
            out.object_end();
        }
        8 => {
            return settings_form::render::application_value(kind, bytes, out);
        }
        _ => return false,
    }
    true
}
fn render_freshness(value: &w::RuleFreshness, out: &mut Encoder) {
    out.string(match value {
        w::RuleFreshness::Exact => b"exact",
        w::RuleFreshness::Clean => b"clean",
    });
}
fn render_rule(value: &w::LandingRule, out: &mut Encoder) {
    out.object_start();
    out.key(b"pattern");
    out.object_start();
    out.key(b"segments");
    out.array_start();
    for segment in value.pattern().segments() {
        crate::jig::render::document::hex(segment, out);
    }
    out.array_end();
    out.key(b"last");
    out.object_start();
    match value.pattern().last() {
        w::LandingLast::Exact(value) => {
            out.key(b"exact");
            crate::jig::render::document::hex(value.value(), out);
        }
        w::LandingLast::Open(value) => {
            out.key(b"open");
            crate::jig::render::document::hex(value.value(), out);
        }
    }
    out.object_end();
    out.object_end();
    out.key(b"enforce");
    out.boolean(value.enforce());
    out.key(b"ci");
    out.boolean(value.ci());
    out.key(b"up_to_date");
    out.boolean(value.up_to_date());
    out.key(b"gates");
    out.array_start();
    for gate in value.gates() {
        out.object_start();
        out.key(b"number");
        out.unsigned(u64::from(gate.number()));
        out.key(b"blocking");
        out.boolean(gate.blocking());
        out.key(b"freshness");
        render_freshness(gate.freshness(), out);
        out.object_end();
    }
    out.array_end();
    out.key(b"approvals");
    out.array_start();
    for approval in value.approvals() {
        out.object_start();
        out.key(b"role");
        out.unsigned(u64::from(approval.role()));
        out.key(b"people");
        out.unsigned(u64::from(approval.people()));
        out.key(b"freshness");
        render_freshness(approval.freshness(), out);
        out.object_end();
    }
    out.array_end();
    if !value.sets().is_empty() {
        out.key(b"sets");
        out.array_start();
        for set in value.sets() {
            out.unsigned(u64::from(*set));
        }
        out.array_end();
    }
    out.object_end();
}

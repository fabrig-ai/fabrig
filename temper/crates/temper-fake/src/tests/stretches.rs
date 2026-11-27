//! Standalone stretches and draw menus (temper/docs/design/protocol/setup/playground.md, 9).
use super::{api, llm, llm_document, query};
use serde_json::{Value, json, to_vec};
use skein_fake_llm_domain::{self as domain, Domain, Event, Request};
use skein_lib::{Env, Queue, ReplyTo, Time, Token, Wall};

fn menu() -> Value {
    json!({
        "arguments":[
            {"task":"{{#task}}","question":"{{question}}"},
            {"task":"{{#task}}","question":"{{question}}","detail":true}
        ],
        "invalid":[{"name":null,"arguments":"["},{"name":"unknown","arguments":"null"}],
        "slots":[
            {"name":"task","before":"Task ","after":", spec","unissued":"999999",
             "weight":0,"numeric":{"low":1,"high":100},"with":null,"foreign":0},
            {"name":"question","before":"Question ","after":"","unissued":"never",
             "weight":0,"numeric":null,"with":"task","foreign":1000}
        ],
        "calls":1
    })
}

fn stretch() -> Value {
    json!({"lines":[{"drawn":{"rounds":2,"tools":["read_task"]}}],"finish":"tool_calls"})
}

fn tools() -> Box<[api::ToolSpec]> {
    [b"read_task".as_slice(), b"finish"]
        .map(|name| api::ToolSpec { name: name.into(), description: Box::new([]), parameters: b"{}".as_slice().into() })
        .into()
}

fn ask(domain: &mut Domain, limits: domain::Config, query: &api::Query) -> Result<api::Answer, api::Failure> {
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let token = Token::new(1);
    let mut out = Queue::with_capacity(1);
    domain::step(
        domain,
        &env,
        Event::Call { reply_to: ReplyTo::new(token), call: token, query: query.clone() },
        &mut out,
    );
    let deadline = domain.next_deadline().expect("reply scheduled");
    domain::fire(domain, &Env { now: deadline, ..env }, &mut out);
    let Some(Request::Reply { result, .. }) = out.pop() else { panic!("one whole reply") };
    domain.reclaim();
    result
}

fn step_through(document: &llm::Document, with_candidates: bool) -> Vec<api::Answer> {
    let limits = document.limits();
    assert_eq!(limits.calls_per_answer, 1);
    assert_eq!(limits.tool_rounds, 0);
    let mut domain = document.domain(23);
    let system: &[u8] = if with_candidates { b"Question foreign\nother text" } else { b"" };
    let mut query = query(system, b"read a task");
    query.tools = tools();
    query.choice = api::ToolChoice::Auto;
    let mut answers = Vec::new();
    for (index, expected_name) in
        [b"read_task".as_slice(), b"read_task", b"read_task", b"finish"].into_iter().enumerate()
    {
        let answer = ask(&mut domain, limits, &query).expect("a scripted or drawn answer");
        assert_eq!(answer.finish, api::Finish::ToolCalls);
        let [api::Part::ToolCall { id, name, arguments }] = &*answer.parts else {
            panic!("exactly one call: {answer:?}")
        };
        assert_eq!(name.as_ref(), expected_name);
        let body: Value = serde_json::from_slice(arguments).expect("valid drawn or scripted arguments");
        match index {
            0 => assert_eq!(body, json!({"task":42})),
            1 | 2 => {
                let (task, question) = if with_candidates { (42, "foreign") } else { (999_999, "never") };
                assert_eq!(body["task"], json!(task));
                assert_eq!(body["question"], json!(question));
            }
            3 => assert_eq!(body, json!({"fixed":true})),
            _ => unreachable!("four fixture rounds"),
        }
        let output: &[u8] =
            if index == 0 && with_candidates { b"Task 42, spec\nQuestion local" } else { b"no references" };
        let mut history = query.messages.into_vec();
        history.push(api::Message { role: api::Role::Assistant, parts: answer.parts.clone() });
        history.push(api::Message {
            role: api::Role::User,
            parts: Box::new([api::Part::ToolOutput { id: id.clone(), output: output.into(), is_error: false }]),
        });
        query.messages = history.into_boxed_slice();
        answers.push(answer);
    }
    answers
}

#[test]
fn drawn_stretch_plays_between_scripted_calls() {
    let otherwise = json!([
        {"lines":[{"call":"read_task","arguments":{"task":42}}],"finish":"tool_calls","tokens":4},
        stretch(),
        {"lines":[{"call":"finish","arguments":{"fixed":true}}],"finish":"tool_calls","tokens":4}
    ]);
    let mut value = llm_document();
    value["menu"] = menu();
    value["otherwise"] = otherwise.clone();
    let direct = llm::Document::decode(&to_vec(&value).unwrap()).expect("inline stretch");
    value["otherwise"] = json!(["replaced"]);
    let document = to_vec(&value).unwrap();
    let scripts = to_vec(&json!({"otherwise":otherwise})).unwrap();
    let replaced = llm::Document::decode_with_scripts(&document, Some(&scripts)).expect("stretch in --scripts");
    let mut join = llm::ScriptJoin::empty();
    join.push(&document, &scripts).expect("stretch in a joined source");
    let joined = llm::Document::decode_with_scripts(&document, Some(&join.into_bytes())).expect("joined stretch");
    for with_candidates in [true, false] {
        let expected = step_through(&direct, with_candidates);
        assert_eq!(step_through(&replaced, with_candidates), expected);
        assert_eq!(step_through(&joined, with_candidates), expected);
    }
}

fn refusal(value: &Value, scripts: Option<&Value>, expected: &str) {
    let scripts = scripts.map(|given| to_vec(given).unwrap());
    let error = llm::Document::decode_with_scripts(&to_vec(value).unwrap(), scripts.as_deref()).unwrap_err();
    assert_eq!(error.line().as_ref(), expected.as_bytes());
}

#[test]
fn stretch_refusals_name_the_turn_or_member() {
    let mut value = llm_document();
    value["otherwise"] = json!([stretch()]);
    refusal(&value, None, "otherwise[0]: stretch requires menu\n");
    refusal(&llm_document(), Some(&json!({"otherwise":[stretch()]})), "otherwise[0]: stretch requires menu\n");
    value["otherwise"] = json!([]);
    value["cued"] = json!([{"cue":"stage","turns":[stretch()]}]);
    refusal(&value, None, "cued[0].turns[0]: stretch requires menu\n");
    value["cued"] = json!([]);
    value["menu"] = menu();
    let mut mixed = stretch();
    mixed["lines"].as_array_mut().unwrap().push(json!({"text":"another line"}));
    for (turn, message) in [
        (mixed, "otherwise[0]: invalid turn\n"),
        (json!({"lines":stretch()["lines"],"finish":"stop"}), "otherwise[0]: invalid turn\n"),
        (json!({"lines":stretch()["lines"],"finish":"tool_calls","tokens":1}), "otherwise[0]: invalid turn\n"),
    ] {
        value["otherwise"] = json!([turn]);
        refusal(&value, None, message);
    }
    for (member, given, message) in [
        ("rounds", json!(0), "rounds: must be at least 1"),
        ("rounds", json!(65), "rounds: past its bound"),
        ("tools", json!([]), "tools: must be non-empty"),
        ("tools", json!(["a", "b", "c", "d", "e", "f", "g", "h", "i"]), "tools: past its bound"),
        ("tools", json!(["read_task", "read_task"]), "tools[1]: must be unique"),
    ] {
        value["otherwise"] = json!([stretch()]);
        value["otherwise"][0]["lines"][0]["drawn"][member] = given;
        refusal(&value, None, &format!("otherwise[0].lines[0].drawn.{message}\n"));
    }
    let scripts = json!({"asked":[{"cue":"stage","turns":[stretch()],"inputs":[{"cue":"","turn":0}]}]});
    refusal(&value, Some(&scripts), "asked[0].inputs[0].turn: must not index a stretch\n");
}

#[test]
fn menu_refusals_name_the_member() {
    let mut value = llm_document();
    for (member, given, message) in [
        ("arguments", json!([]), "arguments: must be non-empty"),
        ("arguments", json!(vec![json!({}); 17]), "arguments: past its bound"),
        ("arguments", json!([[]]), "arguments[0]: expected a JSON object"),
        ("invalid", json!(vec![menu()["invalid"][0].clone(); 9]), "invalid: past its bound"),
        ("slots", json!(vec![menu()["slots"][0].clone(); 9]), "slots: past its bound"),
        ("calls", json!(0), "calls: must be at least 1"),
        ("calls", json!(17), "calls: past its bound"),
    ] {
        value["menu"] = menu();
        value["menu"][member] = given;
        refusal(&value, None, &format!("menu.{message}\n"));
    }
    value["menu"] = menu();
    value["menu"]["slots"][0]["before"] = json!("x".repeat(257));
    refusal(&value, None, "menu.slots: invalid slots or their use in arguments, or past joint byte bound\n");
    value["menu"] = menu();
    refusal(&value, Some(&json!({"menu":menu()})), "menu: unknown member\n");
}

#[test]
fn deployment_refuses_menus_and_stretches() {
    let mut scripts = json!({"asked":[],"otherwise":[stretch()]});
    assert_eq!(
        crate::demo::decode_scripts(&to_vec(&scripts).unwrap()).unwrap_err().line().as_ref(),
        b"demo.otherwise[0].lines[0].drawn: unknown member\n"
    );
    scripts["otherwise"] = json!(["fallback"]);
    scripts["menu"] = menu();
    assert_eq!(
        crate::demo::decode_scripts(&to_vec(&scripts).unwrap()).unwrap_err().line().as_ref(),
        b"demo.menu: unknown member\n"
    );
}

#[test]
fn a_menu_does_not_change_answers_outside_stretches() {
    let mut value = llm_document();
    value["otherwise"] = json!([]);
    let old = llm::Document::decode(&to_vec(&value).unwrap()).unwrap();
    value["menu"] = menu();
    let with_menu = llm::Document::decode(&to_vec(&value).unwrap()).unwrap();
    let mut query = query(b"", b"unmatched");
    query.max_tokens = llm::LIMITS.answer_tokens;
    query.tools = tools();
    query.choice = api::ToolChoice::Auto;
    let done = ask(&mut old.domain(23), old.limits(), &query).unwrap();
    assert_eq!(&*done.parts, &[api::Part::Text { text: b"done".as_slice().into() }]);
    assert_eq!(ask(&mut with_menu.domain(23), with_menu.limits(), &query), Ok(done));
    let old = old.with_strict(true);
    let with_menu = with_menu.with_strict(true);
    let failure = ask(&mut old.domain(23), old.limits(), &query).unwrap_err();
    assert_eq!(failure.error, api::Error::InvalidRequest);
    assert_eq!(&*failure.report.code, b"no_script");
    assert_eq!(ask(&mut with_menu.domain(23), with_menu.limits(), &query), Err(failure));
}

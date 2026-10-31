//! The scripted agent's protocol calls and observed answers.
use super::Peers;
use jig_conformance::Input;
use jig_domain_host as host;
use jig_ops_domain as root;

pub(super) fn agent(peers: &mut Peers, request: host::ToAgent) -> Vec<Input<root::Event, root::Record>> {
    if let host::ToAgent::Answer { answer: host::CallAnswer::Relayed { error, text }, .. } = &request {
        peers.call_answers.push((*error, text.clone()));
    }
    if let Some(state) = &mut peers.scenario { state.agent(request) } else { peers.agents.answer(request) }
}

impl Peers {
    /// Ask through the scripted agent using a fresh smith call name.
    pub fn tool_call(&mut self, tool: &[u8], input: &[u8], writes: bool) -> root::Event {
        self.agents.tool_call(tool, input, writes)
    }
}

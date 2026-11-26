//! Task-owned semantic authority and accounting carriers
//! (domain/tasks/README.md, sections 2–3; domain/authority.md, sections 3–6, and domain/authority-budgets.md, section 1).
//! Root translates permission values for pure authority checks. Tasks owns
//! concrete financial counters and admits bounded payload shape; it neither
//! interprets connector names nor serves tools or implements a move route.

pub use jig_domain_contract::ops::{
    Authority, AuthorityExecutor, Budget, Delegation, Funder, Grant, Last, Models, Numbers, Pattern, ResourceScope,
    Scopes, Tools,
};

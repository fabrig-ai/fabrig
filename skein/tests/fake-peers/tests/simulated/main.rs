#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

#[path = "../suite/forge.rs"]
mod forge;
#[path = "../suite/github_memory.rs"]
mod github_memory;
#[path = "../suite/llm.rs"]
mod llm;
#[path = "../suite/oauth.rs"]
mod oauth;
#[path = "../suite/push.rs"]
mod push;

#[path = "../suite/fuzzy"]
mod fuzzy {
    mod push;
}

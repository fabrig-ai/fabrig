#[path = "../suite/pages.rs"]
mod pages;

#[cfg(debug_assertions)]
mod refused_add;
mod serve;
#[cfg(debug_assertions)]
mod tree;

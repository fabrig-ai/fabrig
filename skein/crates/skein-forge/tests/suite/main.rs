#![cfg(test)]

#[path = "../common/mod.rs"]
pub mod common;

pub mod fixtures;
mod forgejo_bounds;
mod forgejo_create_file;
mod forgejo_create_repository;
mod forgejo_deliveries;
mod forgejo_files;
mod forgejo_items;
mod forgejo_repo_search;
mod forgejo_repo_setup;
mod forgejo_reviews;
mod forgejo_setup;
mod forgejo_start;
mod forgejo_user_repos;
mod forgejo_writes;
mod github_items;
mod github_plans;
mod github_start_plans;

mod fuzzy {
    mod forgejo;
}

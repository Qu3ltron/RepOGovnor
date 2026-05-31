mod activation;
mod cli;
mod hook_io;
mod install;
mod landing;
mod manifest;
mod metrics;
mod model;
mod mutation_hook;
mod plan_contract;
mod policy;
mod receipts;
mod registry_io;
mod release_checks;
mod reports;
mod runtime;
mod schema;
mod source_limit;
mod status_checks;
mod validation;
mod verifiers;
mod verify_chain;

pub(crate) use runtime::{
    append_event, load_registry, normalize_relative_path, timestamp, truncate_detail,
};

fn main() {
    std::process::exit(cli::main_entry());
}

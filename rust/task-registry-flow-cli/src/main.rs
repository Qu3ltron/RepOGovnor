mod cli;
mod hook_io;
mod install;
mod manifest;
mod model;
mod mutation_hook;
mod plan_contract;
mod policy;
mod receipts;
mod release_checks;
mod reports;
mod runtime;
mod schema;
mod source_limit;
mod status_checks;
mod verifiers;

pub(crate) use runtime::{
    append_event, load_registry, normalize_relative_path, timestamp, truncate_detail,
};

fn main() {
    std::process::exit(cli::main_entry());
}

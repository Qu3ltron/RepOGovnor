mod activation;
mod backlog_check;
mod cli;
mod cost_evidence;
mod cost_ingest;
mod hook_io;
mod install;
mod landing;
mod manifest;
mod metrics;
mod model;
mod model_attribution;
mod mutation_hook;
mod plan_contract;
mod policy;
mod receipts;
mod registry_io;
mod release_checks;
mod reports;
mod reviewer_report;
mod runtime;
mod schema;
mod source_limit;
mod status_checks;
mod validation;
mod verifiers;
mod verify_chain;
mod version_check;

pub(crate) use runtime::{
    append_event, load_registry, normalize_relative_path, timestamp, truncate_detail,
};

fn main() {
    std::process::exit(cli::main_entry());
}

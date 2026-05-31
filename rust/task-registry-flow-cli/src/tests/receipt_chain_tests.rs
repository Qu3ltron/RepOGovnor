use super::*;
use crate::model::EVENTS_PATH;
use fs2::FileExt;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

#[test]
fn receipt_chain_concurrent_writers_preserve_chain() {
    let root = temp_root("receipt-chain-concurrent");
    let writer_count = 8;
    let barrier = Arc::new(Barrier::new(writer_count));
    let mut handles = Vec::new();

    for index in 0..writer_count {
        let root = root.clone();
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            barrier.wait();
            for _ in 0..100 {
                let result = append_event(
                    &root,
                    EventRecord::new(
                        timestamp(),
                        CliCommand::Metrics,
                        EventOutcome::Ok,
                        index as u128,
                        format!("concurrent event {index}"),
                    ),
                );
                match result {
                    Ok(()) => return,
                    Err(error) if error.contains("locked by another process") => {
                        thread::sleep(Duration::from_millis(2));
                    }
                    Err(error) => panic!("{error}"),
                }
            }
            panic!("writer {index} never acquired receipt lock");
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let result = crate::verify_chain::run_verify_chain(&root, &[]).unwrap();
    assert!(result.contains("intact"), "{result}");
}

#[test]
fn receipt_chain_locked_file_fails_append() {
    let root = temp_root("receipt-chain-locked");
    let events_path = root.join(EVENTS_PATH);
    fs::create_dir_all(events_path.parent().unwrap()).unwrap();
    let file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&events_path)
        .unwrap();
    file.try_lock_exclusive().unwrap();

    let error = append_event(
        &root,
        EventRecord::new(
            timestamp(),
            CliCommand::Metrics,
            EventOutcome::Ok,
            1,
            "locked append".to_string(),
        ),
    )
    .expect_err("locked events file must fail append");

    assert!(error.contains("locked by another process"), "{error}");
}

#[test]
fn receipt_chain_explicit_read_only_policy() {
    assert!(!crate::receipts::should_record(CliCommand::Validate, false));
    assert!(crate::receipts::should_record(CliCommand::Validate, true));
    assert!(crate::receipts::should_record(CliCommand::Status, false));
}

#[test]
fn receipt_chain_required_failure_fails_command() {
    let root = temp_root("receipt-chain-required-failure");
    let events_path = root.join(EVENTS_PATH);
    fs::create_dir_all(events_path.parent().unwrap()).unwrap();
    let file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&events_path)
        .unwrap();
    file.try_lock_exclusive().unwrap();
    let result = Ok("mutating command ok".to_string());

    let error =
        crate::cli::record_command_receipt(&root, CliCommand::Status, EventOutcome::Ok, 1, &result)
            .expect_err("required receipt append failure must be surfaced");

    assert!(
        error.summary().contains("required receipt append failed"),
        "{}",
        error.summary()
    );
}

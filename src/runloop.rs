use crate::config;
use crate::config::Config;
use anyhow::Result;
use clokwerk::{ScheduleHandle, Scheduler, TimeUnits};
use std::{backtrace::BacktraceStatus, fs, sync::{Arc, Mutex, MutexGuard}, time::SystemTime};
use std::{path::Path, time::Duration};

// Running state for polling. This lets us keep
// track of things that are happening while the program
// is running.
struct RunState {
    config: Option<Config>,
    config_last_modified: Option<SystemTime>,
    config_len: Option<u64>,
}

impl RunState {
    pub fn new() -> RunState {
        RunState {
            config: None,
            config_last_modified: None,
            config_len: None,
        }
    }
}

fn run(mut run_state: MutexGuard<RunState>) {
    match run_with_result(&mut run_state) {
        Err(e) => {
            eprintln!("Error in run: {}", e);
            if e.backtrace().status() == BacktraceStatus::Captured {
              eprintln!("{}", e.backtrace());
            }
        }
        Ok(()) => {}
    }
}

fn run_with_result(run_state: &mut MutexGuard<RunState>) -> Result<()> {
    check_config_loaded(run_state)?;

    Ok(())
}

fn get_config_file_metadata() -> Result<Option<(SystemTime, u64)>> {
    let path = config::get_config_path();
    match path.exists() {
        false => Ok(None),
        true => {
            let metadata = fs::metadata(path)?;
            Ok(Some((metadata.modified()?, metadata.len())))
        }
    }
}

fn is_file_modified(
    path: &Path,
    last_read_ts: Option<SystemTime>,
    last_len: Option<u64>,
) -> Result<bool> {
    let modified = if !path.exists() {
        false
    } else if let (Some(modified), Some(len)) = (last_read_ts, last_len) {
        let metadata = fs::metadata(path)?;
        metadata.len() != len || metadata.modified()? != modified
    } else {
        false
    };

    Ok(modified)
}

fn check_config_loaded(run_state: &mut MutexGuard<RunState>) -> Result<()> {
    let load = match run_state.config {
        None => true,
        Some(_) => is_file_modified(
            &config::get_config_path(),
            run_state.config_last_modified,
            run_state.config_len,
        )?,
    };

    if load {
        let config = config::load()?;

        run_state.config = Some(config);
        if let Some((modified, len)) = get_config_file_metadata()? {
            run_state.config_last_modified = Some(modified);
            run_state.config_len = Some(len);
        }
    }

    Ok(())
}

pub fn start() -> ScheduleHandle {
    println!("Starting run loop");
    let run_state = Arc::new(Mutex::new(RunState::new()));

    let mut scheduler = Scheduler::new();

    let rs = Arc::clone(&run_state);
    scheduler
        .every(2.seconds())
        .run(move || run(rs.lock().unwrap()));
    scheduler.watch_thread(Duration::from_millis(1000))
}

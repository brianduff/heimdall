use crate::config::Config;
use crate::config::OpenPeriod;
use crate::config::{self, Instant};
use anyhow::Result;
use chrono::{DateTime, Datelike, Local, Timelike};
use clokwerk::{ScheduleHandle, Scheduler, TimeUnits};
use std::{
  backtrace::BacktraceStatus,
  collections::HashMap,
  fs,
  sync::{Arc, Mutex, MutexGuard},
  time::SystemTime,
};
use std::{path::Path, time::Duration};

// Running state for polling. This lets us keep
// track of things that are happening while the program
// is running.
struct RunState {
  config: Option<Config>,
  config_last_modified: Option<SystemTime>,
  config_len: Option<u64>,
  user_state: HashMap<String, UserInMemoryState>,
}

struct UserInMemoryState {
  is_locked: Option<bool>,
}

impl RunState {
  pub fn new() -> RunState {
    RunState {
      config: None,
      config_last_modified: None,
      config_len: None,
      user_state: HashMap::new(),
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

fn get_start_of_week(now: &DateTime<Local>) -> Option<DateTime<Local>> {
  let weekday = now.weekday().num_days_from_sunday();
  let sunday = *now - chrono::Duration::days(weekday.into());
  sunday
    .with_hour(0)
    .and_then(|d| d.with_minute(0))
    .and_then(|d| d.with_hour(0))
    .and_then(|d| d.with_minute(0))
    .and_then(|d| d.with_second(0))
    .and_then(|d| d.with_nanosecond(0))
}

fn from_instant(start_of_week: &DateTime<Local>, instant: &Instant) -> Option<DateTime<Local>> {
  (*start_of_week + chrono::Duration::days(instant.weekday.into()))
    .with_hour(instant.hour.into())
    .and_then(|d| d.with_minute(instant.minute.into()))
}

fn get_local_period(
  now: &DateTime<Local>,
  period: &OpenPeriod,
) -> Option<(DateTime<Local>, DateTime<Local>)> {
  if let Some(start_of_week) = get_start_of_week(now) {
    let start = from_instant(&start_of_week, &period.start);
    let end = from_instant(&start_of_week, &period.end);

    if let (Some(start), Some(end)) = (start, end) {
      return Some((start, end));
    }
  }

  None
}

/// Given a schedule, returns true if we're in a locked down period, false otherwise.
fn is_lockdown_time(now: DateTime<Local>, schedule: &config::Schedule) -> bool {
  for period in &schedule.open_periods {
    if let Some((start, end)) = get_local_period(&now, &period) {
      if now >= start && now < end {
        return false;
      }
    }
  }

  true
}

fn run_with_result(run_state: &mut MutexGuard<RunState>) -> Result<()> {
  check_config_loaded(run_state)?;

  if let Some(config) = &run_state.config {
    for (user, user_config) in &config.user_config {
      // Check if the user should be locked out right now.
      if is_lockdown_time(Local::now(), &user_config.schedule) {}
    }
  }

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

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_start_of_week() {
    println!("{:?}", get_start_of_week(&Local::now()));
  }
}

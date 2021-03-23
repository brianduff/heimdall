use crate::config::Config;
use crate::config::OpenPeriod;
use crate::config::{self, Instant, Schedule};
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

// fn is_user_locked(run_state: &mut MutexGuard<RunState>, user: &str) -> bool {
//   run_state.user_state.insert(user.to_owned(), UserInMemoryState { is_locked: None});
//   true
// }

fn set_locked(user: &str, locked: bool) -> Result<()> {
  println!("User {} locked={}", user, locked);
  Ok(())

}

fn run_with_result(run_state: &mut RunState) -> Result<()> {
  check_config_loaded(run_state)?;

  if let Some(config) = &mut run_state.config {
    for (user, user_config) in &mut config.user_config {
      // Check if the user should be locked out right now.
      let state = run_state
        .user_state
        .entry(user.to_owned())
        .or_insert(UserInMemoryState { is_locked: None });

      let open_period = find_max_open_period(Local::now(), &user_config.schedule);
      let should_lock = open_period.is_some();
      let is_locked = state.is_locked.unwrap_or(!should_lock);

      if should_lock != is_locked {
        set_locked(user, should_lock)?;
        state.is_locked = Some(should_lock);
      }
    }
  }

  Ok(())
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

/// Given a schedule, returns the longest open period containing now.
fn find_max_open_period(now: DateTime<Local>, schedule: &config::Schedule) -> Option<&OpenPeriod> {
  let mut max_period_duration = chrono::Duration::zero();
  let mut max_period = None;

  for period in &schedule.open_periods {
    if let Some((start, end)) = get_local_period(&now, &period) {
      if now >= start && now < end {
        let period_duration = end - start;
        if period_duration > max_period_duration {
          max_period_duration = period_duration;
          max_period = Some(period)
        }
      }
    }
  }

  max_period
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

fn check_config_loaded(run_state: &mut RunState) -> Result<()> {
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
  use chrono::TimeZone;
  use config::{OpenPeriod, Schedule};

  use super::*;
  #[test]
  fn test_start_of_week() {
    println!("{:?}", get_start_of_week(&Local::now()));
  }

  fn create_schedule(start: (u8, u8, u8), end: (u8, u8, u8)) -> Schedule {
    Schedule {
      open_periods: vec![OpenPeriod {
        start: Instant {
          weekday: start.0,
          hour: start.1,
          minute: start.2,
        },
        end: Instant {
          weekday: end.0,
          hour: end.1,
          minute: end.2,
        },
        note: "".to_owned(),
      }],
    }
  }

  #[test]
  fn test_find_open_period() {
    let schedule = create_schedule((3, 14, 45), (3, 15, 0));

    let now = Local.ymd(2020, 1, 1).and_hms(14, 30, 0);
    assert_eq!(false, find_max_open_period(now, &schedule).is_some());

    let now = Local.ymd(2020, 1, 1).and_hms(14, 45, 0);
    assert_eq!(true, find_max_open_period(now, &schedule).is_some());
  }
}

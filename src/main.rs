#![feature(proc_macro_hygiene, decl_macro, let_chains, backtrace)]

#[macro_use]
extern crate rocket;

use anyhow::Result;
use rocket_contrib::serve::StaticFiles;

mod api;
mod config;
mod os;
mod runloop;



fn main() -> Result<()> {
  // TODO use a flag.
  let static_path = if cfg!(debug_assertions) {
    "static"
  } else {
    "/etc/heimdall/static"
  };

  let _scheduler = runloop::start();

  rocket::ignite()
    .mount("/api/", api::get_routes())
    .mount("/", StaticFiles::from(static_path))
    .launch();

  Ok(())
}

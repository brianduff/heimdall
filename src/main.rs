#![feature(proc_macro_hygiene, decl_macro, let_chains, backtrace)]

#[macro_use]
extern crate rocket;

use anyhow::Result;
use env_logger::Env;
use rocket_contrib::serve::StaticFiles;

mod api;
mod config;
mod constants;
mod os;
mod runloop;
mod scratch;

use log::info;

fn main() -> Result<()> {
  env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

  // let mut bar = sysbar::Sysbar::new("Hello");
  // // bar.add_quit_item("Quit");
  // bar.display();

  info!("Starting heimdall");

  // TODO use a flag.
  let static_path = if cfg!(debug_assertions) {
    "static"
  } else {
    "/usr/local/etc/heimdall/static"
  };

  let _scheduler = runloop::start();

  println!("HELLO");
  // bar.set_title("Starting Rocket");
  rocket::ignite()
    .mount("/api/", api::get_routes())
    .mount("/", StaticFiles::from(static_path))
    .launch();

  Ok(())
}

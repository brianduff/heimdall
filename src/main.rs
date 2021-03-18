#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket_contrib::serve::StaticFiles;

mod api;
mod config;
mod os;

fn main() {

  // TODO use a flag.
  let static_path = if cfg!(debug_assertions) {
    "static"
  } else {
    "/etc/heimdall/static"
  };

  rocket::ignite()
    .mount("/api/", api::get_routes())
    .mount("/", StaticFiles::from(static_path))
    .launch();
}

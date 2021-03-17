#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use rocket_contrib::serve::StaticFiles;

mod api;
mod config;

fn main() {
    rocket::ignite()
        .mount("/api/", api::get_routes())
        .mount("/", StaticFiles::from("/etc/heimdall/static"))
        .launch();
}

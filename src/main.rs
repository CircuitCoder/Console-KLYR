// Features
#![feature(never_type)]

extern crate actix;
extern crate actix_redis;
extern crate actix_web;
extern crate futures;
extern crate serde;
extern crate tokio;
#[macro_use]
extern crate serde_derive;
extern crate env_logger;
extern crate serde_json;
#[macro_use]
extern crate log;

mod data;
mod handler;
mod storage;

use actix::{Arbiter, System};
use actix_web::{fs, fs::NamedFile, http::Method, server, App, HttpRequest, Result};
use futures::Future;
use handler::State;

fn get_index(_: &HttpRequest<State>) -> Result<NamedFile> {
	Ok(NamedFile::open("./static-dist/index.html")?)
}

fn build_app() -> App<State> {
	App::with_state(Default::default())
		.resource("/api/posts", |r| {
			r.method(Method::GET).f(handler::post::list_posts);
			r.method(Method::POST).f(handler::post::new_post)
		})
		.resource("/api/posts/{id}", |r| {
			r.method(Method::PUT).with(handler::post::update_post);
			r.method(Method::DELETE).f(handler::post::delete_post)
		})
		.resource("/api/posts/{id}/accept", |r| {
			r.method(Method::PUT).f(handler::post::accept_post)
		})
		.handler("/static", fs::StaticFiles::new("./static-dist").unwrap())
		.default_resource(|r| r.method(Method::GET).f(get_index))
}

fn main() {
	env_logger::init();
	// Init database
	let bootstrapper = System::new("setup");
	let bootstrap = storage::setup();
	let handle = bootstrap.then(|_| {
		System::current().stop();
		futures::future::ok(())
	});
	Arbiter::spawn(handle);
	bootstrapper.run();

	info!("Starting server...");
	let server = server::new(build_app).bind("127.0.0.1:8088").unwrap();
	server.run();
}

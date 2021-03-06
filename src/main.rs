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
extern crate pulldown_cmark;
extern crate rand;
extern crate ring;

mod data;
mod handler;
mod storage;
mod util;

use actix::{Arbiter, System};
use actix_web::fs::StaticFileConfig;
use actix_web::middleware::session::{CookieSessionBackend, SessionStorage};
use actix_web::middleware::Logger;
use actix_web::{fs, fs::NamedFile, http::Method, server, App, HttpRequest, Result, HttpResponse, Error};
use futures::Future;
use handler::State;
use std::fs::File;
//use rand::distributions::Alphanumeric;
//use rand::{self, Rng};

#[derive(Default)]
struct NoCacheConfig;
impl StaticFileConfig for NoCacheConfig {
	fn is_use_etag() -> bool {
		false
	}

	fn is_use_last_modifier() -> bool {
		false
	}
}

fn get_index(_: &HttpRequest<State>) -> Result<NamedFile<NoCacheConfig>> {
	Ok(NamedFile::open_with_config(
		"./static-dist/index.html",
		NoCacheConfig,
	)?)
}

/*
fn post_file(req: &HttpRequest<State>) -> Box<Future<Item=HttpResponse, Error=Error>> {
	let id = rand::thread_rng()
		.sample_iter(&Alphanumeric)
		.take(16)
		.collect::<String>();
}
*/

fn build_app() -> App<State> {
	App::with_state(Default::default())
		.middleware(Logger::default())
		.middleware(
			SessionStorage::new(
				CookieSessionBackend::signed(&[0; 32]).secure(false))) // TODO: investigate
		.scope("/api", |scope| {
			scope
				.resource("/auth", |r| {
					r.method(Method::POST).f(handler::auth::login);
					r.method(Method::DELETE).f(handler::auth::logout);
					r.method(Method::GET).f(handler::auth::fetch)
				})
				.nested("/posts", |scope| {
					scope
						.resource("", |r| {
							r.method(Method::GET).f(handler::post::list_posts);
							r.method(Method::POST).f(handler::post::new_post)
						})
						.resource("/{id}", |r| {
							r.method(Method::GET).f(handler::post::fetch_post);
							r.method(Method::PUT).with(handler::post::update_post);
							r.method(Method::DELETE).f(handler::post::delete_post)
						})
						.resource("/{id}/accept", |r| {
							r.method(Method::PUT).f(handler::post::accept_post)
						})
						.resource("/{id}/reject", |r| {
							r.method(Method::PUT).with(handler::post::reject_post)
						})
						.resource("/{id}/digest", |r| {
							r.method(Method::GET).f(handler::post::digest_post)
						})
				})
				.nested("/steps", |scope| {
					scope
					.resource("", |r| {
						r.method(Method::POST).f(handler::step::create)
					})
					.resource("/staged", |r| {
						r.method(Method::GET).f(handler::step::fetch_staged)
					})
					.resource("/{id}", |r| {
						r.method(Method::GET).f(handler::step::detail)
					})
					.resource("/{id}/assign", |r| {
						r.method(Method::PUT).f(handler::step::assign)
					})
					.resource("/{id}/resolve", |r| {
						r.method(Method::POST).f(handler::step::resolve)
					})
				})
				.resource("/chrono", |r| {
					r.method(Method::GET).f(handler::chrono::get_chrono);
					r.method(Method::PUT).f(handler::chrono::update_chrono)
				})
				.nested("/msg", |scope| {
					scope
						.resource("", |r| {
							r.method(Method::GET).f(handler::msg::fetch_msgs);
						})
						.resource("/done", |r| {
							r.method(Method::POST).f(handler::msg::done_msg);
						})
				})
		})
		.handler(
			"/static",
			fs::StaticFiles::with_config("./static-dist", NoCacheConfig).unwrap(),
		)
		.default_resource(|r| r.method(Method::GET).f(get_index))
}

fn main() {
	env_logger::init();
	// Init database
    /*
	let bootstrapper = System::new("setup");
	let bootstrap = storage::setup();
	let handle = bootstrap.then(|_| {
		System::current().stop();
		futures::future::ok(())
	});
	Arbiter::spawn(handle);
	bootstrapper.run();
    */

	info!("Starting server...");
	let server = server::new(build_app).bind("10.11.8.1:80").unwrap();
	server.run();
}

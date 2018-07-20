extern crate actix_web;

use actix_web::{fs, fs::NamedFile, http::Method, server, App, HttpRequest, Result};

fn get_index(_: HttpRequest) -> Result<NamedFile> {
	Ok(NamedFile::open("./static-dist/index.html")?)
}

fn build_app() -> App {
	App::new()
		.resource("/", |r| r.method(Method::GET).f(get_index))
		.handler(
			"/static",
			fs::StaticFiles::new("./static-dist").show_files_listing(),
		)
}

fn main() {
	server::new(build_app).bind("127.0.0.1:8088").unwrap().run();
}

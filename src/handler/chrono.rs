use actix_web;
use actix_web::{AsyncResponder, HttpMessage, HttpResponse};
use data::*;
use futures::Future;
use handler::AsyncResponse;
use handler::Request;

pub fn update_chrono(req: &Request) -> AsyncResponse {
	// TODO: clone state instead, as it only needs an Arc
	let _state = req.state().clone();

	req
		.state()
		.storage
		.get_chrono_spec()
		.map_err(|e| {
			let error: actix_web::Error = e.into();
			error
		})
		.join(req.json().map_err(|e| {
			let error: actix_web::Error = e.into();
			error
		}))
		.and_then(move |(spec, payload): (_, ChronoUpdate)| {
			let spec = spec.derive(payload.ratio);
			_state.storage.set_chrono_spec(spec).map_err(|e| e.into())
		})
		.map(|_| HttpResponse::Ok().body(r#"{"ok":true}"#))
		.responder()
}

pub fn get_chrono(req: &Request) -> AsyncResponse {
	req
		.state()
		.storage
		.get_chrono_spec()
		.map_err(|e| e.into())
		.map(|p| HttpResponse::Ok().json(p))
		.responder()
}

use actix_web;
use actix_web::{AsyncResponder, HttpMessage, HttpResponse};
use data::*;
use futures::Future;
use handler::AsyncResponse;
use handler::Request;

pub fn fetch_msgs(req: &Request) -> AsyncResponse {
	let id = "root";
	let _state = req.state().clone();

	req
		.state()
		.storage
		.get_groups(id)
		.and_then(move |ids| {
			let mut rcpts = vec![Rcpt::User(id.to_owned())];
			rcpts.extend(ids.into_iter().map(Rcpt::Group));
			_state.storage.fetch_messages(&rcpts)
		})
		.map(|p| HttpResponse::Ok().json(p))
		.map_err(|e| e.into())
		.responder()
}

pub fn done_msg(req: &Request) -> AsyncResponse {
	let state = req.state().clone();

	req.json()
	.map_err(|e| {
		let error: actix_web::Error = e.into();
		error
	})
	.and_then(move |payload: Message| {
		state.storage.done_msg(payload).map_err(|e| e.into())
	})
	.map(|_| HttpResponse::Ok().body(r#"{"ok":true}"#))
	.responder()
}

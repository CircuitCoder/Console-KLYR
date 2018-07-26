use actix_web;
use actix_web::error;
use actix_web::AsyncResponder;
use actix_web::FromRequest;
use actix_web::HttpMessage;
use actix_web::HttpResponse;
use actix_web::Json;
use actix_web::Path;
use actix_web::State;
use data::*;
use futures::future::Either;
use futures::{future, Future};
use handler::{AsyncResponse, Request};
use storage::StorageError;
use util;

pub fn list_posts(req: &Request) -> AsyncResponse {
	let tag = req.query().get("tag").cloned();

	req.state()
		.storage
		.filter_posts(tag)
		.map_err(|e| e.into())
		.map(|p| HttpResponse::Ok().json(p))
		.responder()
}

pub fn new_post(req: &Request) -> AsyncResponse {
	let _state = req.state().clone();
	let _state2 = req.state().clone();

	req.json()
		.from_err()
		.join(req.state().storage.get_chrono_spec().map_err(|e| {
			let error: actix_web::Error = e.into();
			error
		}))
		.and_then(move |(mut payload, chrono): (Post, _)| {
			if payload.has_id() {
				return future::Either::A(future::err(error::ErrorBadRequest(
					"No ID field should be present",
				)));
			}

			// Overwrite time field
			payload.time = chrono.now();

			future::Either::B(
				_state
					.storage
					.fetch_next_post_id()
					.and_then(move |id| {
						payload.set_id(id);
						_state
							.storage
							.put_post(&payload)
							.map(move |_| (id, payload.time))
					})
					.map_err(|e| e.into()),
			)
		})
		.and_then(move |(id, time)| {
			let msg = Message::new(
				MessageContent::WaitingReview { id },
				time,
				Rcpt::Group("reviewers".to_owned()),
			);
			_state2.storage.send_msg(msg).map_err(|e| e.into())
		})
		.map(|_| HttpResponse::Ok().body(r#"{"ok":true}"#))
		.responder()
}

pub fn update_post(
	(path, mut payload, state): (Path<(i64,)>, Json<Post>, State<::handler::State>),
) -> AsyncResponse {
	// TODO: authenticate

	if payload.has_id() {
		return future::err(error::ErrorBadRequest("No ID field should be present")).responder();
	}

	payload.set_id(path.0);

	state
		.storage
		.get_chrono_spec()
		.and_then(move |chrono| {
			payload.time = chrono.now();
			state.storage.put_post(&payload)
		})
		.map(|_| HttpResponse::Ok().body(r#"{"ok":true}"#))
		.map_err(|e| e.into())
		.responder()
}

pub fn accept_post(req: &Request) -> AsyncResponse {
	let _state = req.state().clone();
	let _state2 = _state.clone();
	let _state3 = _state.clone();

	let id = Path::<(i64,)>::extract(req);
	let id = match id {
		Ok(id) => id,
		Err(_) => return future::err(error::ErrorNotFound("Invalid id field")).responder(),
	};

	req.state()
		.storage
		.get_chrono_spec()
		.and_then(move |spec| _state.storage.accept_post(id.0, spec.now()))
		.and_then(move |post| _state2.storage.apply_index(&post).map(move |_| post))
		.join(_state3.storage.get_chrono_spec())
		.and_then(move |(post, chrono)| {
			let msg = Message::new(
				MessageContent::ReviewPassed {
					id: post.id.unwrap(),
				},
				chrono.now(),
				Rcpt::User(post.author),
			);
			_state3.storage.send_msg(msg)
		})
		.map(|_| HttpResponse::Ok().body(r#"{"ok":true}"#))
		.map_err(|e| e.into())
		.responder()
}

pub fn delete_post(req: &Request) -> AsyncResponse {
	let _state = req.state().clone();
	let _state2 = req.state().clone();

	let id = Path::<(i64,)>::extract(req);
	let id = match id {
		Ok(id) => id,
		Err(_) => return future::err(error::ErrorNotFound("Invalid id field")).responder(),
	};

	req.state()
		.storage
		.fetch_posts(vec![id.0.to_string()])
		.and_then(move |posts| {
			if posts.len() != 1 {
				return Either::A(future::err(StorageError::Racing));
			}

			Either::B(_state.storage.remove_index(&posts[0]))
		})
		.and_then(move |_| _state2.storage.delete_post(id.0))
		.map(|_| HttpResponse::Ok().body(r#"{"ok":true}"#))
		.map_err(|e| e.into())
		.responder()
}

fn digest_amplifier<T>(fut: T) -> AsyncResponse
where
	T: Future<Item = Vec<Post>, Error = StorageError> + 'static,
{
	fut.map_err(|e| e.into())
		.and_then(|mut posts| {
			if posts.len() != 1 {
				return future::err(error::ErrorNotFound("Not Found"));
			};

			let mut post = posts.pop().unwrap();
			post.content = util::digest_markdown(&post.content, 40);
			future::ok(HttpResponse::Ok().json(post))
		})
		.responder()
}

pub fn digest_post(req: &Request) -> AsyncResponse {
	let pending = req.query().get("pending").is_some();
	let id = Path::<(i64,)>::extract(req);

	let id = match id {
		Ok(v) => v,
		Err(_) => return future::err(error::ErrorNotFound("Not Found")).responder(),
	};

	if pending {
		digest_amplifier(
			req.state()
				.storage
				.fetch_pending_posts(vec![id.0.to_string()]),
		)
	} else {
		digest_amplifier(req.state().storage.fetch_posts(vec![id.0.to_string()]))
	}
}

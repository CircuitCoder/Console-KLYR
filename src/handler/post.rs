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
							.put_post(&payload, true)
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
			let time = chrono.now();
			payload.time = time;
			state.storage.put_post(&payload, false).map(move |_| (state, time))
		})
		.and_then(move |(state, time)| {
			let msg = Message::new(
				MessageContent::WaitingReview{ id: path.0 },
				time,
				Rcpt::Group("reviewers".to_owned())
			);

			state.storage.send_msg(msg)
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

pub fn reject_post(
	(id, state, payload): (Path<(i64,)>, State<::handler::State>, Json<PostRejection>),
) -> AsyncResponse {
	state.storage.fetch_pending_posts(vec![id.0.to_string()])
	.join(state.storage.get_chrono_spec())
	.map_err(|e| e.into())
	.and_then(move |(mut e, chrono)| {
		if e.len() != 1 {
			return Either::A(future::err(error::ErrorNotFound("Not Found")));
		}
		let post = e.pop().unwrap();
		let msg = Message::new(
			payload.0.into_msg(id.0),
			chrono.now(),
			Rcpt::User(post.author),
		);
		Either::B(state.storage.send_msg(msg).map_err(|e| e.into()))
	})
	.map(|_| HttpResponse::Ok().body(r#"{"ok":true}"#))
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
	T: Future<Item = (Vec<Post>, usize), Error = StorageError> + 'static,
{
	fut.map_err(|e| e.into())
		.and_then(|(mut posts, maxlen)| {
			if posts.len() != 1 {
				return future::err(error::ErrorNotFound("Not Found"));
			};

			let mut post = posts.pop().unwrap();
			post.content = util::digest_markdown(&post.content, maxlen);
			future::ok(HttpResponse::Ok().json(post))
		})
		.responder()
}

pub fn digest_post(req: &Request) -> AsyncResponse {
	let pending = req.query().get("pending").is_some();
	let mut maxlen = req.query().get("pending")
	  .and_then(|ml| ml.parse::<usize>().ok()).unwrap_or(40);
	
	if maxlen > 100 { maxlen = 100 };
	let id = Path::<(i64,)>::extract(req);

	let id = match id {
		Ok(v) => v,
		Err(_) => return future::err(error::ErrorNotFound("Not Found")).responder(),
	};

	if pending {
		digest_amplifier(
			req.state()
				.storage
				.fetch_pending_posts(vec![id.0.to_string()])
				.map(move |e| (e, maxlen))
		)
	} else {
		digest_amplifier(req.state().storage.fetch_posts(vec![id.0.to_string()])
				.map(move |e| (e, maxlen))
		)
	}
}

pub fn fetch_post(req: &Request) -> AsyncResponse {
	let pending = req.query().get("pending").is_some();
	let id = Path::<(i64,)>::extract(req);

	let id = match id {
		Ok(v) => v,
		Err(_) => return future::err(error::ErrorNotFound("Not Found")).responder(),
	};

	if pending {
		req.state()
			.storage
			.fetch_pending_posts(vec![id.0.to_string()])
			.map_err(|e| e.into())
			.and_then(|mut e| {
				if e.len() != 1 {
					return future::err(error::ErrorNotFound("Not Found"));
				}
				future::ok(HttpResponse::Ok().json(e.pop().unwrap()))
			})
			.responder()
	} else {
		req.state()
			.storage
			.fetch_posts(vec![id.0.to_string()])
			.map_err(|e| e.into())
			.and_then(|mut e| {
				if e.len() != 1 {
					return future::err(error::ErrorNotFound("Not Found"));
				}
				future::ok(HttpResponse::Ok().json(e.pop().unwrap()))
			})
			.responder()
	}
}

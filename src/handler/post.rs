use actix_web::error;
use actix_web::AsyncResponder;
use actix_web::HttpMessage;
use actix_web::HttpResponse;
use actix_web::Path;
use actix_web::Json;
use actix_web::State;
use data::*;
use futures::{future, Future};
use handler::{AsyncResponse, Request};
use actix_web::FromRequest;
use storage::StorageError;
use futures::future::Either;

pub fn list_posts(req: &Request) -> AsyncResponse {
	let _req = req.clone();
	let tag = req.query().get("tag").cloned();

	let filter_fut = req.state().storage.filter_posts(tag);
	let post_fut = filter_fut.and_then(move |ids| _req.state().storage.fetch_posts(ids));

	post_fut
		.map_err(|e| e.into())
		.map(|p| HttpResponse::Ok().json(p))
		.responder()
}

pub fn new_post(req: &Request) -> AsyncResponse {
	let _req = req.clone();

	req
		.json()
		.from_err()
		.and_then(move |mut payload: Post| {
			if payload.has_id() {
				return future::Either::A(future::err(error::ErrorBadRequest(
					"No ID field should be present",
				)));
			}

			future::Either::B(
				_req
					.state()
					.storage
					.fetch_next_post_id()
					.and_then(move |id| {
						payload.set_id(id);
						_req.state().storage.put_post(&payload)
					})
					.map(|_| HttpResponse::Ok().body(r#"{"ok":true}"#))
					.map_err(|e| e.into()),
			)
		})
		.responder()
}

pub fn update_post(
	(path, mut payload, state): (Path<(i64,)>, Json<Post>, State<::handler::State>),
) -> AsyncResponse {
	// TODO: verify that it is not published yet

	if payload.has_id() {
		return future::err(error::ErrorBadRequest(
			"No ID field should be present",
		)).responder();
	}

	payload.set_id(path.0);

	// TODO: authenticate

	state
		.storage
		.put_post(&payload)
		.map(|_| HttpResponse::Ok().body(r#"{"ok":true}"#))
		.map_err(|e| e.into())
		.responder()
}

pub fn accept_post(req: &Request) -> AsyncResponse {
	let _req = req.clone();
	let _req2 = req.clone();

	let id = Path::<(i64,)>::extract(req);
	let id = match id {
		Ok(id) => id,
		Err(_) => return future::err(error::ErrorNotFound("Invalid id field")).responder(),
	};

	req.state()
	.storage
	.accept_post(id.0)
	.and_then(move |_| {
		_req.state().storage.fetch_posts(vec![id.0.to_string()])
	})
	.and_then(move |posts| {
		if posts.len() != 1 {
			return Either::A(future::err(StorageError::Racing));
		}

		Either::B(_req2.state().storage.apply_index(&posts[0]))
	})
	.map(|_| HttpResponse::Ok().body(r#"{"ok":true}"#))
	.map_err(|e| e.into())
	.responder()
}

pub fn delete_post(req: &Request) -> AsyncResponse {
	let _req = req.clone();
	let _req2 = req.clone();

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

		Either::B(_req.state().storage.remove_index(&posts[0]))
	})
	.and_then(move |_| {
		_req2.state().storage.delete_post(id.0)
	})
	.map(|_| HttpResponse::Ok().body(r#"{"ok":true}"#))
	.map_err(|e| e.into())
	.responder()
}
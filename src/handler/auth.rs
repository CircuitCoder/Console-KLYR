use actix_web;
use actix_web::{AsyncResponder, HttpMessage, HttpResponse, error};
use actix_web::middleware::session::RequestSession;
use data::*;
use futures::Future;
use futures::future;
use handler::AsyncResponse;
use handler::Request;
use storage::StorageError;

pub fn fetch(req: &Request) -> AsyncResponse {
	let uid = req.session().get::<String>("uid");
	if let Ok(Some(uid)) = uid {
		req.state()
		.storage.get_groups(&uid)
		.join(req.state().storage.get_user_name(&uid))
		.map_err(|e| e.into())
		.and_then(|(groups, name)| {
			if let Some(name) = name {
				future::ok(HttpResponse::Ok().json(UserSpec{ id: uid, groups, name }))
			} else {
				future::err(StorageError::DivergedState.into())
			}
		})
		.responder()
	} else {
		future::err(error::ErrorForbidden("Haven't logged in")).responder()
	}
}

pub fn login(req: &Request) -> AsyncResponse {
	// TODO: use ring
	
	let state = req.state().clone();
	let session = req.session();
	req.json()
	.map_err(|e| {
		let e: actix_web::Error = e.into();
		e
	})
	.and_then(move |r: AuthReq| {
		state.storage.get_user_pass_hash(&r.username).map_err(|e| e.into()).map(move |v| (v, r))
	})
	.and_then(move |(v, r)| {
		if v == Some(r.password.into_bytes()) {
			session.set("uid", r.username);
			future::ok(HttpResponse::Ok().body(r#"{"ok":true}"#))
		} else {
			future::ok(HttpResponse::Ok().body(r#"{"ok":false}"#))
		}
	})
	.from_err()
	.responder()
}

pub fn logout(req: &Request) -> AsyncResponse {
	req.session().remove("uid");
	return future::ok(HttpResponse::Ok().body(r#"{"ok":true}"#)).responder();
}
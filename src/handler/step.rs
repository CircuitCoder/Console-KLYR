use actix_web;
use actix_web::{error, AsyncResponder, FromRequest, HttpMessage, HttpResponse, Path};
use data::*;
use futures::future;
use futures::Future;
use handler::AsyncResponse;
use handler::Request;

pub fn fetch_staged(req: &Request) -> AsyncResponse {
	let _state = req.state().clone();

	req.state()
		.storage
		.fetch_staged_steps()
		.and_then(move |ids| _state.storage.fetch_steps(ids))
		.map(|p| HttpResponse::Ok().json(p))
		.map_err(|e| e.into())
		.responder()
}

pub fn resolve(req: &Request) -> AsyncResponse {
	let id = Path::<(i64,)>::extract(req);
	let id = match id {
		Ok(id) => id,
		Err(_) => return future::err(error::ErrorNotFound("Not Found")).responder(),
	};

	return req
		.state()
		.storage
		.resolve_step(id.0)
		.map_err(|e| e.into())
		.map(|_| HttpResponse::Ok().body(r#"{"ok":true}"#))
		.responder();
}

pub fn assign(req: &Request) -> AsyncResponse {
	let id = Path::<(i64,)>::extract(req);
	let id = match id {
		Ok(id) => id,
		Err(_) => return future::err(error::ErrorNotFound("Not Found")).responder(),
	};

	let id = id.0;

	let state = req.state().clone();
	let state2 = state.clone();

	req
		.json()
		.map_err(|e| {
			let e: actix_web::Error = e.into();
			e
		})
		.and_then(move |v: Vec<String>| {
			state
				.storage
				.assign_step(id, v.clone())
				.map_err(|e| e.into())
				.map(move |_| v)
		})
		.join(state2.storage.get_chrono_spec().map_err(|e| e.into()))
		.and_then(move |(v, c)| {
			state2.storage.send_msgs(
				v.into_iter()
					.map(|s| {
						Message::new(MessageContent::StepAssigned { id }, c.now(), Rcpt::User(s))
					})
					.collect(),
			)
			.map_err(|e| e.into())
		})
		.map(|_| HttpResponse::Ok().body(r#"{"ok":true}"#))
		.responder()
}

pub fn create(req: &Request) -> AsyncResponse {
	let state = req.state().clone();
	let state2 = req.state().clone();

	req.json()
	.map_err(|e| {
		let e: actix_web::Error = e.into();
		e
	})
	.join(req.state().storage.fetch_next_step_id().map_err(|e| e.into()))
	.join(req.state().storage.get_chrono_spec().map_err(|e| e.into()))
	.and_then(move |((mut payload, id), chrono): ((Step, _), _)| {
		payload.id = Some(id);
		payload.time = chrono.now();
		state.storage.put_step(&payload).map_err(|e| e.into()).map(move |_| (state, id, chrono))
	})
	.and_then(move |(state, id, chrono)| {
		state.storage.stage_step(id).map_err(|e| e.into()).map(move |_| (id, chrono))
	})
	.and_then(move |(id, chrono)| {
		let msg = Message::new(
			MessageContent::StepCreated{ id },
			chrono.now(),
			Rcpt::Group("coordinators".to_owned())
		);
		state2.storage.send_msg(msg).map_err(|e| e.into()).map(move |_| id)
	})
	.map(|id| HttpResponse::Ok().body(format!(r#"{{"ok":true,"id":{}}}"#, id)))
	.responder()
}

pub fn detail(req: &Request) -> AsyncResponse {
	let id = Path::<(i64,)>::extract(req);
	let id = match id {
		Ok(id) => id,
		Err(_) => return future::err(error::ErrorNotFound("Not Found")).responder(),
	};

	let id = id.0;
	let state = req.state().clone();

	state.storage.fetch_steps(vec![id.to_string()])
	.map_err(|e| e.into())
	.and_then(|mut e| {
		if e.len() != 1 {
			return future::err(error::ErrorNotFound("Not Found"));
		}
		future::ok(e.pop().unwrap())
	})
	.join(state.storage.get_step_assignees(id).map_err(|e| e.into()))
	.join(state.storage.is_step_staged(id).map_err(|e| e.into()))
	.map(|((step, assignees), staged)| {
		HttpResponse::Ok().json(StepDetail{ step, assignees, staged })
	})
	.responder()
}
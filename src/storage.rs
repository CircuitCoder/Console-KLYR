use actix::Addr;
use actix::MailboxError;
use actix_redis;
use actix_redis::{Command, RedisActor, RespValue};
use data::*;
use futures::future;
use futures::future::Either;
use futures::prelude::*;
use serde_json;
use std::fmt;

#[derive(Debug)]
pub enum StorageError {
	Mailbox(MailboxError),
	Redis(actix_redis::Error),
	Format,
	InvalidArgument,
	DivergedState,
	Racing,
}

impl fmt::Display for StorageError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use storage::StorageError::*;
		match *self {
			Mailbox(_) => write!(f, "StorageError: Mailbox"),
			Redis(_) => write!(f, "StorageError: Redis"),
			Format => write!(f, "StorageError: Format"),
			InvalidArgument => write!(f, "StorageError: InvalidArgument"),
			DivergedState => write!(f, "StorageError: DivergedState"),
			Racing => write!(f, "StorageError: Racing"),
		}
	}
}

impl From<MailboxError> for StorageError {
	fn from(m: MailboxError) -> StorageError {
		StorageError::Mailbox(m)
	}
}

impl From<actix_redis::Error> for StorageError {
	fn from(m: actix_redis::Error) -> StorageError {
		StorageError::Redis(m)
	}
}

fn non_err(
	input: Result<RespValue, actix_redis::Error>,
) -> impl Future<Item = (), Error = StorageError> {
	match input {
		Ok(v) => match v {
			RespValue::Error(_) => future::err(StorageError::Format),
			_ => future::ok(()),
		},
		Err(e) => future::err(StorageError::Redis(e)),
	}
}

#[derive(Clone)]
pub struct Storage {
	db: Addr<RedisActor>,
}

impl Storage {
	pub fn new(db: Addr<RedisActor>) -> Storage {
		Storage { db }
	}

	pub fn setup(&self) -> impl Future<Item = (), Error = StorageError> {
		self.put_post(&Post::demo())
		.join(self.set_chrono_spec(Default::default()))
		.map(|_| ())
	}

	/* Posts */

	pub fn filter_posts(
		&self,
		tag: Option<String>,
	) -> impl Future<Item = Vec<String>, Error = StorageError> {
		let key = String::from("post:tag:") + &tag.unwrap_or_else(|| String::from(""));
		self
			.db
			.send(Command(RespValue::Array(vec![
				"ZREVRANGE".into(),
				key.into(),
				0.to_string().into(),
				(-1).to_string().into(),
			])))
			.from_err()
			.and_then(|f| {
				let value = match f {
					Ok(v) => v,
					Err(e) => return future::err(e.into()),
				};

				let array = match value {
					RespValue::Array(a) => a,
					RespValue::Nil => vec![],
					_ => return future::err(StorageError::Format),
				};

				let posts = array
					.into_iter()
					.filter_map(|e| match e {
						RespValue::BulkString(cont) => Some(String::from(String::from_utf8_lossy(&cont))),
						RespValue::SimpleString(cont) => Some(cont),
						_ => None,
					})
					.collect();

				future::ok(posts)
			})
	}

	pub fn fetch_posts(
		&self,
		posts: Vec<String>,
	) -> impl Future<Item = Vec<Post>, Error = StorageError> {
		// Currently we ignore malformat contents

		if posts.is_empty() {
			return future::Either::A(future::ok(vec![]));
		}

		let mut cmd = vec!["MGET".into()];

		let keys = posts
			.into_iter()
			.map(|e| String::from("post:content:") + &e)
			.map(String::into_bytes)
			.map(RespValue::BulkString);

		cmd.extend(keys);

		let unwrapped = self
			.db
			.send(Command(RespValue::Array(cmd)))
			.from_err()
			.and_then(|resp| {
				debug!("Response from DB: {:?}", resp);
				let values = if let Ok(v) = resp {
					v
				} else {
					return future::err(StorageError::Format);
				};

				let array = if let RespValue::Array(a) = values {
					a
				} else if let RespValue::Nil = values {
					vec![]
				} else {
					return future::err(StorageError::Format);
				};

				let result = array
					.into_iter()
					.filter_map(|e| match e {
						RespValue::SimpleString(s) => serde_json::from_str(&s).ok(),
						RespValue::BulkString(s) => serde_json::from_slice(&s).ok(),
						_ => None,
					})
					.collect();

				future::ok(result)
			});

		future::Either::B(unwrapped)
	}

	pub fn put_post(&self, p: &Post) -> impl Future<Item = (), Error = StorageError> {
		// This is for update

		let id = match p.id {
			Some(id) => format!("post:pending:{}", id),
			None => return Either::A(future::err(StorageError::InvalidArgument)),
		};

		let formatted = match serde_json::to_vec(p) {
			Ok(f) => f,
			Err(_) => return Either::A(future::err(StorageError::Format)),
		};

		Either::B(
			self
				.db
				.send(Command(RespValue::Array(vec![
					"SET".into(),
					id.into(),
					formatted.into(),
				])))
				.from_err()
				.map(|_| ()),
		)
	}

	pub fn accept_post(&self, id: i64) -> impl Future<Item = (), Error = StorageError> {
		let original = format!("post:pending:{}", id);
		let target = format!("post:content:{}", id);

		self
			.db
			.send(Command(RespValue::Array(vec![
				"RENAMENX".into(),
				original.into_bytes().into(),
				target.clone().into_bytes().into(),
			])))
			.from_err()
			.and_then(|r| {
				let inner = match r {
					Ok(v) => v,
					_ => return future::err(StorageError::DivergedState),
				};

				match inner {
					RespValue::Error(_) => future::err(StorageError::DivergedState),
					_ => future::ok(()),
				}
			})
	}

	pub fn apply_index(&self, p: &Post) -> impl Future<Item = (), Error = StorageError> {
		let id = match p.id {
			Some(id) => id,
			None => return Either::A(future::err(StorageError::InvalidArgument)),
		};
		let mut command = format!("redis.call('ZADD', 'post:tag:', '{}', {})\n", id, id);
		for e in &p.tags {
			command += &format!("redis.call('ZADD', 'post:tag:{}', '{}', {})\n", e, id, id);
		}
		Either::B(
			self
				.db
				.send(Command(RespValue::Array(vec![
					"EVAL".into(),
					command.into(),
					0.to_string().into_bytes().into(),
				])))
				.from_err()
				.and_then(|r| {
					let inner = match r {
						Ok(v) => v,
						_ => return future::err(StorageError::DivergedState),
					};

					match inner {
						RespValue::Error(_) => future::err(StorageError::DivergedState),
						_ => future::ok(()),
					}
				}),
		)
	}

	pub fn delete_post(&self, id: i64) -> impl Future<Item = (), Error = StorageError> {
		// TODO: merge with accept_post
		let original = format!("post:content:{}", id);
		let target = format!("post:deleted:{}", id);

		self
			.db
			.send(Command(RespValue::Array(vec![
				"RENAMENX".into(),
				original.into_bytes().into(),
				target.clone().into_bytes().into(),
			])))
			.from_err()
			.and_then(|r| {
				let inner = match r {
					Ok(v) => v,
					_ => return future::err(StorageError::DivergedState),
				};

				match inner {
					RespValue::Error(_) => future::err(StorageError::DivergedState),
					_ => future::ok(()),
				}
			})
	}

	pub fn remove_index(&self, p: &Post) -> impl Future<Item = (), Error = StorageError> {
		// TODO: merge with apply_index
		let id = match p.id {
			Some(id) => id,
			None => return Either::A(future::err(StorageError::InvalidArgument)),
		};
		let mut command = format!("redis.call('ZREM', 'post:tag:', '{}')\n", id);
		for e in &p.tags {
			command += &format!("redis.call('ZREM', 'post:tag:{}', '{}')\n", e, id);
		}
		Either::B(
			self
				.db
				.send(Command(RespValue::Array(vec![
					"EVAL".into(),
					command.into(),
					0.to_string().into_bytes().into(),
				])))
				.from_err()
				.and_then(|r| {
					let inner = match r {
						Ok(v) => v,
						_ => return future::err(StorageError::DivergedState),
					};

					match inner {
						RespValue::Error(_) => future::err(StorageError::DivergedState),
						_ => future::ok(()),
					}
				}),
		)
	}

	pub fn fetch_next_post_id(&self) -> impl Future<Item = i64, Error = StorageError> {
		self
			.db
			.send(Command(RespValue::Array(vec![
				"INCR".into(),
				"post:counter".to_owned().into_bytes().into(),
			])))
			.from_err()
			.and_then(|counter| {
				let counter = match counter {
					Ok(v) => v,
					Err(_) => return future::err(StorageError::Format),
				};

				match counter {
					RespValue::Integer(i) => future::ok(i),
					_ => future::err(StorageError::Format),
				}
			})
	}

	/* Chronometer */
	pub fn get_chrono_spec(&self) -> impl Future<Item = ChronoSpec, Error = StorageError> {
		self
			.db
			.send(Command(RespValue::Array(vec![
				"GET".into(),
				"chrono:spec".to_owned().into_bytes().into(),
			])))
			.from_err()
			.and_then(|v| {
				let v = match v {
					Ok(v) => v,
					Err(e) => return future::err(StorageError::Redis(e)),
				};

				let v = match v {
					RespValue::Nil => return future::err(StorageError::DivergedState),
					RespValue::SimpleString(s) => serde_json::from_str(&s),
					RespValue::BulkString(s) => serde_json::from_slice(&s),
					_ => return future::err(StorageError::Format),
				};

				match v {
					Ok(v) => future::ok(v),
					Err(_) => future::err(StorageError::Format),
				}
			})
	}

	pub fn set_chrono_spec(&self, spec: ChronoSpec) -> impl Future<Item = (), Error = StorageError> {
		let converted = match serde_json::to_vec(&spec) {
			Ok(v) => v,
			Err(_) => return future::Either::A(future::err(StorageError::Format)),
		};

		future::Either::B(
			self
				.db
				.send(Command(RespValue::Array(vec![
					"SET".into(),
					"chrono:spec".to_owned().into_bytes().into(),
					converted.into(),
				])))
				.from_err()
				.and_then(non_err),
		)
	}
}

impl Default for Storage {
	fn default() -> Storage {
		let db = RedisActor::start("127.0.0.1:6379");
		Storage::new(db)
	}
}

pub fn setup() -> impl Future<Item = (), Error = StorageError> {
	let s: Storage = Default::default();
	s.setup()
}

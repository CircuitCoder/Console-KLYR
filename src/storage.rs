use actix::Addr;
use actix::MailboxError;
use actix_redis;
use actix_redis::{Command, RedisActor, RespValue};
use data::*;
use futures::future;
use futures::future::Either;
use futures::prelude::*;
use serde::de::DeserializeOwned;
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
			RespValue::Error(e) => {
				debug!("Redis inner error: {}", e);
				future::err(StorageError::Format)
			}
			_ => future::ok(()),
		},
		Err(e) => future::err(StorageError::Redis(e)),
	}
}

fn parse_arr<T>(
	v: Result<RespValue, actix_redis::Error>,
) -> impl Future<Item = Vec<T>, Error = StorageError>
where
	T: DeserializeOwned,
{
	let v = match v {
		Ok(v) => v,
		Err(e) => return future::err(StorageError::Redis(e)),
	};

	let v = match v {
		RespValue::Array(a) => a,
		_ => return future::err(StorageError::Format),
	};

	let r = v
		.into_iter()
		.filter_map(|v| match v {
			RespValue::SimpleString(s) => Some(serde_json::from_str(&s)),
			RespValue::BulkString(s) => Some(serde_json::from_slice(&s)),
			_ => None,
		})
		.filter_map(|v| match v {
			Ok(v) => Some(v),
			Err(_) => None,
		})
		.collect();

	future::ok(r)
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
		self.db
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
						RespValue::BulkString(cont) => {
							Some(String::from(String::from_utf8_lossy(&cont)))
						}
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
		self.multi_fetch(posts, "post:content")
	}

	pub fn fetch_pending_posts(
		&self,
		posts: Vec<String>,
	) -> impl Future<Item = Vec<Post>, Error = StorageError> {
		self.multi_fetch(posts, "post:pending")
	}

	pub fn put_post(&self, p: &Post) -> impl Future<Item = (), Error = StorageError> {
		let id = match p.id {
			Some(id) => format!("post:pending:{}", id),
			None => return Either::A(future::err(StorageError::InvalidArgument)),
		};

		let formatted = match serde_json::to_vec(p) {
			Ok(f) => f,
			Err(_) => return Either::A(future::err(StorageError::Format)),
		};

		Either::B(
			self.db
				.send(Command(RespValue::Array(vec![
					"SET".into(),
					id.into(),
					formatted.into(),
				])))
				.from_err()
				.map(|_| ()),
		)
	}

	pub fn accept_post(
		&self,
		id: i64,
		time: u64,
	) -> impl Future<Item = Post, Error = StorageError> {
		let original = format!("post:pending:{}", id);
		let target = format!("post:content:{}", id);

		let atomic_get_del = format!(
			"s = redis.call('GET', '{}')\nredis.call('DEL', '{}')\nreturn s",
			original, original
		);
		let _self = self.clone();

		self.db
			.send(Command(RespValue::Array(vec![
				"EVAL".into(),
				atomic_get_del.into_bytes().into(),
				0.to_string().into_bytes().into(),
			])))
			.from_err()
			.and_then(move |r| {
				let inner = match r {
					Ok(v) => v,
					_ => return Either::A(future::err(StorageError::DivergedState)),
				};

				let content = match inner {
					RespValue::Error(_) => {
						return Either::A(future::err(StorageError::DivergedState))
					}
					RespValue::SimpleString(s) => serde_json::from_str(&s),
					RespValue::BulkString(s) => serde_json::from_slice(&s),
					_ => return Either::A(future::err(StorageError::Format)),
				};

				let mut content: Post = match content {
					Ok(c) => c,
					Err(_) => return Either::A(future::err(StorageError::Format)),
				};

				content.time = time;

				let serialized = match serde_json::to_vec(&content) {
					Ok(s) => s,
					Err(_) => return Either::A(future::err(StorageError::Format)),
				};

				Either::B(
					_self
						.db
						.send(Command(RespValue::Array(vec![
							"SETNX".into(),
							target.into_bytes().into(),
							serialized.into(),
						])))
						.from_err()
						.map(move |_| content),
				)
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
			self.db
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

		self.db
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
			self.db
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
		self.next_id("post")
	}

	/* Reactor */

	pub fn fetch_steps(
		&self,
		s: Vec<String>,
	) -> impl Future<Item = Vec<Step>, Error = StorageError> {
		self.multi_fetch(s, "step:content")
	}

	pub fn put_step(&self, s: &Step) -> impl Future<Item = (), Error = StorageError> {
		let id = match s.id {
			Some(id) => format!("step:content:{}", id),
			None => return Either::A(future::err(StorageError::InvalidArgument)),
		};

		let formatted = match serde_json::to_vec(s) {
			Ok(f) => f,
			Err(_) => return Either::A(future::err(StorageError::Format)),
		};

		Either::B(
			self.db
				.send(Command(RespValue::Array(vec![
					"SET".into(),
					id.into(),
					formatted.into(),
				])))
				.from_err()
				.map(|_| ()),
		)
	}

	pub fn assign_step(
		&self,
		id: i64,
		assignees: Vec<String>,
	) -> impl Future<Item = (), Error = StorageError> {
		let id = format!("step:assignees:{}", id);
		let mut command = vec!["SADD".into(), id.into_bytes().into()];
		command.extend(assignees.into_iter().map(|a| a.into_bytes().into()));

		self.db
			.send(Command(RespValue::Array(command)))
			.map(|_| ())
			.from_err()
	}

	pub fn stage_step(&self, id: i64) -> impl Future<Item = (), Error = StorageError> {
		self.db
			.send(Command(RespValue::Array(vec![
				"SADD".into(),
				"step:stage".to_owned().into_bytes().into(),
				id.to_string().into_bytes().into(),
			])))
			.map(|_| ())
			.from_err()
	}

	pub fn resolve_step(&self, id: i64) -> impl Future<Item = (), Error = StorageError> {
		self.db
			.send(Command(RespValue::Array(vec![
				"SREM".into(),
				"step:stage".to_owned().into_bytes().into(),
				id.to_string().into_bytes().into(),
			])))
			.map(|_| ())
			.from_err()

		// TODO: bulk resolve
	}

	pub fn fetch_next_step_id(&self) -> impl Future<Item = i64, Error = StorageError> {
		self.next_id("step")
	}

	/* Chronometer */
	pub fn get_chrono_spec(&self) -> impl Future<Item = ChronoSpec, Error = StorageError> {
		self.db
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

	pub fn set_chrono_spec(
		&self,
		spec: ChronoSpec,
	) -> impl Future<Item = (), Error = StorageError> {
		let converted = match serde_json::to_vec(&spec) {
			Ok(v) => v,
			Err(_) => return future::Either::A(future::err(StorageError::Format)),
		};

		future::Either::B(
			self.db
				.send(Command(RespValue::Array(vec![
					"SET".into(),
					"chrono:spec".to_owned().into_bytes().into(),
					converted.into(),
				])))
				.from_err()
				.and_then(non_err),
		)
	}

	/* Users */
	pub fn get_groups(&self, id: String) -> impl Future<Item = Vec<String>, Error = StorageError> {
		self.db
			.send(Command(RespValue::Array(vec![
				"SMEMBERS".into(),
				format!("user:{}:groups", id).into_bytes().into(),
			])))
			.from_err()
			.and_then(|v| {
				let v = match v {
					Ok(v) => v,
					Err(e) => return future::err(StorageError::Redis(e)),
				};

				let v = match v {
					RespValue::Array(a) => a,
					_ => return future::err(StorageError::Format),
				};

				let r = v
					.into_iter()
					.filter_map(|v| match v {
						RespValue::SimpleString(s) => Some(s),
						RespValue::BulkString(s) => String::from_utf8(s).ok(),
						_ => None,
					})
					.collect();

				future::ok(r)
			})
	}

	/* Messaging */
	pub fn fetch_messages(
		&self,
		target: &Vec<Rcpt>,
	) -> impl Future<Item = Vec<Message>, Error = StorageError> {
		debug!("Rcpts: {:?}", target);
		let mut command = vec!["SUNION".into()];
		command.extend(target.iter().map(|v| v.mailbox().into_bytes().into()));
		self.db
			.send(Command(RespValue::Array(command)))
			.from_err()
			.and_then(parse_arr)
	}

	pub fn send_msg(&self, msg: Message) -> impl Future<Item = (), Error = StorageError> {
		let mailbox = msg.rcpt.mailbox();
		let serialized = match serde_json::to_vec(&msg) {
			Ok(s) => s,
			Err(_) => return Either::A(future::err(StorageError::Format)),
		};

		debug!("Insert msg: {} <- {:?}", mailbox, msg);

		Either::B(
			self.db
				.send(Command(RespValue::Array(vec![
					"SADD".into(),
					mailbox.into_bytes().into(),
					serialized.into(),
					// RespValue::Integer(msg.time as i64),
				])))
				.from_err()
				.and_then(non_err),
		)
	}

	pub fn done_msg(&self, msg: Message) -> impl Future<Item = (), Error = StorageError> {
		let mailbox = msg.rcpt.mailbox();
		let backlog = msg.rcpt.backlog();

		let serialized = match serde_json::to_vec(&msg) {
			Ok(s) => s,
			Err(_) => return Either::A(future::err(StorageError::Format)),
		};

		Either::B(
			self.db
				.send(Command(RespValue::Array(vec![
					"SMOVE".into(),
					mailbox.into_bytes().into(),
					backlog.into_bytes().into(),
					serialized.into(),
				])))
				.map(|_| ())
				.from_err(),
		)
	}

	/* Internal */
	pub fn next_id(&self, desc: &'static str) -> impl Future<Item = i64, Error = StorageError> {
		self.db
			.send(Command(RespValue::Array(vec![
				"INCR".into(),
				format!("{}:counter", desc).to_owned().into_bytes().into(),
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

	pub fn multi_fetch<T>(
		&self,
		ids: Vec<String>,
		prefix: &str,
	) -> impl Future<Item = Vec<T>, Error = StorageError>
	where
		T: DeserializeOwned,
	{
		// Currently we ignore malformat contents

		if ids.is_empty() {
			return future::Either::A(future::ok(vec![]));
		}

		let mut cmd = vec!["MGET".into()];

		let keys = ids
			.into_iter()
			.map(|e| format!("{}:{}", prefix, e))
			.map(String::into_bytes)
			.map(RespValue::BulkString);

		cmd.extend(keys);

		let unwrapped = self
			.db
			.send(Command(RespValue::Array(cmd)))
			.from_err()
			.and_then(|resp| {
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

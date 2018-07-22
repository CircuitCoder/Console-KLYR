use actix::Addr;
use actix::MailboxError;
use actix_redis;
use actix_redis::{Command, RedisActor, RespValue};
use data::*;
use futures::future;
use futures::prelude::*;
use serde_json;
use std::fmt;

#[derive(Debug)]
pub enum StorageError {
	Mailbox(MailboxError),
	Redis(actix_redis::Error),
	Format,
	InvalidArgument,
}

impl fmt::Display for StorageError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use storage::StorageError::*;
		match self {
			&Mailbox(_) => write!(f, "StorageError: Mailbox"),
			&Redis(_) => write!(f, "StorageError: Redis"),
			&Format => write!(f, "StorageError: Format"),
			&InvalidArgument => write!(f, "StorageError: InvalidArgument"),
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

pub struct Storage {
	db: Addr<RedisActor>,
}

impl Storage {
	pub fn new(db: Addr<RedisActor>) -> Storage {
		Storage { db }
	}

	pub fn setup(&self) -> impl Future<Item = (), Error = StorageError> {
		let demo = Post::demo();
		info!("Inserting demo post...");
		let fut = self.put_post(&demo);
		fut
	}

	pub fn filter_posts(
		&self,
		tag: Option<String>,
	) -> impl Future<Item = Vec<String>, Error = StorageError> {
		let key = String::from("post:tag:") + &tag.unwrap_or_else(|| String::from(""));
		self
			.db
			.send(Command(RespValue::Array(vec![
				"LRANGE".into(),
				key.into(),
				0.to_string().into(),
				(-1).to_string().into(),
			])))
			.from_err()
			.and_then(|f| {
				debug!("Response from DB: {:?}", f);
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

		if posts.len() == 0 {
			return future::Either::A(future::ok(vec![]))
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

	pub fn put_post(&self, p: &Post) -> Box<dyn Future<Item = (), Error = StorageError>> {
		// This is for update

		let id = match p.db_id() {
			Some(id) => id,
			None => return Box::new(future::err(StorageError::InvalidArgument)),
		};

		let formatted = match serde_json::to_vec(p) {
			Ok(f) => f,
			Err(_) => return Box::new(future::err(StorageError::Format)),
		};
		
		Box::new(self.db.send(Command(RespValue::Array(vec![
			"SET".into(),
			id.into(),
			formatted.into(),
		])))
		.from_err()
		.map(|_| ()))
	}

	pub fn fetch_next_post_id(&self) -> impl Future<Item = i64, Error = StorageError> {
		self.db.send(Command(RespValue::Array(vec![
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
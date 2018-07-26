use rand::distributions::Alphanumeric;
use rand::{self, Rng};
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Clone)]
pub struct Post {
	pub(crate) id: Option<i64>,
	pub(crate) tags: Vec<String>,
	pub(crate) author: String,
	pub(crate) title: String,
	pub(crate) content: String,
	pub(crate) time: u64,
}

impl Post {
	pub fn demo() -> Post {
		Post {
			id: Some(1),
			tags: vec!["demo".to_string()],
			author: "root".to_string(),
			title: "Demo".to_string(),
			content: "This is a demo post".to_string(),
			time: SystemTime::now()
				.duration_since(SystemTime::UNIX_EPOCH)
				.unwrap()
				.as_secs(),
		}
	}

	/**
	 *  Will silently fail if the id is present
	 */
	pub fn set_id(&mut self, id: i64) {
		self.id = Some(id)
	}

	pub fn has_id(&self) -> bool {
		self.id.is_some()
	}
}

#[derive(Serialize, Deserialize)]
pub struct User {
	id: Option<String>,
	name: String,
}

#[derive(Serialize, Deserialize)]
pub struct Step {
	pub(crate) id: Option<i64>,
	pub(crate) parent: Option<i64>,

	title: String,
	content: String,
}

#[derive(Serialize, Deserialize)]
pub struct Comment {
	content: String,
	author: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageContent {
	WaitingReview { id: i64 },
	ReviewPassed { id: i64 },
	ReviewRejected { id: i64, comment: String },
}

// TODO: create message

#[derive(Serialize, Deserialize, Debug)]
pub enum Rcpt {
	Group(String),
	User(String),
}

impl Rcpt {
	pub fn mailbox(&self) -> String {
		use self::Rcpt::*;
		match *self {
			Group(ref g) => format!("mailbox:group:{}", g),
			User(ref u) => format!("mailbox:user:{}", u),
		}
	}

	pub fn backlog(&self) -> String {
		use self::Rcpt::*;
		match *self {
			Group(ref g) => format!("backlog:group:{}", g),
			User(ref u) => format!("backlog:user:{}", u),
		}
	}
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
	pub(crate) id: String, // A random string
	pub(crate) content: MessageContent,
	pub(crate) time: u64,
	pub(crate) rcpt: Rcpt,
	// TODO: do we need realtime?
}

impl Message {
	pub fn new(content: MessageContent, time: u64, rcpt: Rcpt) -> Message {
		let id = rand::thread_rng()
			.sample_iter(&Alphanumeric)
			.take(16)
			.collect::<String>();
		Message {
			id,
			content,
			time,
			rcpt,
		}
	}
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct ChronoSpec {
	anchor: u64,
	real: u64,
	ratio: f64,
}

impl ChronoSpec {
	fn calc_virt(&self, cur_real: u64) -> u64 {
		let diff = cur_real - self.real;
		self.anchor + (diff as f64 * self.ratio).floor() as u64
	}

	pub fn now(&self) -> u64 {
		// Rounded towards zero

		let cur_real = SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.unwrap()
			.as_secs();
		self.calc_virt(cur_real)
	}

	pub fn derive(&self, ratio: f64) -> ChronoSpec {
		let real = SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.unwrap()
			.as_secs();
		let anchor = self.calc_virt(real);

		ChronoSpec {
			real,
			anchor,
			ratio,
		}
	}
}

impl Default for ChronoSpec {
	fn default() -> ChronoSpec {
		ChronoSpec {
			real: 0, // Epoch
			anchor: 0,
			ratio: 1_f64,
		}
	}
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct ChronoUpdate {
	pub(crate) ratio: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PostRejection {
	pub(crate) comment: String,
}

impl PostRejection {
	pub fn into_msg(self, id: i64) -> MessageContent {
		let PostRejection{ comment } = self;
		MessageContent::ReviewRejected{
			id, comment,
		}
	}
}
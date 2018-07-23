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

	is_admin: bool,
	is_coordinator: bool,
	is_resolver: bool,
	is_author: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Step {
	id: Option<i64>,
	parent: Option<i64>,

	title: String,
	content: String,
	resolvers: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Comment {
	content: String,
	author: String,
}

#[derive(Serialize, Deserialize)]
pub enum Message {
	WaitingReview { id: u64 },
	ReviewPassed { id: u64 },
	ReviewRejected { id: u64 },
}

#[derive(Serialize, Deserialize)]
pub struct Chronometer {
	anchor: u64,
	real: u64,
	ratio: f64,
}
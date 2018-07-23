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
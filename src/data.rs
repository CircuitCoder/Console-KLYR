#[derive(Serialize, Deserialize)]
pub struct Post {
	id: Option<i64>,
	tags: Vec<String>,
	author: String,
	title: String,
	content: String,

	deleted: bool,
}

impl Post {
	pub fn demo() -> Post {
		Post {
			id: Some(1),
			tags: vec!["demo".to_string()],
			author: "root".to_string(),
			title: "Demo".to_string(),
			content: "This is a demo post".to_string(),

			deleted: false,
		}
	}

	pub fn db_id(&self) -> Option<String> {
		self.id.map(|id| format!("post:content:{}", id))
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

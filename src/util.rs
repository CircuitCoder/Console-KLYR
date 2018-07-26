use pulldown_cmark::{Parser, Event};

pub fn digest_markdown(s: &str, maxlen: usize) -> String {
	let parser = Parser::new(s);

	let mut result = String::new();
	let mut last_is_space = true;

	for ev in parser {
		match ev {
			Event::Text(t) => {
				result.push_str(&*t);
				last_is_space = false;
			},
			_ => {
				if !last_is_space {
					result.push(' ');
					last_is_space = true;
				}
			},
		};

		if result.len() >= maxlen {
			result.split_off(maxlen);
			return result;
		};
	};

	return result;
}
#[derive(Clone, PartialEq, Eq, Copy)]
pub enum Font {
	Large,
	Medium,
	Small,
	Smallest,
}

impl Font {
	pub fn smaller(&self) -> Self {
		match self {
			Font::Large => Font::Medium,
			Font::Medium => Font::Small,
			_ => Font::Smallest,
		}
	}

	pub fn larger(&self) -> Self {
		match self {
			Font::Smallest => Font::Small,
			Font::Small => Font::Medium,
			_ => Font::Large,
		}
	}

	pub fn is_smallest(&self) -> bool {
		self == &Font::Smallest
	}

	pub fn is_largest(&self) -> bool {
		self == &Font::Large
	}
}

pub trait FontMetrics {
	fn width(&self, font: Font, text: &str) -> i32;
	fn advance(&self, font: Font, text: &str) -> i32;
	fn height(&self, font: Font) -> i32;
}

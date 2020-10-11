use crate::font::Font;
use crate::layout::{Layout, TokenType};
use rscalc_math::number::ToNumber;
use rscalc_math::unit::CompositeUnit;

#[cfg(not(feature = "std"))]
use alloc::boxed::Box;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

pub trait CompositeUnitLayout {
	fn layout(&self, base_font: Font) -> Option<Layout>;
}

impl CompositeUnitLayout for CompositeUnit {
	fn layout(&self, base_font: Font) -> Option<Layout> {
		// Font sizes are different depending on if the units have a fraction
		// representation or not, so keep track of both
		let mut numer_layout = Vec::new();
		let mut numer_only_layout = Vec::new();
		let mut denom_layout = Vec::new();
		let mut denom_only_layout = Vec::new();
		// Sort units into numerator and denominator layout lists
		for (_, unit) in &self.units {
			if unit.1 < 0 {
				// Power is negative, unit is in denominator
				if denom_layout.len() != 0 {
					// Add multiplication symbol to separate unit names
					denom_layout.push(Layout::StaticText(
						"∙",
						base_font.smaller(),
						TokenType::Unit,
					));
					denom_only_layout.push(Layout::StaticText("∙", base_font, TokenType::Unit));
				}
				// Create layout in denomator of a fraction
				let unit_text =
					Layout::StaticText(unit.0.to_str(), base_font.smaller(), TokenType::Unit);
				let layout = if unit.1 < -1 {
					Layout::Power(
						Box::new(unit_text),
						Box::new(Layout::Text(
							(-unit.1).to_number().to_string(),
							Font::Smallest,
							TokenType::Unit,
						)),
					)
				} else {
					unit_text
				};
				denom_layout.push(layout);
				// Create layout if there is no numerator
				denom_only_layout.push(Layout::Power(
					Box::new(Layout::StaticText(
						unit.0.to_str(),
						base_font,
						TokenType::Unit,
					)),
					Box::new(Layout::Text(
						unit.1.to_number().to_string(),
						base_font.smaller(),
						TokenType::Unit,
					)),
				));
			} else if unit.1 > 0 {
				// Power is positive, unit is in numerator
				if numer_layout.len() != 0 {
					// Add multiplication symbol to separate unit names
					numer_layout.push(Layout::StaticText(
						"∙",
						base_font.smaller(),
						TokenType::Unit,
					));
					numer_only_layout.push(Layout::StaticText("∙", base_font, TokenType::Unit));
				}
				// Create layout in numerator of a fraction
				let unit_text =
					Layout::StaticText(unit.0.to_str(), base_font.smaller(), TokenType::Unit);
				let layout = if unit.1 > 1 {
					Layout::Power(
						Box::new(unit_text),
						Box::new(Layout::Text(
							unit.1.to_number().to_string(),
							Font::Smallest,
							TokenType::Unit,
						)),
					)
				} else {
					unit_text
				};
				numer_layout.push(layout);
				// Create layout if there is no denominator
				let unit_text = Layout::StaticText(unit.0.to_str(), base_font, TokenType::Unit);
				let layout = if unit.1 > 1 {
					Layout::Power(
						Box::new(unit_text),
						Box::new(Layout::Text(
							unit.1.to_number().to_string(),
							base_font.smaller().smaller(),
							TokenType::Unit,
						)),
					)
				} else {
					unit_text
				};
				numer_only_layout.push(layout);
			}
		}
		// Create final layout
		if numer_layout.len() == 0 && denom_layout.len() == 0 {
			// No unit
			None
		} else if denom_layout.len() == 0 {
			// Numerator only
			numer_only_layout.insert(0, Layout::StaticText(" ", base_font, TokenType::Unit));
			Some(Layout::Horizontal(numer_only_layout))
		} else if numer_layout.len() == 0 {
			// Denominator only
			denom_only_layout.insert(0, Layout::StaticText(" ", base_font, TokenType::Unit));
			Some(Layout::Horizontal(denom_only_layout))
		} else {
			// Fraction
			let mut final_layout = Vec::new();
			final_layout.push(Layout::StaticText(" ", base_font, TokenType::Unit));
			final_layout.push(Layout::Fraction(
				Box::new(Layout::Horizontal(numer_layout)),
				Box::new(Layout::Horizontal(denom_layout)),
				TokenType::Unit,
			));
			Some(Layout::Horizontal(final_layout))
		}
	}
}

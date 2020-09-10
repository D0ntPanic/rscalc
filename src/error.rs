use num_bigint::TryFromBigIntError;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Error {
	NotEnoughValues,
	NotANumber,
	InvalidInteger,
	DataTypeMismatch,
	IncompatibleUnits,
	InvalidEntry,
	InvalidStackIndex,
	ValueNotDefined,
	ValueOutOfRange,
	FloatRequiresDecimalMode,
	RequiresSizedIntegerMode,
	InvalidDate,
	InvalidTime,
}

impl Error {
	pub fn to_str(&self) -> &str {
		match self {
			Error::NotEnoughValues => "Not enough values",
			Error::NotANumber => "Not a number",
			Error::InvalidInteger => "Invalid integer",
			Error::DataTypeMismatch => "Data type mismatch",
			Error::IncompatibleUnits => "Incompatible units",
			Error::InvalidEntry => "Invalid entry",
			Error::InvalidStackIndex => "Invalid stack index",
			Error::ValueNotDefined => "Value not defined",
			Error::ValueOutOfRange => "Value out of range",
			Error::FloatRequiresDecimalMode => "Requires decimal mode",
			Error::RequiresSizedIntegerMode => "Requires sized int mode",
			Error::InvalidDate => "Invalid date",
			Error::InvalidTime => "Invalid time",
		}
	}
}

impl<T> From<TryFromBigIntError<T>> for Error {
	fn from(_: TryFromBigIntError<T>) -> Self {
		Error::ValueOutOfRange
	}
}

pub type Result<T> = core::result::Result<T, Error>;

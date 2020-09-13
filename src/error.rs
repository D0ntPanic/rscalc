use core::array::TryFromSliceError;
use num_bigint::TryFromBigIntError;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Error {
	OutOfMemory,
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
	CorruptData,
}

impl Error {
	pub fn to_str(&self) -> &str {
		match self {
			Error::OutOfMemory => "Out of memory",
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
			Error::CorruptData => "Corrupt data",
		}
	}
}

impl<T> From<TryFromBigIntError<T>> for Error {
	fn from(_: TryFromBigIntError<T>) -> Self {
		Error::ValueOutOfRange
	}
}

impl From<TryFromSliceError> for Error {
	fn from(_: TryFromSliceError) -> Self {
		Error::CorruptData
	}
}

pub type Result<T> = core::result::Result<T, Error>;

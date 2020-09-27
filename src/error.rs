use core::array::TryFromSliceError;
use num_bigint::TryFromBigIntError;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Error {
	OutOfMemory,
	NotEnoughValues,
	NotARealNumber,
	InvalidInteger,
	DataTypeMismatch,
	IncompatibleUnits,
	InvalidEntry,
	InvalidStackIndex,
	ValueNotDefined,
	ValueOutOfRange,
	IndexOutOfRange,
	FloatRequiresDecimalMode,
	RequiresSizedIntegerMode,
	InvalidDate,
	InvalidTime,
	CorruptData,
	StackOverflow,
	UndoBufferEmpty,
	VectorTooLarge,
}

impl Error {
	pub fn to_str(&self) -> &'static str {
		match self {
			Error::OutOfMemory => "Out of memory",
			Error::NotEnoughValues => "Not enough values",
			Error::NotARealNumber => "Not a real number",
			Error::InvalidInteger => "Invalid integer",
			Error::DataTypeMismatch => "Data type mismatch",
			Error::IncompatibleUnits => "Incompatible units",
			Error::InvalidEntry => "Invalid entry",
			Error::InvalidStackIndex => "Invalid stack index",
			Error::ValueNotDefined => "Value not defined",
			Error::ValueOutOfRange => "Value out of range",
			Error::IndexOutOfRange => "Index out of range",
			Error::FloatRequiresDecimalMode => "Requires decimal mode",
			Error::RequiresSizedIntegerMode => "Requires sized int mode",
			Error::InvalidDate => "Invalid date",
			Error::InvalidTime => "Invalid time",
			Error::CorruptData => "Corrupt data",
			Error::StackOverflow => "Stack overflow",
			Error::UndoBufferEmpty => "Undo buffer empty",
			Error::VectorTooLarge => "Vector too large",
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

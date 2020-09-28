use crate::error::{Error, Result};
use crate::storage::{
	store_reclaimable, DeserializeInput, SerializeOutput, StorageObject, StorageRef,
	StorageRefSerializer,
};
use crate::value::{Value, ValueRef};
use alloc::vec::Vec;
use spin::Mutex;

const MAX_UNDO_ENTRIES: usize = 100;

pub enum UndoAction {
	Push,
	Pop(ValueRef),
	Replace(Vec<ValueRef>),
	Swap(usize, usize),
	Clear(Vec<ValueRef>),
	RotateDown,
	SetStackEntry(usize, ValueRef),
	ReplaceTopWithMultiple(usize, ValueRef),
}

type UndoActionRef = StorageRef<UndoAction>;

pub struct UndoBuffer {
	entries: Vec<UndoActionRef>,
}

const UNDO_SERIALIZE_TYPE_PUSH: u8 = 0;
const UNDO_SERIALIZE_TYPE_POP: u8 = 1;
const UNDO_SERIALIZE_TYPE_REPLACE: u8 = 2;
const UNDO_SERIALIZE_TYPE_SWAP: u8 = 3;
const UNDO_SERIALIZE_TYPE_CLEAR: u8 = 4;
const UNDO_SERIALIZE_TYPE_ROTATE_DOWN: u8 = 5;
const UNDO_SERIALIZE_TYPE_SET_STACK_ENTRY: u8 = 6;
const UNDO_SERIALIZE_TYPE_REPLACE_TOP_WITH_MULTIPLE: u8 = 7;

impl StorageObject for UndoAction {
	fn serialize<Ref: StorageRefSerializer, Out: SerializeOutput>(
		&self,
		output: &mut Out,
		storage_refs: &mut Ref,
	) -> Result<()> {
		match self {
			UndoAction::Push => {
				output.write_u8(UNDO_SERIALIZE_TYPE_PUSH)?;
			}
			UndoAction::Pop(value) => {
				output.write_u8(UNDO_SERIALIZE_TYPE_POP)?;
				storage_refs.serialize(value, output)?;
			}
			UndoAction::Replace(values) => {
				output.write_u8(UNDO_SERIALIZE_TYPE_REPLACE)?;
				output.write_u32(values.len() as u32)?;
				for value in values {
					storage_refs.serialize(value, output)?;
				}
			}
			UndoAction::Swap(a, b) => {
				output.write_u8(UNDO_SERIALIZE_TYPE_SWAP)?;
				output.write_u32(*a as u32)?;
				output.write_u32(*b as u32)?;
			}
			UndoAction::Clear(values) => {
				output.write_u8(UNDO_SERIALIZE_TYPE_CLEAR)?;
				output.write_u32(values.len() as u32)?;
				for value in values {
					storage_refs.serialize(value, output)?;
				}
			}
			UndoAction::RotateDown => {
				output.write_u8(UNDO_SERIALIZE_TYPE_ROTATE_DOWN)?;
			}
			UndoAction::SetStackEntry(idx, value) => {
				output.write_u8(UNDO_SERIALIZE_TYPE_SET_STACK_ENTRY)?;
				output.write_u32(*idx as u32)?;
				storage_refs.serialize(value, output)?;
			}
			UndoAction::ReplaceTopWithMultiple(count, value) => {
				output.write_u8(UNDO_SERIALIZE_TYPE_REPLACE_TOP_WITH_MULTIPLE)?;
				output.write_u32(*count as u32)?;
				storage_refs.serialize(value, output)?;
			}
		}
		Ok(())
	}

	unsafe fn deserialize<T: StorageRefSerializer>(
		input: &mut DeserializeInput,
		storage_refs: &T,
	) -> Result<Self> {
		match input.read_u8()? {
			UNDO_SERIALIZE_TYPE_PUSH => Ok(UndoAction::Push),
			UNDO_SERIALIZE_TYPE_POP => Ok(UndoAction::Pop(storage_refs.deserialize(input)?)),
			UNDO_SERIALIZE_TYPE_REPLACE => {
				let count = input.read_u32()? as usize;
				let mut values = Vec::new();
				values.reserve(count);
				for _ in 0..count {
					values.push(storage_refs.deserialize(input)?);
				}
				Ok(UndoAction::Replace(values))
			}
			UNDO_SERIALIZE_TYPE_SWAP => {
				let a = input.read_u32()? as usize;
				let b = input.read_u32()? as usize;
				Ok(UndoAction::Swap(a, b))
			}
			UNDO_SERIALIZE_TYPE_CLEAR => {
				let count = input.read_u32()? as usize;
				let mut values = Vec::new();
				values.reserve(count);
				for _ in 0..count {
					values.push(storage_refs.deserialize(input)?);
				}
				Ok(UndoAction::Clear(values))
			}
			UNDO_SERIALIZE_TYPE_ROTATE_DOWN => Ok(UndoAction::RotateDown),
			UNDO_SERIALIZE_TYPE_SET_STACK_ENTRY => {
				let idx = input.read_u32()? as usize;
				let value = storage_refs.deserialize(input)?;
				Ok(UndoAction::SetStackEntry(idx, value))
			}
			UNDO_SERIALIZE_TYPE_REPLACE_TOP_WITH_MULTIPLE => {
				let count = input.read_u32()? as usize;
				let value = storage_refs.deserialize(input)?;
				Ok(UndoAction::ReplaceTopWithMultiple(count, value))
			}
			_ => Err(Error::CorruptData),
		}
	}
}

impl UndoBuffer {
	fn new() -> Self {
		UndoBuffer {
			entries: Vec::new(),
		}
	}

	fn push(&mut self, action: UndoActionRef) -> Result<()> {
		self.entries.push(action);
		while self.entries.len() > MAX_UNDO_ENTRIES {
			self.prune();
		}
		Ok(())
	}

	fn pop(&mut self) -> Result<UndoAction> {
		if let Some(action) = self.entries.pop() {
			action.get()
		} else {
			Err(Error::UndoBufferEmpty)
		}
	}

	fn prune(&mut self) -> bool {
		if self.entries.len() != 0 {
			self.entries.remove(0);
			true
		} else {
			false
		}
	}

	fn clear(&mut self) {
		self.entries.clear();
	}
}

lazy_static! {
	static ref UNDO_BUFFER: Mutex<UndoBuffer> = Mutex::new(UndoBuffer::new());
}

pub fn push_undo_action(action: UndoAction) {
	if let Ok(action) = store_reclaimable(action) {
		let _ = UNDO_BUFFER.lock().push(action);
	}
}

pub fn pop_undo_action() -> Result<UndoAction> {
	let entry = UNDO_BUFFER.lock().pop()?;

	// When popping entries off the stack, store any values back onto the non-reclaimable
	// storage so that it gets accounted for properly.
	Ok(match entry {
		UndoAction::Pop(value) => UndoAction::Pop(Value::deep_copy_value(value)?),
		UndoAction::Replace(mut values) => {
			for value in &mut values {
				*value = Value::deep_copy_value(value.clone())?;
			}
			UndoAction::Replace(values)
		}
		UndoAction::Clear(mut values) => {
			for value in &mut values {
				*value = Value::deep_copy_value(value.clone())?;
			}
			UndoAction::Clear(values)
		}
		UndoAction::SetStackEntry(idx, value) => {
			UndoAction::SetStackEntry(idx, Value::deep_copy_value(value)?)
		}
		UndoAction::ReplaceTopWithMultiple(count, value) => {
			UndoAction::ReplaceTopWithMultiple(count, Value::deep_copy_value(value)?)
		}
		entry => entry,
	})
}

pub fn prune_undo_buffer() -> bool {
	UNDO_BUFFER.lock().prune()
}

pub fn clear_undo_buffer() {
	UNDO_BUFFER.lock().clear()
}

use crate::error::{Error, Result};
use crate::format::IntegerMode;
use crate::number::Number;
use crate::storage::store;
use crate::undo::{clear_undo_buffer, pop_undo_action, push_undo_action, UndoAction};
use crate::value::{Value, ValueRef};
use num_bigint::ToBigInt;

#[cfg(not(feature = "std"))]
use alloc::boxed::Box;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg(feature = "limited_heap")]
const MAX_STACK_ENTRIES: usize = 1024;

pub struct Stack {
	entries: Vec<ValueRef>,
	push_new_entry: bool,
	empty: bool,
	undo: bool,
	notifications: Vec<Box<dyn Fn(&StackEvent)>>,
}

pub enum StackEvent {
	ValuePushed,
	ValuePopped,
	ValueChanged(usize),
	TopReplacedWithEntries(usize),
	RotateUp,
	Invalidate,
}

macro_rules! push_undo_action {
	($undo: expr, $action: expr) => {
		if $undo {
			push_undo_action($action);
			}
	};
}

impl Stack {
	pub fn new() -> Self {
		Stack {
			entries: Vec::new(),
			push_new_entry: false,
			empty: true,
			undo: false,
			notifications: Vec::new(),
		}
	}

	pub fn new_with_undo() -> Self {
		Stack {
			entries: Vec::new(),
			push_new_entry: false,
			empty: true,
			undo: true,
			notifications: Vec::new(),
		}
	}

	pub fn add_event_notify<T: 'static>(&mut self, notify_fn: T)
	where
		T: Fn(&StackEvent),
	{
		self.notifications.push(Box::new(notify_fn));
	}

	fn notify(&self, event: StackEvent) {
		for notify_fn in &self.notifications {
			notify_fn(&event);
		}
	}

	pub fn len(&self) -> usize {
		self.entries.len()
	}

	pub fn value_for_integer_mode(mode: &IntegerMode, value: Value) -> Value {
		match mode {
			IntegerMode::Float => value,
			IntegerMode::BigInteger => {
				if let Ok(int) = value.to_int_value() {
					int.into_owned()
				} else {
					value
				}
			}
			IntegerMode::SizedInteger(size, signed) => {
				if let Ok(int) = value.to_int() {
					let mask = 2.to_bigint().unwrap().pow(*size as u32) - 1.to_bigint().unwrap();
					let mut int = &*int & &mask;
					if *signed {
						let sign_bit = 2.to_bigint().unwrap().pow((*size - 1) as u32);
						if (&int & &sign_bit) != 0.to_bigint().unwrap() {
							int = -((int ^ mask) + 1.to_bigint().unwrap());
						}
					}
					Value::Number(Number::Integer(int))
				} else {
					value
				}
			}
		}
	}

	fn push_internal(&mut self, value: Value) -> Result<()> {
		#[cfg(feature = "limited_heap")]
		if self.entries.len() >= MAX_STACK_ENTRIES {
			return Err(Error::StackOverflow);
		}

		self.entries.push(store(value)?);

		self.notify(StackEvent::ValuePushed);
		self.push_new_entry = true;
		self.empty = false;
		Ok(())
	}

	pub fn push(&mut self, value: Value) -> Result<()> {
		self.push_internal(value)?;
		push_undo_action!(self.undo, UndoAction::Push);
		Ok(())
	}

	pub fn entry(&self, idx: usize) -> Result<Value> {
		let value_ref = self.entry_ref(idx)?;
		Ok(value_ref.get()?)
	}

	fn entry_ref(&self, idx: usize) -> Result<&ValueRef> {
		if idx >= self.entries.len() {
			return Err(Error::NotEnoughValues);
		}
		Ok(&self.entries[(self.entries.len() - 1) - idx])
	}

	fn entry_mut(&mut self, idx: usize) -> Result<&mut ValueRef> {
		if idx >= self.entries.len() {
			return Err(Error::NotEnoughValues);
		}

		self.notify(StackEvent::ValueChanged(idx));

		let len = self.entries.len();
		Ok(&mut self.entries[(len - 1) - idx])
	}

	fn set_entry_internal(&mut self, idx: usize, value: Value) -> Result<()> {
		if idx >= self.entries.len() {
			return Err(Error::NotEnoughValues);
		}
		let len = self.entries.len();
		let value_ref = store(value)?;
		self.entries[(len - 1) - idx] = value_ref;

		self.notify(StackEvent::ValueChanged(idx));
		self.empty = false;
		Ok(())
	}

	pub fn set_entry(&mut self, idx: usize, value: Value) -> Result<()> {
		if idx >= self.entries.len() {
			return Err(Error::NotEnoughValues);
		}
		let len = self.entries.len();
		let value_ref = store(value)?;
		push_undo_action!(
			self.undo,
			UndoAction::SetStackEntry(idx, self.entries[(len - 1) - idx].clone(),)
		);
		self.entries[(len - 1) - idx] = value_ref;

		self.notify(StackEvent::ValueChanged(idx));
		self.empty = false;
		Ok(())
	}

	pub fn top(&self) -> Result<Value> {
		self.entry(0)
	}

	fn top_ref(&self) -> Result<&ValueRef> {
		self.entry_ref(0)
	}

	fn set_top_internal(&mut self, value: Value) -> Result<()> {
		self.set_entry_internal(0, value)?;
		self.push_new_entry = true;
		self.empty = false;
		Ok(())
	}

	pub fn set_top(&mut self, value: Value) -> Result<()> {
		let old_value = self.top_ref()?.clone();
		self.set_top_internal(value)?;
		push_undo_action!(self.undo, UndoAction::Replace([old_value].to_vec()));
		Ok(())
	}

	fn replace_entries_internal(&mut self, count: usize, value: Value) -> Result<()> {
		if count > self.entries.len() {
			return Err(Error::NotEnoughValues);
		}

		// Replace what will be the top entry with the new value
		self.set_entry_internal(count - 1, value)?;

		// Remove consumed values
		for _ in 1..count {
			let _ = self.pop_internal();
		}

		self.push_new_entry = true;
		Ok(())
	}

	pub fn replace_entries(&mut self, count: usize, value: Value) -> Result<()> {
		if count > self.entries.len() {
			return Err(Error::NotEnoughValues);
		}
		let old_values = self.entries[self.entries.len() - count..].to_vec();
		self.replace_entries_internal(count, value)?;
		push_undo_action!(self.undo, UndoAction::Replace(old_values));
		Ok(())
	}

	pub fn replace_top_with_multiple(&mut self, items: Vec<ValueRef>) -> Result<()> {
		let old_value = self.top_ref()?.clone();
		if items.len() == 0 {
			self.pop_internal()?;
		} else {
			#[cfg(feature = "limited_heap")]
			if (self.entries.len() + items.len() - 1) >= MAX_STACK_ENTRIES {
				return Err(Error::StackOverflow);
			}

			*self.entry_mut(0).unwrap() = items[0].clone();
			self.entries.extend_from_slice(&items[1..]);

			self.notify(StackEvent::TopReplacedWithEntries(items.len()));
			self.push_new_entry = true;
			self.empty = false;
		}
		push_undo_action!(
			self.undo,
			UndoAction::ReplaceTopWithMultiple(items.len(), old_value)
		);
		Ok(())
	}

	fn pop_internal(&mut self) -> Result<ValueRef> {
		match self.entries.pop() {
			Some(value) => {
				self.notify(StackEvent::ValuePopped);
				Ok(value)
			}
			None => Err(Error::NotEnoughValues),
		}
	}

	pub fn pop(&mut self) -> Result<Value> {
		let value = self.pop_internal()?;
		push_undo_action!(self.undo, UndoAction::Pop(value.clone()));
		value.get()
	}

	fn swap_internal(&mut self, a_idx: usize, b_idx: usize) -> Result<()> {
		let a = self.entry_ref(a_idx)?.clone();
		let b = self.entry_ref(b_idx)?.clone();
		*self.entry_mut(a_idx)? = b;
		*self.entry_mut(b_idx)? = a;
		self.push_new_entry = true;
		Ok(())
	}

	pub fn swap(&mut self, a_idx: usize, b_idx: usize) -> Result<()> {
		self.swap_internal(a_idx, b_idx)?;
		push_undo_action!(self.undo, UndoAction::Swap(a_idx, b_idx));
		Ok(())
	}

	pub fn rotate_down(&mut self) {
		if self.entries.len() > 1 {
			push_undo_action!(self.undo, UndoAction::RotateDown);
			let top = self.top_ref().unwrap().clone();
			let _ = self.pop_internal();
			self.entries.insert(0, top);

			// Do not need to send notifications, as the pop_internal
			// call above will ensure that the modified entries have
			// sent notifications.
		}
	}

	fn rotate_up_internal(&mut self) {
		if self.entries.len() > 1 {
			let bottom = self.entries[0].clone();
			self.entries.remove(0);
			self.entries.push(bottom);

			self.notify(StackEvent::RotateUp);
			self.push_new_entry = true;
		}
	}

	pub fn clear(&mut self) {
		push_undo_action!(self.undo, UndoAction::Clear(self.entries.clone()));
		self.entries.clear();
		self.notify(StackEvent::Invalidate);
		self.push_new_entry = false;
		self.empty = true;
	}

	pub fn enter(&mut self) -> Result<()> {
		self.push(self.top()?.clone())?;
		self.push_new_entry = false;
		Ok(())
	}

	pub fn input_value(&mut self, value: Value) -> Result<()> {
		if self.push_new_entry {
			self.push(value)
		} else {
			self.set_top(value)
		}
	}

	pub fn clear_undo_buffer(&mut self) {
		if self.undo {
			clear_undo_buffer();
		}
	}

	pub fn undo(&mut self) -> Result<()> {
		if self.undo {
			match pop_undo_action()? {
				UndoAction::Push => {
					self.pop_internal()?;
				}
				UndoAction::Pop(value) => {
					if self.empty {
						self.set_top_internal(value.get()?)?;
					} else {
						self.push_internal(value.get()?)?;
					}
				}
				UndoAction::Replace(values) => {
					if values.len() == 0 {
						self.pop_internal()?;
					} else {
						self.set_top_internal(values[0].get()?)?;
						for value in &values[1..] {
							self.push_internal(value.get()?)?;
						}
					}
				}
				UndoAction::Swap(a, b) => {
					self.swap_internal(a, b)?;
				}
				UndoAction::Clear(values) => {
					let mut value_refs = Vec::new();
					for value in values.iter() {
						value_refs.push(store(value.get()?)?);
					}
					if !self.empty {
						value_refs.extend_from_slice(&self.entries);
					}
					self.entries = value_refs;
					self.notify(StackEvent::Invalidate);
					self.push_new_entry = true;
					//self.editor = None;
					self.empty = false;
				}
				UndoAction::RotateDown => {
					self.rotate_up_internal();
				}
				UndoAction::SetStackEntry(idx, value) => {
					self.set_entry_internal(idx, value.get()?)?;
				}
				UndoAction::ReplaceTopWithMultiple(count, value) => {
					self.replace_entries_internal(count, value.get()?)?;
				}
			}
			Ok(())
		} else {
			Err(Error::UndoBufferEmpty)
		}
	}

	pub fn invalidate_caches(&self) {
		self.notify(StackEvent::Invalidate);
	}
}

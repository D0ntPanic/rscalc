use crate::error::{Error, Result};
use crate::undo::prune_undo_buffer;
use core::alloc::Layout;
use core::marker::PhantomData;
use core::ptr::NonNull;
use linked_list_allocator::Heap;
use spin::Mutex;

#[cfg(not(feature = "std"))]
use alloc::boxed::Box;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

const STORAGE_SIZE: usize = 65536;
type OffsetType = u16;
type ReferenceType = u16;

pub trait StorageRefSerializer {
	fn serialize<T: StorageObject, Out: SerializeOutput>(
		&mut self,
		value: &StorageRef<T>,
		output: &mut Out,
	) -> Result<()>;
	fn serialize_array<T: StorageObject, Out: SerializeOutput>(
		&mut self,
		value: &StorageRefArray<T>,
		output: &mut Out,
	) -> Result<()>;
	unsafe fn deserialize<T: StorageObject>(
		&self,
		input: &mut DeserializeInput,
	) -> Result<StorageRef<T>>;
	unsafe fn deserialize_array<T: StorageObject>(
		&self,
		input: &mut DeserializeInput,
	) -> Result<StorageRefArray<T>>;
}

pub trait StorageObject: Sized {
	fn serialize<Ref: StorageRefSerializer, Out: SerializeOutput>(
		&self,
		output: &mut Out,
		storage_refs: &mut Ref,
	) -> Result<()>;
	unsafe fn deserialize<T: StorageRefSerializer>(
		input: &mut DeserializeInput,
		storage_refs: &T,
	) -> Result<Self>;
}

pub trait SerializeOutput {
	fn size_only(&self) -> bool;
	fn write(&mut self, data: &[u8]) -> Result<()>;

	fn write_u8(&mut self, value: u8) -> Result<()> {
		self.write(&[value])
	}

	fn write_i8(&mut self, value: i8) -> Result<()> {
		self.write(&[value as u8])
	}

	fn write_u16(&mut self, value: u16) -> Result<()> {
		self.write(&value.to_le_bytes())
	}

	fn write_i16(&mut self, value: i16) -> Result<()> {
		self.write(&value.to_le_bytes())
	}

	fn write_u32(&mut self, value: u32) -> Result<()> {
		self.write(&value.to_le_bytes())
	}

	fn write_i32(&mut self, value: i32) -> Result<()> {
		self.write(&value.to_le_bytes())
	}

	fn write_u64(&mut self, value: u64) -> Result<()> {
		self.write(&value.to_le_bytes())
	}

	fn write_i64(&mut self, value: i64) -> Result<()> {
		self.write(&value.to_le_bytes())
	}
}

struct SerializeBuffer<'a> {
	buffer: &'a mut [u8],
	offset: usize,
}

struct SerializeSizer {
	size: usize,
}

pub struct DeserializeInput<'a> {
	buffer: &'a [u8],
	offset: usize,
}

pub struct StorageRef<T: StorageObject> {
	offset: OffsetType,
	_type: PhantomData<T>,
}

pub struct StorageRefArray<T: StorageObject> {
	offset: OffsetType,
	len: usize,
	_type: PhantomData<T>,
}

struct StorageObjectHeader {
	size: OffsetType,
	refs: ReferenceType,
	reclaimable: bool,
}

struct NormalStorageRefSerializer {
	cleanup: Vec<Box<dyn FnOnce()>>,
}

struct DropStorageRefSerializer;

struct ReclaimableStorageRefSerializer {
	cleanup: Vec<Box<dyn FnOnce()>>,
}

impl NormalStorageRefSerializer {
	fn new() -> Self {
		NormalStorageRefSerializer {
			cleanup: Vec::new(),
		}
	}

	fn commit(&mut self) {
		self.cleanup.clear();
	}
}

impl Drop for NormalStorageRefSerializer {
	fn drop(&mut self) {
		for cleanup in self.cleanup.drain(..) {
			cleanup();
		}
	}
}

impl ReclaimableStorageRefSerializer {
	fn new() -> Self {
		ReclaimableStorageRefSerializer {
			cleanup: Vec::new(),
		}
	}

	fn commit(&mut self) {
		self.cleanup.clear();
	}
}

impl Drop for ReclaimableStorageRefSerializer {
	fn drop(&mut self) {
		for cleanup in self.cleanup.drain(..) {
			cleanup();
		}
	}
}

impl<'a> SerializeBuffer<'a> {
	fn new(slice: &'a mut [u8]) -> Self {
		SerializeBuffer {
			buffer: slice,
			offset: 0,
		}
	}
}

impl SerializeSizer {
	fn new() -> Self {
		SerializeSizer { size: 0 }
	}
}

impl<'a> SerializeOutput for SerializeBuffer<'a> {
	fn size_only(&self) -> bool {
		false
	}

	fn write(&mut self, data: &[u8]) -> Result<()> {
		if (self.offset + data.len()) > self.buffer.len() {
			return Err(Error::CorruptData);
		}

		&self.buffer[self.offset..self.offset + data.len()].copy_from_slice(data);
		self.offset += data.len();
		Ok(())
	}
}

impl SerializeOutput for SerializeSizer {
	fn size_only(&self) -> bool {
		true
	}

	fn write(&mut self, data: &[u8]) -> Result<()> {
		self.size += data.len();
		Ok(())
	}
}

impl<'a> DeserializeInput<'a> {
	fn new(slice: &'a [u8]) -> Self {
		DeserializeInput {
			buffer: slice,
			offset: 0,
		}
	}

	pub fn read(&mut self, data: &mut [u8]) -> Result<()> {
		if (self.offset + data.len()) > self.buffer.len() {
			return Err(Error::CorruptData);
		}

		data.copy_from_slice(&self.buffer[self.offset..self.offset + data.len()]);
		self.offset += data.len();
		Ok(())
	}

	pub fn read_u8(&mut self) -> Result<u8> {
		let mut buffer = [0; 1];
		self.read(&mut buffer)?;
		Ok(buffer[0])
	}

	pub fn read_i8(&mut self) -> Result<i8> {
		let mut buffer = [0; 1];
		self.read(&mut buffer)?;
		Ok(buffer[0] as i8)
	}

	pub fn read_u16(&mut self) -> Result<u16> {
		let mut buffer = [0; 2];
		self.read(&mut buffer)?;
		Ok(u16::from_le_bytes(buffer))
	}

	pub fn read_i16(&mut self) -> Result<i16> {
		let mut buffer = [0; 2];
		self.read(&mut buffer)?;
		Ok(i16::from_le_bytes(buffer))
	}

	pub fn read_u32(&mut self) -> Result<u32> {
		let mut buffer = [0; 4];
		self.read(&mut buffer)?;
		Ok(u32::from_le_bytes(buffer))
	}

	pub fn read_i32(&mut self) -> Result<i32> {
		let mut buffer = [0; 4];
		self.read(&mut buffer)?;
		Ok(i32::from_le_bytes(buffer))
	}

	pub fn read_u64(&mut self) -> Result<u64> {
		let mut buffer = [0; 8];
		self.read(&mut buffer)?;
		Ok(u64::from_le_bytes(buffer))
	}

	pub fn read_i64(&mut self) -> Result<i64> {
		let mut buffer = [0; 8];
		self.read(&mut buffer)?;
		Ok(i64::from_le_bytes(buffer))
	}
}

impl<T: StorageObject> StorageRef<T> {
	pub fn get(&self) -> Result<T> {
		let mut serializer = NormalStorageRefSerializer::new();
		let result = self.deserialize(&serializer)?;
		serializer.commit();
		Ok(result)
	}

	fn deserialize<Ref: StorageRefSerializer>(&self, storage_ref: &Ref) -> Result<T> {
		let (header, data) = obj_data(self.offset);
		unsafe {
			let size = (*header).size as usize;
			let data_slice = core::slice::from_raw_parts(data, size);
			let mut input = DeserializeInput::new(data_slice);
			T::deserialize(&mut input, storage_ref)
		}
	}

	fn add_ref(&self) {
		obj_add_ref(self.offset);
	}
}

impl<T: StorageObject> Clone for StorageRef<T> {
	fn clone(&self) -> Self {
		self.add_ref();
		StorageRef {
			offset: self.offset,
			_type: PhantomData,
		}
	}
}

impl<T: StorageObject> Drop for StorageRef<T> {
	fn drop(&mut self) {
		// Decrement the reference count of the object
		let (header_ptr, _) = obj_data_mut::<u8>(self.offset);
		unsafe {
			(*header_ptr).refs -= 1;
			if (*header_ptr).refs == 0 {
				// Last reference dropped, drop object from storage. First run the deserializer with the
				// reference dropper to drop all references to other objects.
				let _ = self.deserialize(&mut DropStorageRefSerializer);

				let reclaimable = (*header_ptr).reclaimable;
				let alloc_size =
					(*header_ptr).size as usize + core::mem::size_of::<StorageObjectHeader>();
				let prev_used_bytes = used_bytes();

				HEAP.lock().deallocate(
					core::ptr::NonNull::new_unchecked(header_ptr as *mut u8),
					Layout::from_size_align(
						alloc_size,
						core::mem::align_of::<StorageObjectHeader>(),
					)
					.unwrap(),
				);

				if reclaimable {
					*RECLAIMABLE.lock() -= prev_used_bytes - used_bytes();
				}
			}
		}
	}
}

impl<T: StorageObject> StorageRefArray<T> {
	pub fn new(len: usize, default_value: StorageRef<T>) -> Result<Self> {
		// Create a memory buffer large enough to hold the offsets of all values in the array
		let size = core::mem::size_of::<OffsetType>() * len;
		let (buffer, _alloc_size, _used_size) = alloc_obj(size, false)?;

		// Populate the array entries with the default value. Ensure that each use of the default
		// value increments its reference count.
		let array_buffer = (buffer.as_ptr() as usize + core::mem::size_of::<StorageObjectHeader>())
			as *mut OffsetType;
		let array_slice = unsafe { core::slice::from_raw_parts_mut(array_buffer, len) };
		for i in 0..len {
			array_slice[i] = default_value.offset;
			default_value.add_ref();
		}

		Ok(StorageRefArray {
			offset: (buffer.as_ptr() as usize - HEAP.lock().bottom()) as OffsetType,
			len,
			_type: PhantomData,
		})
	}

	/// Duplicates the array. This is used to create an independent copy when setting elements
	/// of a shared array.
	fn duplicate(&self, new_len: usize, default_value: Option<StorageRef<T>>) -> Result<Self> {
		// Create a memory buffer large enough to hold the offsets of all values in the array
		let size = core::mem::size_of::<OffsetType>() * new_len;
		let (buffer, _alloc_size, _used_size) = alloc_obj(size, false)?;

		// Duplicate the entries in the array
		let (_, old_array_buffer) = obj_data(self.offset);
		let new_array_buffer = (buffer.as_ptr() as usize
			+ core::mem::size_of::<StorageObjectHeader>()) as *mut OffsetType;
		let old_array_slice = unsafe { core::slice::from_raw_parts(old_array_buffer, self.len) };
		let new_array_slice = unsafe { core::slice::from_raw_parts_mut(new_array_buffer, new_len) };
		for i in 0..new_len {
			let offset = if i < self.len {
				old_array_slice[i]
			} else {
				default_value.clone().unwrap().offset
			};
			new_array_slice[i] = offset;
			obj_add_ref(offset);
		}

		Ok(StorageRefArray {
			offset: (buffer.as_ptr() as usize - HEAP.lock().bottom()) as OffsetType,
			len: new_len,
			_type: PhantomData,
		})
	}

	/// Duplicates the array and all of its contents into reclaimable memory. This is used for
	/// undo buffers and the like that allow normal storage to automatically free. Arrays used
	/// by this storage should also have the values referenced to also be stored in reclaimable
	/// memory.
	fn duplicate_reclaimable(&self) -> Result<Self> {
		// Create a memory buffer large enough to hold the offsets of all values in the array
		let size = core::mem::size_of::<OffsetType>() * self.len;
		let (buffer, alloc_size, used_size) = alloc_obj(size, true)?;

		// Duplicate the entries in the array. Values will be duplicated into reclaimable memory.
		let (_, old_array_buffer) = obj_data(self.offset);
		let new_array_buffer = (buffer.as_ptr() as usize
			+ core::mem::size_of::<StorageObjectHeader>()) as *mut OffsetType;
		let old_array_slice = unsafe { core::slice::from_raw_parts(old_array_buffer, self.len) };
		let new_array_slice =
			unsafe { core::slice::from_raw_parts_mut(new_array_buffer, self.len) };
		for i in 0..self.len {
			// Get a reference to the old value
			let old_offset = old_array_slice[i];
			obj_add_ref(old_offset);
			let old_value = StorageRef::<T> {
				offset: old_offset,
				_type: PhantomData,
			};

			// Try to store the value into reclaimable memory
			if let Ok(old_value) = old_value.get() {
				if let Ok(new_value) = store_reclaimable(old_value) {
					// Value stored successfully, store into array and continue
					new_array_slice[i] = new_value.offset;
					obj_add_ref(new_value.offset);
					continue;
				}
			}

			// Value could not be stored, free the array and fail the duplication
			for j in 0..i {
				// Drop values from array
				drop(StorageRef::<T> {
					offset: new_array_slice[j],
					_type: PhantomData,
				});
			}
			unsafe {
				HEAP.lock().deallocate(
					buffer,
					Layout::from_size_align(
						alloc_size,
						core::mem::align_of::<StorageObjectHeader>(),
					)
					.unwrap(),
				);
			}
			return Err(Error::OutOfMemory);
		}

		*RECLAIMABLE.lock() += used_size;

		Ok(StorageRefArray {
			offset: (buffer.as_ptr() as usize - HEAP.lock().bottom()) as OffsetType,
			len: self.len,
			_type: PhantomData,
		})
	}

	pub fn len(&self) -> usize {
		self.len
	}

	pub fn get(&self, idx: usize) -> Result<StorageRef<T>> {
		if idx >= self.len {
			return Err(Error::IndexOutOfRange);
		}

		let (_, buffer) = obj_data(self.offset);
		let array_slice = unsafe { core::slice::from_raw_parts(buffer, self.len) };
		let offset = array_slice[idx];
		obj_add_ref(offset);
		Ok(StorageRef {
			offset,
			_type: PhantomData,
		})
	}

	pub fn set(&mut self, idx: usize, mut value: StorageRef<T>) -> Result<()> {
		if idx >= self.len {
			return Err(Error::IndexOutOfRange);
		}

		let (header, mut buffer) = obj_data_mut::<OffsetType>(self.offset);
		if unsafe { (*header).refs } > 1 {
			// This is a shared reference, we must duplicate this array before writing to it.
			let mut new_array = self.duplicate(self.len, None)?;

			// Writing will be performed on the duplicated copy.
			let (_, new_buffer) = obj_data_mut::<OffsetType>(new_array.offset);
			buffer = new_buffer;

			// Exchange the offsets in this object and the freshly created duplicate. This
			// will make this instance of the reference point to the duplicated array, and
			// the old reference will automatically be dropped when new_array is dropped.
			core::mem::swap(&mut self.offset, &mut new_array.offset);
		}

		// Swap in the new value reference. We have ownership of the incoming value reference,
		// and it will be dropped at the end of this function. Swapping with the old value offset
		// will place the old reference into the incoming object, and the new value will be stored
		// in the array. This will cause the old value to be dropped and the new value will retain
		// the reference count from the incoming reference object.
		let array_slice = unsafe { core::slice::from_raw_parts_mut(buffer, self.len) };
		core::mem::swap(&mut array_slice[idx], &mut value.offset);
		Ok(())
	}

	pub fn with_size(&self, new_len: usize, default_value: StorageRef<T>) -> Result<Self> {
		self.duplicate(new_len, Some(default_value))
	}

	fn add_ref(&self) {
		obj_add_ref(self.offset);
	}
}

impl<T: StorageObject> Clone for StorageRefArray<T> {
	fn clone(&self) -> Self {
		self.add_ref();
		StorageRefArray {
			offset: self.offset,
			len: self.len,
			_type: PhantomData,
		}
	}
}

impl<T: StorageObject> Drop for StorageRefArray<T> {
	fn drop(&mut self) {
		// Decrement the reference count of the object
		let (header_ptr, buffer) = obj_data_mut::<OffsetType>(self.offset);
		unsafe {
			(*header_ptr).refs -= 1;
			if (*header_ptr).refs == 0 {
				// Last reference dropped, drop array elements.
				let array_slice = core::slice::from_raw_parts(buffer, self.len);
				for i in 0..self.len {
					// This will invoke the drop handler for the value reference, which will
					// decrement its reference count.
					drop(StorageRef::<T> {
						offset: array_slice[i],
						_type: PhantomData,
					});
				}

				let reclaimable = (*header_ptr).reclaimable;
				let alloc_size =
					(*header_ptr).size as usize + core::mem::size_of::<StorageObjectHeader>();
				let prev_used_bytes = used_bytes();

				HEAP.lock().deallocate(
					core::ptr::NonNull::new_unchecked(header_ptr as *mut u8),
					Layout::from_size_align(
						alloc_size,
						core::mem::align_of::<StorageObjectHeader>(),
					)
					.unwrap(),
				);

				if reclaimable {
					*RECLAIMABLE.lock() -= prev_used_bytes - used_bytes();
				}
			}
		}
	}
}

impl StorageRefSerializer for NormalStorageRefSerializer {
	fn serialize<T: StorageObject, Out: SerializeOutput>(
		&mut self,
		value: &StorageRef<T>,
		output: &mut Out,
	) -> Result<()> {
		// Serialize as the offset
		output.write(&value.offset.to_le_bytes())?;

		if output.size_only() {
			// If calculating size, don't touch reference counts
			return Ok(());
		}

		// Make sure to add a reference to the value so that the reference will stay valid
		// as long as the stored object lives. When the object that contains the reference
		// is dropped, we will call the deserializer without adding references and let
		// them drop, which will get rid of the references added here.
		let offset = value.offset;
		self.cleanup.push(Box::new(move || {
			// If there is an error later, we need to drop previous references to avoid
			// leaking unused references.
			drop(StorageRef::<T> {
				offset,
				_type: PhantomData,
			})
		}));
		value.add_ref();

		Ok(())
	}

	fn serialize_array<T: StorageObject, Out: SerializeOutput>(
		&mut self,
		value: &StorageRefArray<T>,
		output: &mut Out,
	) -> Result<()> {
		// Serialize as the offset
		output.write(&value.offset.to_le_bytes())?;

		if output.size_only() {
			// If calculating size, don't touch reference counts
			return Ok(());
		}

		// Make sure to add a reference to the value so that the reference will stay valid
		// as long as the stored object lives. When the object that contains the reference
		// is dropped, we will call the deserializer without adding references and let
		// them drop, which will get rid of the references added here.
		let offset = value.offset;
		let len = value.len;
		self.cleanup.push(Box::new(move || {
			// If there is an error later, we need to drop previous references to avoid
			// leaking unused references.
			drop(StorageRefArray::<T> {
				offset,
				len,
				_type: PhantomData,
			})
		}));
		value.add_ref();

		Ok(())
	}

	unsafe fn deserialize<T: StorageObject>(
		&self,
		input: &mut DeserializeInput,
	) -> Result<StorageRef<T>> {
		let mut buffer = [0; core::mem::size_of::<OffsetType>()];
		input.read(&mut buffer)?;
		let offset = OffsetType::from_le_bytes(buffer);
		let result = StorageRef {
			offset,
			_type: PhantomData,
		};
		result.add_ref();
		Ok(result)
	}

	unsafe fn deserialize_array<T: StorageObject>(
		&self,
		input: &mut DeserializeInput,
	) -> Result<StorageRefArray<T>> {
		let mut buffer = [0; core::mem::size_of::<OffsetType>()];
		input.read(&mut buffer)?;
		let offset = OffsetType::from_le_bytes(buffer);

		// Compute length of array from object header information
		let (header, _) = obj_data::<OffsetType>(offset);
		let len = (*header).size as usize / core::mem::size_of::<OffsetType>();

		let result = StorageRefArray {
			offset,
			len,
			_type: PhantomData,
		};
		result.add_ref();
		Ok(result)
	}
}

impl StorageRefSerializer for DropStorageRefSerializer {
	fn serialize<T: StorageObject, Out: SerializeOutput>(
		&mut self,
		value: &StorageRef<T>,
		output: &mut Out,
	) -> Result<()> {
		// Serialize as the offset
		output.write(&value.offset.to_le_bytes())?;
		Ok(())
	}

	fn serialize_array<T: StorageObject, Out: SerializeOutput>(
		&mut self,
		value: &StorageRefArray<T>,
		output: &mut Out,
	) -> Result<()> {
		// Serialize as the offset
		output.write(&value.offset.to_le_bytes())?;
		Ok(())
	}

	unsafe fn deserialize<T: StorageObject>(
		&self,
		input: &mut DeserializeInput,
	) -> Result<StorageRef<T>> {
		let mut buffer = [0; core::mem::size_of::<OffsetType>()];
		input.read(&mut buffer)?;
		let offset = OffsetType::from_le_bytes(buffer);
		let result = StorageRef {
			offset,
			_type: PhantomData,
		};
		// Do not add a reference here. This serializer is used when the value being
		// deserialized here is being dropped, which means we want to drop the references
		// that were stored in it. By not incrementing the reference here, when the
		// drop implementation runs on the value it will get rid of the reference made
		// by storing it once this result is dropped.
		Ok(result)
	}

	unsafe fn deserialize_array<T: StorageObject>(
		&self,
		input: &mut DeserializeInput,
	) -> Result<StorageRefArray<T>> {
		let mut buffer = [0; core::mem::size_of::<OffsetType>()];
		input.read(&mut buffer)?;
		let offset = OffsetType::from_le_bytes(buffer);

		// Compute length of array from object header information
		let (header, _) = obj_data::<OffsetType>(offset);
		let len = (*header).size as usize / core::mem::size_of::<OffsetType>();

		let result = StorageRefArray {
			offset,
			len,
			_type: PhantomData,
		};
		// Do not add a reference here. This serializer is used when the value being
		// deserialized here is being dropped, which means we want to drop the references
		// that were stored in it. By not incrementing the reference here, when the
		// drop implementation runs on the value it will get rid of the reference made
		// by storing it once this result is dropped.
		Ok(result)
	}
}

impl StorageRefSerializer for ReclaimableStorageRefSerializer {
	fn serialize<T: StorageObject, Out: SerializeOutput>(
		&mut self,
		value: &StorageRef<T>,
		output: &mut Out,
	) -> Result<()> {
		if output.size_only() {
			// If calculating size, don't touch reference counts
			output.write(&value.offset.to_le_bytes())?;
			return Ok(());
		}

		// Duplicate the object as reclaimable memory. This allows the old reference
		// to go away and be freed, and this object's dependencies will be counted
		// correctly as reclaimable.
		let new_value = store_reclaimable(value.get()?)?;

		// Serialize as the offset
		output.write(&new_value.offset.to_le_bytes())?;

		// Make sure to add a reference to the value so that the reference will stay valid
		// as long as the stored object lives. When the object that contains the reference
		// is dropped, we will call the deserializer without adding references and let
		// them drop, which will get rid of the references added here.
		let offset = new_value.offset;
		self.cleanup.push(Box::new(move || {
			// If there is an error later, we need to drop previous references to avoid
			// leaking unused references.
			drop(StorageRef::<T> {
				offset,
				_type: PhantomData,
			})
		}));
		new_value.add_ref();

		Ok(())
	}

	fn serialize_array<T: StorageObject, Out: SerializeOutput>(
		&mut self,
		value: &StorageRefArray<T>,
		output: &mut Out,
	) -> Result<()> {
		if output.size_only() {
			// If calculating size, don't touch reference counts
			output.write(&value.offset.to_le_bytes())?;
			return Ok(());
		}

		// Duplicate the object as reclaimable memory. This allows the old reference
		// to go away and be freed, and this object's dependencies will be counted
		// correctly as reclaimable.
		let new_value = value.duplicate_reclaimable()?;

		// Serialize as the offset
		output.write(&new_value.offset.to_le_bytes())?;

		// Make sure to add a reference to the value so that the reference will stay valid
		// as long as the stored object lives. When the object that contains the reference
		// is dropped, we will call the deserializer without adding references and let
		// them drop, which will get rid of the references added here.
		let offset = new_value.offset;
		let len = value.len;
		self.cleanup.push(Box::new(move || {
			// If there is an error later, we need to drop previous references to avoid
			// leaking unused references.
			drop(StorageRefArray::<T> {
				offset,
				len,
				_type: PhantomData,
			})
		}));
		new_value.add_ref();

		Ok(())
	}

	unsafe fn deserialize<T: StorageObject>(
		&self,
		input: &mut DeserializeInput,
	) -> Result<StorageRef<T>> {
		let mut buffer = [0; core::mem::size_of::<OffsetType>()];
		input.read(&mut buffer)?;
		let offset = OffsetType::from_le_bytes(buffer);
		let result = StorageRef {
			offset,
			_type: PhantomData,
		};
		result.add_ref();
		Ok(result)
	}

	unsafe fn deserialize_array<T: StorageObject>(
		&self,
		input: &mut DeserializeInput,
	) -> Result<StorageRefArray<T>> {
		let mut buffer = [0; core::mem::size_of::<OffsetType>()];
		input.read(&mut buffer)?;
		let offset = OffsetType::from_le_bytes(buffer);

		// Compute length of array from object header information
		let (header, _) = obj_data::<OffsetType>(offset);
		let len = (*header).size as usize / core::mem::size_of::<OffsetType>();

		let result = StorageRefArray {
			offset,
			len,
			_type: PhantomData,
		};
		result.add_ref();
		Ok(result)
	}
}

lazy_static! {
	static ref HEAP: Mutex<Heap> = unsafe {
		let layout = Layout::from_size_align(STORAGE_SIZE, 16).unwrap();

		#[cfg(feature = "std")]
		let backing_mem = std::alloc::alloc(layout);
		#[cfg(not(feature = "std"))]
		let backing_mem = alloc::alloc::alloc(layout);

		Mutex::new(Heap::new(backing_mem as usize, STORAGE_SIZE))
	};
	static ref RECLAIMABLE: Mutex<usize> = Mutex::new(0);
}

fn alloc_result(layout: Layout) -> Result<(NonNull<u8>, usize)> {
	loop {
		// Try allocating
		let prev_used_bytes = used_bytes();
		let result = HEAP.lock().allocate_first_fit(layout);
		match result {
			Ok(ptr) => return Ok((ptr, used_bytes() - prev_used_bytes)),
			Err(_) => (),
		};

		// If allocation fails, try to prune reclaimable memory. If there is no
		// more memory to reclaim, fail the allocation.
		if !prune_undo_buffer() {
			return Err(Error::OutOfMemory);
		}
	}
}

fn alloc_obj(size: usize, reclaimable: bool) -> Result<(NonNull<u8>, usize, usize)> {
	let alloc_size = size + core::mem::size_of::<StorageObjectHeader>();

	// Allocate a buffer with space for a reference count and the serialized contents
	let (buffer, used_size) = alloc_result(
		Layout::from_size_align(alloc_size, core::mem::align_of::<StorageObjectHeader>()).unwrap(),
	)?;

	// Initialize reference count and allocation length in header
	unsafe {
		*(buffer.as_ptr() as usize as *mut StorageObjectHeader) = StorageObjectHeader {
			size: size as OffsetType,
			refs: 1,
			reclaimable,
		};
	}

	Ok((buffer, alloc_size, used_size))
}

fn obj_add_ref(offset: OffsetType) {
	let header_ptr = (HEAP.lock().bottom() + offset as usize) as *mut StorageObjectHeader;
	unsafe {
		(*header_ptr).refs += 1;
	}
}

fn obj_data<T>(offset: OffsetType) -> (*const StorageObjectHeader, *const T) {
	let heap_bottom = HEAP.lock().bottom();
	let header = (heap_bottom + offset as usize) as *const StorageObjectHeader;
	let data =
		(heap_bottom + offset as usize + core::mem::size_of::<StorageObjectHeader>()) as *const T;
	(header, data)
}

fn obj_data_mut<T>(offset: OffsetType) -> (*mut StorageObjectHeader, *mut T) {
	let heap_bottom = HEAP.lock().bottom();
	let header = (heap_bottom + offset as usize) as *mut StorageObjectHeader;
	let data =
		(heap_bottom + offset as usize + core::mem::size_of::<StorageObjectHeader>()) as *mut T;
	(header, data)
}

fn store_obj<T: StorageObject>(value: T, reclaimable: bool) -> Result<StorageRef<T>> {
	// Determine the size of the serialized value
	let mut size = SerializeSizer::new();
	if reclaimable {
		let mut serializer = ReclaimableStorageRefSerializer::new();
		value.serialize(&mut size, &mut serializer)?;
		serializer.commit();
	} else {
		let mut serializer = NormalStorageRefSerializer::new();
		value.serialize(&mut size, &mut serializer)?;
		serializer.commit();
	}
	let size = size.size;

	let (buffer, alloc_size, used_size) = alloc_obj(size, reclaimable)?;

	// Serialize object into buffer
	let serialize_buffer =
		(buffer.as_ptr() as usize + core::mem::size_of::<StorageObjectHeader>()) as *mut u8;
	let serialize_slice = unsafe { core::slice::from_raw_parts_mut(serialize_buffer, size) };
	if let Err(error) = if reclaimable {
		let mut serializer = ReclaimableStorageRefSerializer::new();
		let result = value.serialize(&mut SerializeBuffer::new(serialize_slice), &mut serializer);
		if result.is_ok() {
			serializer.commit();
		}
		result
	} else {
		let mut serializer = NormalStorageRefSerializer::new();
		let result = value.serialize(&mut SerializeBuffer::new(serialize_slice), &mut serializer);
		if result.is_ok() {
			serializer.commit();
		}
		result
	} {
		// Serialization failed, deallocate and return error
		unsafe {
			HEAP.lock().deallocate(
				buffer,
				Layout::from_size_align(alloc_size, core::mem::align_of::<StorageObjectHeader>())
					.unwrap(),
			);
		}
		return Err(error);
	}

	if reclaimable {
		*RECLAIMABLE.lock() += used_size;
	}

	// Return offset into heap buffer as storage reference
	Ok(StorageRef {
		offset: (buffer.as_ptr() as usize - HEAP.lock().bottom()) as OffsetType,
		_type: PhantomData,
	})
}

/// Stores an object for long term storage. This will return failure when out of memory.
pub fn store<T: StorageObject>(value: T) -> Result<StorageRef<T>> {
	store_obj(value, false)
}

pub fn store_reclaimable<T: StorageObject>(value: T) -> Result<StorageRef<T>> {
	store_obj(value, true)
}

pub fn used_bytes() -> usize {
	HEAP.lock().used()
}

pub fn reclaimable_bytes() -> usize {
	*RECLAIMABLE.lock()
}

pub fn free_bytes() -> usize {
	HEAP.lock().free()
}

pub fn available_bytes() -> usize {
	free_bytes() + reclaimable_bytes()
}

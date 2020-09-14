use crate::error::{Error, Result};
use crate::undo::prune_undo_buffer;
use core::alloc::Layout;
use core::marker::PhantomData;
use core::ptr::NonNull;
use linked_list_allocator::Heap;
use spin::Mutex;

const STORAGE_SIZE: usize = 65536;
type OffsetType = u16;
type ReferenceType = u16;

pub trait StorageRefSerializer {
	fn serialize<T: StorageObject, Out: SerializeOutput>(
		&self,
		value: &StorageRef<T>,
		output: &mut Out,
	) -> Result<()>;
	unsafe fn deserialize<T: StorageObject>(
		&self,
		input: &mut DeserializeInput,
	) -> Result<StorageRef<T>>;
}

pub trait StorageObject: Sized {
	fn serialize<Ref: StorageRefSerializer, Out: SerializeOutput>(
		&self,
		output: &mut Out,
		storage_refs: &Ref,
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

struct StorageObjectHeader {
	size: OffsetType,
	refs: ReferenceType,
	reclaimable: bool,
}

struct NormalStorageRefSerializer;
struct DropStorageRefSerializer;
struct ReclaimableStorageRefSerializer;

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

	#[allow(dead_code)]
	pub fn read_u8(&mut self) -> Result<u8> {
		let mut buffer = [0; 1];
		self.read(&mut buffer)?;
		Ok(buffer[0])
	}

	#[allow(dead_code)]
	pub fn read_i8(&mut self) -> Result<i8> {
		let mut buffer = [0; 1];
		self.read(&mut buffer)?;
		Ok(buffer[0] as i8)
	}

	#[allow(dead_code)]
	pub fn read_u16(&mut self) -> Result<u16> {
		let mut buffer = [0; 2];
		self.read(&mut buffer)?;
		Ok(u16::from_le_bytes(buffer))
	}

	#[allow(dead_code)]
	pub fn read_i16(&mut self) -> Result<i16> {
		let mut buffer = [0; 2];
		self.read(&mut buffer)?;
		Ok(i16::from_le_bytes(buffer))
	}

	#[allow(dead_code)]
	pub fn read_u32(&mut self) -> Result<u32> {
		let mut buffer = [0; 4];
		self.read(&mut buffer)?;
		Ok(u32::from_le_bytes(buffer))
	}

	#[allow(dead_code)]
	pub fn read_i32(&mut self) -> Result<i32> {
		let mut buffer = [0; 4];
		self.read(&mut buffer)?;
		Ok(i32::from_le_bytes(buffer))
	}

	#[allow(dead_code)]
	pub fn read_u64(&mut self) -> Result<u64> {
		let mut buffer = [0; 8];
		self.read(&mut buffer)?;
		Ok(u64::from_le_bytes(buffer))
	}

	#[allow(dead_code)]
	pub fn read_i64(&mut self) -> Result<i64> {
		let mut buffer = [0; 8];
		self.read(&mut buffer)?;
		Ok(i64::from_le_bytes(buffer))
	}
}

impl<T: StorageObject> StorageRef<T> {
	pub fn get(&self) -> Result<T> {
		self.deserialize(&NormalStorageRefSerializer)
	}

	fn deserialize<Ref: StorageRefSerializer>(&self, storage_ref: &Ref) -> Result<T> {
		let heap_bottom = HEAP.lock().bottom();
		let header = (heap_bottom + self.offset as usize) as *const StorageObjectHeader;
		let data = (heap_bottom
			+ self.offset as usize
			+ core::mem::size_of::<StorageObjectHeader>()) as *const u8;
		unsafe {
			let size = (*header).size as usize;
			let data_slice = core::slice::from_raw_parts(data, size);
			let mut input = DeserializeInput::new(data_slice);
			T::deserialize(&mut input, storage_ref)
		}
	}

	fn add_ref(&self) {
		let header_ptr = (HEAP.lock().bottom() + self.offset as usize) as *mut StorageObjectHeader;
		unsafe {
			(*header_ptr).refs += 1;
		}
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
		// Last reference dropped, drop object from storage. First run the deserializer with the
		// reference dropper to drop all references to other objects.
		let _ = self.deserialize(&DropStorageRefSerializer);

		// Free the object from storage
		let header_ptr = (HEAP.lock().bottom() + self.offset as usize) as *mut StorageObjectHeader;
		unsafe {
			(*header_ptr).refs -= 1;
			if (*header_ptr).refs == 0 {
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
		&self,
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
}

impl StorageRefSerializer for DropStorageRefSerializer {
	fn serialize<T: StorageObject, Out: SerializeOutput>(
		&self,
		value: &StorageRef<T>,
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
}

impl StorageRefSerializer for ReclaimableStorageRefSerializer {
	fn serialize<T: StorageObject, Out: SerializeOutput>(
		&self,
		value: &StorageRef<T>,
		output: &mut Out,
	) -> Result<()> {
		// Duplicate the object as reclaimable memory. This allows the old reference
		// to go away and be freed, and this object's dependencies will be counted
		// correctly as reclaimable.
		let new_value = store_reclaimable(value.get()?)?;

		// Serialize as the offset
		output.write(&new_value.offset.to_le_bytes())?;

		if output.size_only() {
			// If calculating size, don't touch reference counts
			return Ok(());
		}

		// Make sure to add a reference to the value so that the reference will stay valid
		// as long as the stored object lives. When the object that contains the reference
		// is dropped, we will call the deserializer without adding references and let
		// them drop, which will get rid of the references added here.
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
}

lazy_static! {
	static ref HEAP: Mutex<Heap> = unsafe {
		let layout = Layout::from_size_align(STORAGE_SIZE, 16).unwrap();
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

fn store_obj<T: StorageObject>(value: T, reclaimable: bool) -> Result<StorageRef<T>> {
	// Determine the size of the serialized value
	let mut size = SerializeSizer::new();
	value.serialize(&mut size, &NormalStorageRefSerializer)?;
	let size = size.size;
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

	// Serialize object into buffer
	let serialize_buffer =
		(buffer.as_ptr() as usize + core::mem::size_of::<StorageObjectHeader>()) as *mut u8;
	let serialize_slice = unsafe { core::slice::from_raw_parts_mut(serialize_buffer, size) };
	if let Err(error) = if reclaimable {
		value.serialize(
			&mut SerializeBuffer::new(serialize_slice),
			&ReclaimableStorageRefSerializer,
		)
	} else {
		value.serialize(
			&mut SerializeBuffer::new(serialize_slice),
			&NormalStorageRefSerializer,
		)
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

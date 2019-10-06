use std::error::Error;
use std::fmt;
use std::mem;
use crate::types::*;
use crate::assembly::*;

// The two types of memory:
#[derive(Copy, Clone)]
pub enum Type
{
	Linear,
	DeviceIO,
}

impl Type
{
	// Determine the type of a given memory address from its address:
	pub fn from_address(address: Word) -> Type
	{
		if LINEAR_ADDRESS_SPACE_RANGE.contains(&address)
		{
			Type::Linear
		}
		else if DEVICE_IO_ADDRESS_SPACE_RANGE.contains(&address)
		{
			Type::DeviceIO
		}
		else
		{
			panic!("0x{:08X} is not a valid address (it must be in [0x{:08X}, 0x{:08X}]).", address.0, ADDRESS_SPACE_RANGE.start.0, ADDRESS_SPACE_RANGE.end.0 - 1);
		}
	}
}

// How many microcycles does the memory need to complete work?
const MICROCYCLES_PER_ACCESS: u8 = 3;

// The two ways of accessing memory:
#[derive(Copy, Clone)]
pub enum Access
{
	Read,
	Write,
}

// A pending memory access.
// Each microcycle decrements the number of remaining cycles.
// As soon as it falls to 0, a read result is available in SIR.
// Work is executed on copies of SAR and SIR. Changing them during its progress won't change the outcome.
pub struct Work
{
	pub mem_type: Type,
	pub access: Access,
	sar: Word,
	sir: Word,
	pub remaining_cycles: u8,
}

// This error type occurs when we load object code with unknown symbols:
#[derive(Debug)]
pub enum LinkError<'oc>
{
	UnknownDevice(&'oc str),
	UnknownDeviceLabel(&'oc str, &'oc str),
}

impl<'oc> fmt::Display for LinkError<'oc>
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		match self
		{
			LinkError::UnknownDevice(prefix) 				=> write!(f, "Symbol table references unknown device prefix: \"{:}\"", prefix),
			LinkError::UnknownDeviceLabel(prefix, name) 	=> write!(f, "Symbol table references unknown label name \"{:}\" of attached device \"{:}\".", name, prefix),
		}
	}
}

impl<'oc> Error for LinkError<'oc> { }

pub struct Unit
{
	// "Speicheradressregister" (SAR)
	// Holds the memory address to fetch from / write to
	pub sar: Word,

	// "Speicherinhaltsregister" (SIR)
	// Holds a word that has been fetched from / will be written to memory
	pub sir: Word,

	// Pending work:
	work: Option<Work>,

	// The non-DMA memory.
	// This is a linear, heap-allocated blob of host memory
	linear_memory: Box<[Word]>,
}

// Resolved symbols are generated from an object code symbol table:
struct ResolvedSymbol
{
	instruction_address: Word,
	device_address: Word,
}

impl ResolvedSymbol
{
	fn new(instruction_address: Word, device_address: Word) -> ResolvedSymbol
	{
		ResolvedSymbol
		{
			instruction_address,
			device_address,
		}
	}
}


impl Unit
{
	pub fn new() -> Unit
	{
		Unit
		{
			sar: Word(0),
			sir: Word(0),
			work: None,

			// Initialize all words to "Halt" to avoid stupid overflows:
			linear_memory: vec![Instruction::Halt.into(); LINEAR_ADDRESS_SPACE_WORDS].into_boxed_slice(),
		}
	}

	pub fn work(&self) -> Option<&Work>
	{
		self.work.as_ref()
	}

	pub fn linear_memory(&self) -> &[Word]
	{
		&self.linear_memory
	}

	pub fn load_code<'oc>(&mut self, code: &'oc ObjectCode) -> Result<(), LinkError<'oc>>
	{
		// Resolve the symbol table:
		let resolved_symbols = self.resolve_symbol_table(&code.symbol_table)?;

		// Load the raw object code:
		self.load_raw_code(&code.raw_code);

		// Now insert the resolved symbols:
		for symbol in resolved_symbols
		{
			self.linear_memory[symbol.instruction_address.0 as usize].0 &= symbol.device_address.0 & 0x0F_FF_FF_FFu32;
		}

		Ok(())
	}

	pub fn load_raw_code(&mut self, raw_code: &[Word])
	{
		assert!(raw_code.len() <= LINEAR_ADDRESS_SPACE_WORDS, "Raw code must not exceed the size of the linear address space ({} words == {} bytes).",
				LINEAR_ADDRESS_SPACE_WORDS, LINEAR_ADDRESS_SPACE_WORDS * mem::size_of::<Word>());

		// Copy the new image to offset 0:
		self.linear_memory[..raw_code.len()].clone_from_slice(raw_code);
	}

	pub fn load_mem_image(&mut self, mem_image: Box<[Word]>)
	{
		assert!(mem_image.len() == LINEAR_ADDRESS_SPACE_WORDS, "Memory image must exactly match the size of the linear address space ({} words == {} bytes).",
				LINEAR_ADDRESS_SPACE_WORDS, LINEAR_ADDRESS_SPACE_WORDS * mem::size_of::<Word>());

		// Move the box into ours:
		self.linear_memory = mem_image;
	}

	pub fn load_instructions(&mut self, instructions: &[Instruction])
	{
		assert!(instructions.len() <= LINEAR_ADDRESS_SPACE_WORDS, "Asembled instructions must not exceed the size of the linear address space ({} words == {} bytes).",
				LINEAR_ADDRESS_SPACE_WORDS, LINEAR_ADDRESS_SPACE_WORDS * mem::size_of::<Word>());

		// Assemble the instructions to offset 0:
		for (i, &instruction) in instructions.iter().enumerate()
		{
			self.linear_memory[i] = instruction.into();
		}
	}
}

impl Unit
{
	pub(crate) fn poll_work(&mut self)
	{
		// Perform memory work if necessary:
		if let Some(work) = self.work.as_mut()
		{
			if work.remaining_cycles == 0
			{
				let work = self.work.take().unwrap();

				// Linear memory or device I/O?
				match work.mem_type
				{
					Type::Linear 	=> self.finalize_work_linear(work),
					Type::DeviceIO 	=> self.finalize_work_device_io(work),
				}
			}
			else
			{
				work.remaining_cycles -= 1;
			}
		}
	}

	pub(crate) fn signal_memory(&mut self, access: Access)
	{
		assert!(self.work.is_none(), "Memory access is already in progress.");

		self.work = Some(Work
		{
			mem_type: Type::from_address(self.sar),
			access,
			sar: self.sar,
			sir: self.sir,
			remaining_cycles: MICROCYCLES_PER_ACCESS,
		});
	}
}

impl Unit
{
	fn finalize_work_linear(&mut self, work: Work)
	{
		// Access the linear memory:
		match work.access
		{
			Access::Read 	=> self.sir = self.linear_memory[work.sar.0 as usize],
			Access::Write 	=> self.linear_memory[work.sar.0 as usize] = work.sir,
		}
	}

	fn finalize_work_device_io(&mut self, work: Work)
	{
		// TODO
		match work.access
		{
			Access::Read 	=> self.sir = Word(42),
			Access::Write 	=> (),
		}
	}

	fn resolve_symbol_table<'oc>(&self, symbol_table: &'oc [Symbol]) -> Result<Vec<ResolvedSymbol>, LinkError<'oc>>
	{
		//TODO
		Ok(symbol_table.iter().map(|sym| ResolvedSymbol::new(sym.instruction_address, Word(0x0F_FF_FF_FFu32))).collect())
	}
}

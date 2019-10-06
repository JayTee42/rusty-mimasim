use bitflags::bitflags;
use std::fmt;
use std::ops::Range;

// A MiMA machine word (32 bit, newtype idiom):
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub struct Word(pub u32);

impl fmt::Display for Word
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		write!(f, "0x{:08X}", self.0)
	}
}

// A MiMA machine flag (boolean, newtype idiom):
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub struct Flag(pub bool);

// The MiMA address space size in address bits, bytes and words:
pub const ADDRESS_SPACE_BITS: usize 					= 28;
pub const ADDRESS_SPACE_WORDS: usize 					= (1usize << ADDRESS_SPACE_BITS);

// The uppermost quarter of the address space is device IO memory.
// The lower three quarters are linear memory.
pub const LINEAR_ADDRESS_SPACE_WORDS: usize 			= 3 * DEVICE_IO_ADDRESS_SPACE_WORDS;
pub const DEVICE_IO_ADDRESS_SPACE_WORDS: usize 			= ADDRESS_SPACE_WORDS / 4;

// The address space as ranges:
pub const ADDRESS_SPACE_RANGE: Range<Word> 				= Word(0)..Word(ADDRESS_SPACE_WORDS as u32);
pub const LINEAR_ADDRESS_SPACE_RANGE: Range<Word> 		= Word(0)..Word(LINEAR_ADDRESS_SPACE_WORDS as u32);
pub const DEVICE_IO_ADDRESS_SPACE_RANGE: Range<Word> 	= Word(LINEAR_ADDRESS_SPACE_WORDS as u32)..Word(ADDRESS_SPACE_WORDS as u32);


// There is also a flags type to hold register names.
// It is i. e. used for bus transfers.
bitflags!
{
	pub struct Registers: u16
	{
		// Arithmetic registers:
		const ACC 	= (1 << 0);
		const ONE 	= (1 << 1);
		const X 	= (1 << 2);
		const Y 	= (1 << 3);
		const Z 	= (1 << 4);

		// Control registers:
		const IAR 	= (1 << 5);
		const IR 	= (1 << 6);

		// Memory registers:
		const SAR 	= (1 << 7);
		const SIR 	= (1 << 8);
	}
}

impl Registers
{
	// A constant array of all register names (no iter() in bitflags ... yet?):
	pub const ALL_REGISTERS: [Registers; 9] =
	[
		Registers::ACC, Registers::ONE, Registers::X,   Registers::Y,
		Registers::Z,   Registers::IR,  Registers::IAR, Registers::SAR,
		Registers::SIR
	];
}

impl fmt::Display for Registers
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		// Get a vector of string slice representations of the flagged cases and join them:
		let strings: Vec<_> = Registers::ALL_REGISTERS.iter().filter(|&&dest| self.contains(dest)).map(|&dest|
		{
			match dest
			{
				Registers::ACC 		=> "ACC",
				Registers::ONE 		=> "ONE",
				Registers::X 		=> "X",
				Registers::Y 		=> "Y",
				Registers::Z 		=> "Z",
				Registers::IAR		=> "IAR",
				Registers::IR 		=> "IR",
				Registers::SAR		=> "SAR",
				Registers::SIR		=> "SIR",
				_ 					=> "",
			}
		}).collect();

		write!(f, "[{}]", strings.join(", "))
	}
}

// The MiMA instructions are an algebraic datatype:
#[derive(Copy, Clone)]
pub enum Instruction
{
	Add(Word),
	And(Word),
	Or(Word),
	Xor(Word),
	LoadValue(Word),
	StoreValue(Word),
	LoadConstant(Word),
	Jump(Word),
	JumpIfNegative(Word),
	Equals(Word),
	Halt,
	Not,
	RotateRight(Word),
	NoOperation,
}

// Disassemble instructions from machine words:
impl From<Word> for Instruction
{
	fn from(word: Word) -> Instruction
	{
		use Instruction::*;

		// Extract the opcode from the uppermost four bits:
		let opcode = word.0 >> 28;

		if opcode != 15
		{
			// Basic format:
			let payload = Word(word.0 & 0x0F_FF_FF_FFu32);

			match opcode
			{
				0x00 => Add(payload),
				0x01 => And(payload),
				0x02 => Or(payload),
				0x03 => Xor(payload),
				0x04 => LoadValue(payload),
				0x05 => StoreValue(payload),
				0x06 => LoadConstant(payload),
				0x07 => Jump(payload),
				0x08 => JumpIfNegative(payload),
				0x09 => Equals(payload),
				_ => NoOperation,
			}
		}
		else
		{
			// Extended format:
			let payload = Word(word.0 & 0x00_FF_FF_FFu32);

			match (word.0 & 0x0F_00_00_00u32) >> 24
			{
				0x00  => Halt,
				0x01  => Not,
				0x02  => RotateRight(payload),
				_  => NoOperation,
			}
		}
	}
}

// Assemble instructions to machine words:
impl From<Instruction> for Word
{
	fn from(instruction: Instruction) -> Word
	{
		use Instruction::*;

		// Determine opcode, format and payload:
		let (opcode, is_basic_format, Word(payload)): (u32, _, Word) = match instruction
		{
			// Basic format:
			Add(pl) 			=> (0x00, true, pl),
			And(pl) 			=> (0x01, true, pl),
			Or(pl) 				=> (0x02, true, pl),
			Xor(pl) 			=> (0x03, true, pl),
			LoadValue(pl) 		=> (0x04, true, pl),
			StoreValue(pl) 		=> (0x05, true, pl),
			LoadConstant(pl) 	=> (0x06, true, pl),
			Jump(pl) 			=> (0x07, true, pl),
			JumpIfNegative(pl) 	=> (0x08, true, pl),
			Equals(pl) 			=> (0x09, true, pl),

			// Extended format:
			Halt 				=> (0x00, false, Word(0)),
			Not 				=> (0x01, false, Word(0)),
			RotateRight(pl) 	=> (0x02, false, pl),
			NoOperation 		=> (0x0F, false, Word(0)),
		};

		// Basic (28 bit payload) or extended (24 bit payload)?
		if is_basic_format
		{
			assert!(payload <= 0x0F_FF_FF_FFu32, "Payload for basic format exceeded ({:08X} > {:08X}).", payload, 0x0F_FF_FF_FFu32);
			Word((opcode << 28) | payload)
		}
		else
		{
			assert!(payload <= 0x00_FF_FF_FFu32, "Payload for extended format exceeded ({:08X} > {:08X}).", payload, 0x00_FF_FF_FFu32);
			Word(0xF0_00_00_00u32 | (opcode << 24) | payload)
		}
	}
}

impl Instruction
{
	pub fn format_opcode(&self) -> &'static str
	{
		use Instruction::*;

		match self
		{
			Add(_) 				=> "ADD",
			And(_) 				=> "AND",
			Or(_) 				=> "OR",
			Xor(_) 				=> "XOR",
			LoadValue(_) 		=> "LDV",
			StoreValue(_) 		=> "STV",
			LoadConstant(_) 	=> "LDC",
			Jump(_) 			=> "JMP",
			JumpIfNegative(_) 	=> "JMN",
			Equals(_) 			=> "EQL",
			Halt 				=> "HLT",
			Not 				=> "NOT",
			RotateRight(_) 		=> "RAR",
			NoOperation 		=> "NOP",
		}
	}
}
use std::collections::{hash_map, HashMap};
use crate::types::*;
use crate::assembly::error::*;
use crate::assembly::parser::*;

// A fully-qualified label consists of a device namespace prefix and a name suffix:
pub struct Label
{
	pub prefix: String,
	pub name: String,
}

impl Label
{
	fn new(prefix: &str, name: &str) -> Label
	{
		Label
		{
			prefix: String::from(prefix),
			name: String::from(name),
		}
	}
}

// A symbol table contains a bunch of symbols (fully-qualified labels) and maps them to instruction addresses.
// It allows the memory unit to "link" the object code into an executable program.
pub struct Symbol
{
	pub instruction_address: Word,
	pub label: Label,
}

impl Symbol
{
	fn new(instruction_address: Word, label: Label) -> Symbol
	{
		Symbol
		{
			instruction_address,
			label,
		}
	}
}

// Object code consists of raw code and a symbol table:
pub struct ObjectCode
{
	pub raw_code: Box<[Word]>,
	pub symbol_table: Vec<Symbol>,
}

// The string representation of a program:
pub type ProgramRepr = String;

// A label map contains the line numbers and addresses of all local labels (no associated types in impls yet, not even private ...):
type LabelMap<'src> = HashMap<&'src str, (usize, Word)>;

impl ObjectCode
{
	// This placeholder address is inserted for yet unresolved device symbols.
	// Reads from and writes to this address will always trigger an error.
	const PLACEHOLDER_ADDR: Word = Word(ADDRESS_SPACE_RANGE.end.0 - 1);

	pub fn assemble_with_repr(input: &str) -> Result<(ObjectCode, Vec<Diagnostics>, ProgramRepr), AssemblerError>
	{
		// First, try to parse the program token from the input:
		let program = ProgramToken::parse(input)?;

		// Collect all the "locally" defined labels, their line numbers and addresses into a map.
		// The function also tells us the total number of words that is necessary to hold the program.
		let (label_map, number_of_words) = ObjectCode::build_label_map(&program)?;

		// Collect diagnostics into a vector:
		let mut diagnostics = vec![];

		// Create a word vector with the given capacity (=> avoids unnecessary allocations) and an empty symbol table:
		let mut raw_code = Vec::with_capacity(number_of_words);
		let mut symbols = vec![];

		// This helpful little closure takes an address token as it occurs in most instructions (and the address + line number of the corresponding instruction).
		// It resolves it into an address resp. creates a symbol table entry if necessary.
		// Because it might encounter a missing label, it returns a Result.
		let mut resolve_addr = |addr, instruction_address, line_number| -> Result<Word, LabelError>
		{
			match addr
			{
				AddressToken::Address(w) => Ok(w.0),
				AddressToken::Label(LabelIdentifierToken(prefix, name)) =>
				{
					if let Some(prefix) = prefix
					{
						// Append this position to the symbol table.
						// It must be resolved later.
						let label = Label::new(prefix, name);
						symbols.push(Symbol::new(instruction_address, label));

						// Return a magical address that will be replaced later:
						Ok(ObjectCode::PLACEHOLDER_ADDR)
					}
					else
					{
						// We have a local label.
						// It must be located in our label map.
						if let Some((_, addr)) = label_map.get(name)
						{
							Ok(*addr)
						}
						else
						{
							Err(LabelError::new(line_number, LabelErrorType::NotResolved(name)))
						}
					}
				},
			}
		};

		// Iterate through the program:
		for stmt in program.0.iter()
		{
			match stmt.content
			{
				Some(StatementContentToken::Data(data)) =>
				{
					for _ in 0..data.times()
					{
						raw_code.push(data.word());
					}
				},

				Some(StatementContentToken::Instruction(instruction)) =>
				{
					// Get addr and line number of the instruction:
					let addr = Word(raw_code.len() as u32);
					let line_number = stmt.line_number;

					// Assemble it:
					let word: Word = match instruction
					{
						InstructionToken::Add(a) 				=> Instruction::Add(resolve_addr(a, addr, line_number)?).into(),
						InstructionToken::And(a) 				=> Instruction::And(resolve_addr(a, addr, line_number)?).into(),
						InstructionToken::Or(a) 				=> Instruction::Or(resolve_addr(a, addr, line_number)?).into(),
						InstructionToken::Xor(a) 				=> Instruction::Xor(resolve_addr(a, addr, line_number)?).into(),
						InstructionToken::LoadValue(a) 			=> Instruction::LoadValue(resolve_addr(a, addr, line_number)?).into(),
						InstructionToken::StoreValue(a) 		=> Instruction::StoreValue(resolve_addr(a, addr, line_number)?).into(),
						InstructionToken::LoadConstant(w) 		=> Instruction::LoadConstant(w.0).into(),
						InstructionToken::Jump(a) 				=> Instruction::Jump(resolve_addr(a, addr, line_number)?).into(),
						InstructionToken::JumpIfNegative(a) 	=> Instruction::JumpIfNegative(resolve_addr(a, addr, line_number)?).into(),
						InstructionToken::Equals(a) 			=> Instruction::Equals(resolve_addr(a, addr, line_number)?).into(),
						InstructionToken::Halt 					=> Instruction::Halt.into(),
						InstructionToken::Not 					=> Instruction::Not.into(),
						InstructionToken::RotateRight(w) 		=> Instruction::RotateRight(w.0).into(),
						InstructionToken::NoOperation 			=> Instruction::NoOperation.into(),
					};

					raw_code.push(word);
				},
				_ => ()
			}
		}

		// We did it :)
		// Now consume the list of local labels and generate warning diagnostics for unused ones:
		ObjectCode::find_unused_labels(&program, label_map, &mut diagnostics);

		// Bundle code and symbol table into an object code struct and return it, along with the diagnostics:
		let object_code = ObjectCode
		{
			raw_code: raw_code.into_boxed_slice(),
			symbol_table: symbols,
		};

		Ok((object_code, diagnostics, format!("{:}", program)))
	}

	pub fn assemble(input: &str) -> Result<(ObjectCode, Vec<Diagnostics>), AssemblerError>
	{
		// Omit the string representation of the program:
		let (object_code, diagnostics, _) = ObjectCode::assemble_with_repr(input)?;
		Ok((object_code, diagnostics))
	}

	fn build_label_map<'src>(program: &ProgramToken<'src>) -> Result<(LabelMap<'src>, usize), AssemblerError<'src>>
	{
		let mut label_map = LabelMap::new();

		// Iterate through the program statements.
		// Track the addresses of the statements.
		// Use a 64-bit value to detect overflows.
		let mut number_of_words: u64 = 0;

		for stmt in program.0.iter()
		{
			// Iterate through the statement's label definitions.
			// Pattern matching ftw :O seriously, this is just awesome!
			for &LabelDefinitionToken(LabelIdentifierToken(prefix, name)) in stmt.label_defs.iter()
			{
				// If a local label has a prefix != "this", we have an error case:
				if let Some(prefix) = prefix
				{
					if prefix != "this"
					{
						return Err(LabelError::new(stmt.line_number, LabelErrorType::BadDefPrefix(prefix)).into());
					}
				}

				// Try to insert the label into our hashmap.
				// We have another error case if it is already present.
				match label_map.entry(name)
				{
					hash_map::Entry::Occupied(_) 	=> return Err(LabelError::new(stmt.line_number, LabelErrorType::Duplicate(name)).into()),
					hash_map::Entry::Vacant(entry) 	=>
					{
						// Yes, we have to validate the label address here.
						// Only validating the number of words at the increment after the loop is not enough:
						// A program that fills the complete linear memory of the MiMA is totally valid.
						// But if it is followed by a label, that label has an invalid address.
						if number_of_words >= (LINEAR_ADDRESS_SPACE_WORDS as u64)
						{
							return Err(LabelError::new(stmt.line_number, LabelErrorType::BehindFullMemory(name)).into());
						}
						else
						{
							entry.insert((stmt.line_number, Word(number_of_words as u32)));
						}
					},
				}
			}

			// Increment the number of words and check if it is still valid:
			number_of_words += stmt.required_words() as u64;

			if number_of_words > (LINEAR_ADDRESS_SPACE_WORDS as u64)
			{
				return Err(AssemblerError::OverflowError(stmt.line_number));
			}
		}

		Ok((label_map, number_of_words as usize))
	}

	fn find_unused_labels<'src>(program: &ProgramToken, mut label_map: LabelMap<'src>, diagnostics: &mut Vec<Diagnostics<'src>>)
	{
		// Iterate another time through the statements.
		// Remove every local label we encounter from the label map.
		for stmt in program.0.iter()
		{
			// Iterate through the instructions:
			let instruction = match stmt.content
			{
				Some(StatementContentToken::Instruction(i)) => i,
				_ => continue,
			};

			// Get an address token from the instruction:
			let addr_token = match instruction
			{
				InstructionToken::Add(a) 				|
				InstructionToken::And(a) 				|
				InstructionToken::Or(a) 				|
				InstructionToken::Xor(a) 				|
				InstructionToken::LoadValue(a) 			|
				InstructionToken::StoreValue(a) 		|
				InstructionToken::Jump(a) 				|
				InstructionToken::JumpIfNegative(a) 	|
				InstructionToken::Equals(a) => a,
				_ => continue,
			};

			// If there is a local label inside, remove it from the map:
			if let AddressToken::Label(LabelIdentifierToken(_, name)) = addr_token
			{
				label_map.remove(name);
			}
		}

		// Create a diagnostic entry for every remaining label (sorted by line):
		for (name, (line_number, _)) in label_map
		{
			diagnostics.push(Diagnostics::new(line_number, DiagnosticsType::UnusedLocalLabel(name)));
		}
	}
}

// Praise the nom :)
use std::fmt;
use nom::
{
	Err,
	IResult,
	branch::alt,
	bytes::complete::{tag, tag_no_case, take_while, take_while_m_n},
	character::complete::{char as single_char, not_line_ending, space0, space1},
	combinator::{all_consuming, map, map_res, opt, recognize},
	multi::many0,
	sequence::{delimited, pair, separated_pair, preceded, terminated, tuple},
};
use crate::assembly::error::*;
use crate::types::*;

//Note: "'src" is the lifetime of the string slice we parse our assembler program from.
// All references (parser result, labels, error messages, ...) annotated with this lifetime point into that original slice.

// A word token wraps a single machine word:
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub struct WordToken(pub Word);

impl fmt::Display for WordToken
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		write!(f, "{:}", self.0)
	}
}

// A label identifier name occurs in a label definition token (followed by ':') and in all references to that definition.
// It is an alphanumeric identifier with length > 0 (underscores are allowed, first char must not be a number).
// It might be prefixed by a device namespace (same rules for the characters as for the name itself).
// Prefix and name are separated by a'.' character.
// In the local namespace of an assembly program, labels with device prefix must not be defined ("this" as local prefix is allowed).
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub struct LabelIdentifierToken<'src>(pub Option<&'src str>, pub &'src str);

impl<'src> fmt::Display for LabelIdentifierToken<'src>
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		write!(f, "{:}{:}{:}", self.0.unwrap_or(""), "::", self.1)
	}
}

// Every instruction that takes an address payload can also take a label in our assembler dialect.
// To handle those cases correctly, we use another algebraic datatype for addresses.
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub enum AddressToken<'src>
{
	Address(WordToken),
	Label(LabelIdentifierToken<'src>),
}

impl<'src> fmt::Display for AddressToken<'src>
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		match self
		{
			AddressToken::Address(w) 	=> write!(f, "{:}({:})", "Address", w),
			AddressToken::Label(l) 		=> write!(f, "{:}({:})", "Label", l),
		}
	}
}

// A label definition token assigns an alphanumeric identifier to an address:
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub struct LabelDefinitionToken<'src>(pub LabelIdentifierToken<'src>);

impl<'src> fmt::Display for LabelDefinitionToken<'src>
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		write!(f, "{:}({:})", "LabelDefinition", self.0)
	}
}

// A data token represents a word definition with optional repitition count:
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub struct DataToken(WordToken, Option<WordToken>);

impl DataToken
{
	pub fn word(&self) -> Word
	{
		(self.0).0
	}

	pub fn times(&self) -> usize
	{
		self.1.map_or(1, |w| (w.0).0 as usize)
	}
}

impl fmt::Display for DataToken
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		write!(f, "{:}", self.0)?;

		if let Some(times) = self.1
		{
			write!(f, " {:} {:}", "x", (times.0).0)?;
		}

		Ok(())
	}
}

// Our instruction tokens (this enum corresponds to types::Instruction):
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub enum InstructionToken<'src>
{
	Add(AddressToken<'src>),
	And(AddressToken<'src>),
	Or(AddressToken<'src>),
	Xor(AddressToken<'src>),
	LoadValue(AddressToken<'src>),
	StoreValue(AddressToken<'src>),
	LoadConstant(WordToken),
	Jump(AddressToken<'src>),
	JumpIfNegative(AddressToken<'src>),
	Equals(AddressToken<'src>),
	Halt,
	Not,
	RotateRight(WordToken),
	NoOperation,
}

impl<'src> fmt::Display for InstructionToken<'src>
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		match self
		{
			InstructionToken::Add(a) 				=> write!(f, "{:}({:})", "add", a),
			InstructionToken::And(a) 				=> write!(f, "{:}({:})", "and", a),
			InstructionToken::Or(a) 				=> write!(f, "{:}({:})",  "or", a),
			InstructionToken::Xor(a) 				=> write!(f, "{:}({:})", "xor", a),
			InstructionToken::LoadValue(a) 			=> write!(f, "{:}({:})", "ldv", a),
			InstructionToken::StoreValue(a) 		=> write!(f, "{:}({:})", "stv", a),
			InstructionToken::LoadConstant(w) 		=> write!(f, "{:}({:})", "ldc", w),
			InstructionToken::Jump(a) 				=> write!(f, "{:}({:})", "jmp", a),
			InstructionToken::JumpIfNegative(a) 	=> write!(f, "{:}({:})", "jmn", a),
			InstructionToken::Equals(a) 			=> write!(f, "{:}({:})", "eql", a),
			InstructionToken::Halt 					=> write!(f, "{:}", "hlt"),
			InstructionToken::Not 					=> write!(f, "{:}", "not"),
			InstructionToken::RotateRight(w) 		=> write!(f, "{:}({:})", "rar", w),
			InstructionToken::NoOperation 			=> write!(f, "{:}", "nop"),
		}
	}
}

// A statement token wraps a list of 0...n label definition tokens.
// Optionally, it is followed by either a data or an instruction token.
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub enum StatementContentToken<'src>
{
	Data(DataToken),
	Instruction(InstructionToken<'src>),
}

impl<'src> fmt::Display for StatementContentToken<'src>
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		match self
		{
			StatementContentToken::Data(d) 			=> write!(f, "{:}({:})", "DataDefinition", d),
			StatementContentToken::Instruction(i) 	=> write!(f, "{:}({:})", "Instruction", i),
		}
	}
}

#[derive(Clone, PartialEq, PartialOrd)]
pub struct StatementToken<'src>
{
	pub line_number: usize,
	pub label_defs: Vec<LabelDefinitionToken<'src>>,
	pub content: Option<StatementContentToken<'src>>,
}

impl<'src> StatementToken<'src>
{
	fn new(line_number: usize, label_defs: Vec<LabelDefinitionToken<'src>>, content: Option<StatementContentToken<'src>>) -> StatementToken<'src>
	{
		StatementToken
		{
			line_number,
			label_defs,
			content,
		}
	}

	fn is_empty(&self) -> bool
	{
		(self.label_defs.len() == 0) && self.content.is_none()
	}

	// Determine the number of words that is necessary to assemble the content token:
	pub fn required_words(&self) -> usize
	{
		match self.content
		{
			Some(StatementContentToken::Data(d)) 			=> d.times(),
			Some(StatementContentToken::Instruction(_)) 	=> 1,
			_ 												=> 0,
		}
	}
}

impl<'src> fmt::Display for StatementToken<'src>
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		let mut parts = vec![];

		// Label definitions:
		for label_def in self.label_defs.iter()
		{
			parts.push(format!("{:}", label_def));
		}

		// Content:
		if let Some(content) = self.content
		{
			parts.push(format!("{:}", content));
		}

		write!(f, "[Line {:03}] {:}", self.line_number, parts.join(", "))
	}
}

// A program token holds a sequence of statement tokens:
#[derive(Clone, PartialEq, PartialOrd)]
pub struct ProgramToken<'src>(pub Vec<StatementToken<'src>>);

impl<'src> fmt::Display for ProgramToken<'src>
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		for stmt in self.0.iter()
		{
			writeln!(f, "{:}", stmt)?;
		}

		Ok(())
	}
}

fn word_token(i: &str) -> IResult<&str, WordToken>
{
	// Try to match the binary, hexadecimal, or decimal prefix.
	// If all of them fail, the decimal version without prefix must succeed.
	let opt_sign = opt(alt((single_char('+'), single_char('-'))));

	let prefixed_word_token_bin   = separated_pair(&opt_sign, tag("0b"), word_token_bin);
	let prefixed_word_token_dec   = separated_pair(&opt_sign, tag("0d"), word_token_dec);
	let prefixed_word_token_hex   = separated_pair(&opt_sign, tag("0x"), word_token_hex);
	let unprefixed_word_token_dec = pair(&opt_sign, word_token_dec);

	let result = map_res(alt((prefixed_word_token_bin, prefixed_word_token_dec, prefixed_word_token_hex, unprefixed_word_token_dec)), |(opt_sign, num)|
	{
		// Determine if we have a positive or negative sign.
		// No sign means positive.
		// Also treat 0 always as positive. That allows us to perform the 2's complement without wrapping.
		let is_negative = opt_sign.map_or(false, |s| if (s == '-') && (num > 0) { true } else { false });

		// We want to allow literals from [i32.min, u32.max] which will then be encoded as machine word.
		// Example: -1 will be mapped to 0xFF_FF_FF_FFu32.
		// As a consequence, we have to perform a range check in the negative case.
		if is_negative
		{
			if num <= 0x80_00_00_00u32
			{
				// Apply 2's complement:
				Ok(WordToken(Word(!num + 1)))
			}
			else
			{
				Err(())
			}
		}
		else
		{
			Ok(WordToken(Word(num)))
		}
	})(i)?;

	// Separate return step needed to drop "opt_sign" after temporaries ...
	Ok(result)
}

fn word_token_bin(i: &str) -> IResult<&str, u32>
{
	map_res(take_while_m_n(1, 32, |c: char| c.is_digit(2)), |s| u32::from_str_radix(s, 2))(i)
}

fn word_token_dec(i: &str) -> IResult<&str, u32>
{
	map_res(take_while_m_n(1, 10, |c: char| c.is_digit(10)), |s| u32::from_str_radix(s, 10))(i)
}

fn word_token_hex(i: &str) -> IResult<&str, u32>
{
	map_res(take_while_m_n(1, 8, |c: char| c.is_digit(16)), |s| u32::from_str_radix(s, 16))(i)
}

fn label_identifier_token(i: &str) -> IResult<&str, LabelIdentifierToken>
{
	// Match prefix and actual identifier as pair.
	// The first part is optional.
	let prefix = opt(terminated(label_identifier_token_part, single_char('.')));
	map(pair(prefix, label_identifier_token_part), |(p, n)| LabelIdentifierToken(p, n))(i)
}

fn label_identifier_token_part(i: &str) -> IResult<&str, &str>
{
	// We want an alphabetic char (+ '_') at the beginning and 0...n alphanumeric (+ '_') trailing chars.
	// Match as pair and extract the full byte length of the identifier:
	let cond_alpha = |c: char| c.is_alphabetic() || (c == '_');
	let cond_alphanum = |c: char| c.is_alphanumeric() || (c == '_');

	recognize(pair(take_while_m_n(1, 1, cond_alpha), take_while(cond_alphanum)))(i)
}

fn address_token(i: &str) -> IResult<&str, AddressToken>
{
	// Match either a word or a label identifier and map both to our algebraic data type:
	let word_match = map(word_token, |t| AddressToken::Address(t));
	let label_identifier_match = map(label_identifier_token, |t| AddressToken::Label(t));

	alt((word_match, label_identifier_match))(i)
}

fn label_definition_token(i: &str) -> IResult<&str, LabelDefinitionToken>
{
	// Match identifier (terminated by ':') and wrap it:
	map(terminated(label_identifier_token, single_char(':')), |i| LabelDefinitionToken(i))(i)
}

fn data_token(i: &str) -> IResult<&str, DataToken>
{
	// First, we have the actual definition of a word, preceded by "dat" and at least one space:
	let definition = preceded(pair(tag_no_case("dat"), space1), word_token);

	// Then there might be a repitition count.
	// It is a word, preceded by [space1, "times", space1].
	let repitition = preceded(tuple((space1, tag_no_case("times"), space1)), word_token);

	// Assemble everything:
	map(pair(definition, opt(repitition)), |(d, t)| DataToken(d, t))(i)
}

fn instruction_token(i: &str) -> IResult<&str, InstructionToken>
{
	// Match on one big alternative of all the instructions.
	// Some instructions are simple case-insensitive tags.
	// All others are words (ldc, rar) or addresses, preceded by a case-insensitive tag and at least one space.
	let instr_address_arg 	= |opcode| preceded(pair(tag_no_case(opcode), space1), address_token);
	let instr_word_arg		= |opcode| preceded(pair(tag_no_case(opcode), space1), word_token);
	let instr_no_arg 		= |opcode| tag_no_case(opcode);

	// "Return" construct needed for the borrow checker ...
	return alt
	((
		|s| map(instr_address_arg("add"), 	|a| InstructionToken::Add(a))(s),
		|s| map(instr_address_arg("and"), 	|a| InstructionToken::And(a))(s),
		|s| map(instr_address_arg("or"), 	|a| InstructionToken::Or(a))(s),
		|s| map(instr_address_arg("xor"), 	|a| InstructionToken::Xor(a))(s),
		|s| map(instr_address_arg("ldv"), 	|a| InstructionToken::LoadValue(a))(s),
		|s| map(instr_address_arg("stv"), 	|a| InstructionToken::StoreValue(a))(s),
		|s| map(instr_word_arg("ldc"), 		|w| InstructionToken::LoadConstant(w))(s),
		|s| map(instr_address_arg("jmp"), 	|a| InstructionToken::Jump(a))(s),
		|s| map(instr_address_arg("jmn"), 	|a| InstructionToken::JumpIfNegative(a))(s),
		|s| map(instr_address_arg("eql"), 	|a| InstructionToken::Equals(a))(s),
		|s| map(instr_no_arg("hlt"), 		|_| InstructionToken::Halt)(s),
		|s| map(instr_no_arg("not"), 		|_| InstructionToken::Not)(s),
		|s| map(instr_word_arg("rar"), 		|w| InstructionToken::RotateRight(w))(s),
		|s| map(tag_no_case("nop"), 		|_| InstructionToken::NoOperation)(s),
	))(i);
}

fn comment_token(i: &str) -> IResult<&str, ()>
{
	// First '#', then anything except line ending.
	// Drop it all, though.
	map(pair(single_char('#'), not_line_ending), |_| ())(i)
}

// The input string must not contain a line ending!
fn statement_token(line_number: usize, i: &str) -> Result<Option<StatementToken>, ParserError>
{
	// The labels are a whitespace-separated list.
	// We cannot use "separated_list" or "many0" in direct combination with "space0" because of nom's endless-loop-detection (see https://github.com/Geal/nom/issues/834).
	// Therefore, we parse a label as being terminated with "space0".
	let label_defs = many0(terminated(label_definition_token, space0));

	// The data / instruction token (both mapped to a statement content token for type soundness) is an alternative:
	let stmt_content_data = map(data_token, |t| StatementContentToken::Data(t));
	let stmt_content_instruction = map(instruction_token, |t| StatementContentToken::Instruction(t));
	let stmt_content = alt((stmt_content_data, stmt_content_instruction));

	// Combine both parts.
	// The statement content is optional.
	let center = pair(label_defs, opt(stmt_content));

	// The statement start is optional whitespace:
	let start = space0;

	// The statement end is a sequence of 0...n whitespaces and an optional comment:
	let end = pair(space0, opt(comment_token));

	// Now combine everything and capture the center.
	// The input string must be completely consumed.
	// Because we consume the whole input here, it makes no sense to pass the remaining input on.
	// As a consequence, we map nom's result type to a Result<Option(StatementToken), ParserError> in the end.
	// For empty statements, we return Ok(None).
	map(all_consuming(delimited(start, center, end)), |(l, s)| StatementToken::new(line_number, l, s))(i)
		.map(|(_, stmt)| if stmt.is_empty() { None } else { Some(stmt) })
		.map_err(|err|
		{
			let token = match err
			{
				Err::Error(tuple) | Err::Failure(tuple) 	=> Some(tuple.0),
				_ 											=> None,
			};

			ParserError::new(line_number, token)
		})
}

// Expose a public interface for parsing a program token from a string slice:
impl<'src> ProgramToken<'src>
{
	// The input string contains the statements, separated by line endings.
	pub fn parse(input: &str) -> Result<ProgramToken, ParserError>
	{
		// Iterate through the lines.
		// Generate line numbers.
		// Construct a statement token from each line number and line.
		// Transpose Result<Option<StatementToken>> to Option<Result<StatementToken>> and filter => iterator over Result<StatementToken, _>.
		// Then collect into a vector until we have them all or an error occurs.
		let statements = input.lines()
			.enumerate()
			.filter_map(|(line_number, line)| statement_token(line_number, line).transpose())
			.collect::<Result<_, _>>()?;

		Ok(ProgramToken(statements))
	}
}

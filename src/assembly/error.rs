use std::convert::From;
use std::error::Error;
use std::fmt;
use crate::types::*;

// Diagnostics (warnings) help users to improve their otherwise correct code:
pub struct Diagnostics<'src>
{
	line_number: usize,
	diag_type: DiagnosticsType<'src>,
}

impl<'src> Diagnostics<'src>
{
	pub fn new(line_number: usize, diag_type: DiagnosticsType<'src>) -> Diagnostics<'src>
	{
		Diagnostics
		{
			line_number,
			diag_type,
		}
	}
}

impl<'src> fmt::Display for Diagnostics<'src>
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		write!(f, "[Line {:}] Warning: {:}", self.line_number, self.diag_type)
	}
}

pub enum DiagnosticsType<'src>
{
	UnusedLocalLabel(&'src str),
}

impl<'src> fmt::Display for DiagnosticsType<'src>
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		match self
		{
			DiagnosticsType::UnusedLocalLabel(s) => write!(f, "The local label \"{:}\" is never referenced.", s)
		}
	}
}

// Parsing is error-prone.
// We use this custom error type to return some diagnostics.
#[derive(Debug)]
pub struct ParserError<'src>
{
	line_number: usize,
	token: Option<&'src str>,
}

impl<'src> ParserError<'src>
{
	pub fn new(line_number: usize, token: Option<&'src str>) -> ParserError
	{
		ParserError
		{
			line_number,
			token,
		}
	}
}

impl<'src> fmt::Display for ParserError<'src>
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		write!(f, "[Line {:03}] Error: Failed to parse token starting at \"{:32}\".", self.line_number, self.token.unwrap_or("???"))
	}
}

impl<'src> Error for ParserError<'src> { }

// A wrong usage of a label in a syntactically correct program:
#[derive(Debug)]
pub struct LabelError<'src>
{
	line_number: usize,
	err_type: LabelErrorType<'src>,
}

impl<'src> LabelError<'src>
{
	pub fn new(line_number: usize, err_type: LabelErrorType<'src>) -> LabelError<'src>
	{
		LabelError
		{
			line_number,
			err_type,
		}
	}
}

impl<'src> fmt::Display for LabelError<'src>
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		write!(f, "[Line {:}] {:}", self.line_number, self.err_type)
	}
}

impl<'src> Error for LabelError<'src> { }

#[derive(Debug)]
pub enum LabelErrorType<'src>
{
	BadDefPrefix(&'src str),
	Duplicate(&'src str),
	BehindFullMemory(&'src str),
	NotResolved(&'src str),
}

impl<'src> fmt::Display for LabelErrorType<'src>
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		match self
		{
			LabelErrorType::BadDefPrefix(s) =>
			{
				write!(f, "A device prefix like \"{:}\" is not allowed in a local label definition.", s)?;
				write!(f, "If you want to prefix your local label, please use \"this\".")
			},
			LabelErrorType::Duplicate(s) => write!(f, "The label definition \"{:}\" is a duplicate.", s),
			LabelErrorType::BehindFullMemory(s) => write!(f, "The label definition \"{:}\" is located at an invalid address.", s),
			LabelErrorType::NotResolved(s) => write!(f, "The label reference \"{:}\" cannot be resolved.", s),
		}
	}
}

// This is a compound error type that wraps all the other ones:
#[derive(Debug)]
pub enum AssemblerError<'src>
{
	ParserError(ParserError<'src>),
	LabelError(LabelError<'src>),
	OverflowError(usize),
}

impl<'src> From<ParserError<'src>> for AssemblerError<'src>
{
	fn from(err: ParserError<'src>) -> Self
	{
		AssemblerError::ParserError(err)
	}
}

impl<'src> From<LabelError<'src>> for AssemblerError<'src>
{
	fn from(err: LabelError<'src>) -> Self
	{
		AssemblerError::LabelError(err)
	}
}

impl<'src> fmt::Display for AssemblerError<'src>
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		match self
		{
			AssemblerError::ParserError(err) 			=> write!(f, "{:}", err),
			AssemblerError::LabelError(err) 			=> write!(f, "{:}", err),
			AssemblerError::OverflowError(line_number) 	=> write!(f, "[Line {:}] The maximum number of machine words ({:}) is exceeded.", line_number, LINEAR_ADDRESS_SPACE_WORDS),
		}
	}
}

impl<'src> Error for AssemblerError<'src> { }

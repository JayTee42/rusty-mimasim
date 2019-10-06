use std::num::Wrapping;
use crate::types::*;

// How many microcycles does the ALU need to complete work?
const MICROCYCLES_PER_OP: u8 = 1;

// All the operations that can be performed by the ALU:
#[derive(Copy, Clone)]
pub enum Operation
{
	Add,
	And,
	Or,
	Xor,
	Equals,
	Not,
	RotateRight,
}

// A pending ALU calculation.
// Each microcycle decrements the number of remaining cycles.
// As soon as it falls to 0, the ALU result is available in Z.
// Work is executed on copies of X and Y. Changing them during its progress won't change the outcome.
pub struct Work
{
	pub op: Operation,
	x: Word,
	y: Word,
	pub remaining_cycles: u8,
}

pub struct Unit
{
	// "Accumulator" (ACC)
	// General purpose register (calculations, memory, ...)
	pub acc: Word,

	// "Einsregister" (ONE)
	// Holds a constant value of 1
	pub one: Word,

	// "X", "Y", "Z"
	// Input and output for the ALU
	pub x: Word,
	pub y: Word,
	pub z: Word,

	// Pending work:
	work: Option<Work>
}

impl Unit
{
	pub fn new() -> Unit
	{
		Unit
		{
			acc: Word(0),
			one: Word(1),
			x: Word(0),
			y: Word(0),
			z: Word(0),
			work: None,
		}
	}

	pub fn work(&self) -> Option<&Work>
	{
		self.work.as_ref()
	}
}

impl Unit
{
	pub(crate) fn poll_work(&mut self)
	{
		if let Some(work) = self.work.as_mut()
		{
			if work.remaining_cycles > 0
			{
				work.remaining_cycles -= 1;
			}
			else
			{
				let work = self.work.take().unwrap();
				self.finalize_work(work);
			}
		}
	}

	pub(crate) fn signal_alu(&mut self, op: Operation)
	{
		assert!(self.work.is_none(), "ALU operation is already in progress.");

		self.work = Some(Work
		{
			op,
			x: self.x,
			y: self.y,
			remaining_cycles: MICROCYCLES_PER_OP,
		});
	}
}

impl Unit
{
	fn finalize_work(&mut self, work: Work)
	{
		self.z = Word(match work.op
		{
			Operation::Add 			=> (Wrapping(work.x.0) + Wrapping(work.y.0)).0,
			Operation::And 			=> work.x.0 & work.y.0,
			Operation::Or 			=> work.x.0 | work.y.0,
			Operation::Xor 			=> work.x.0 ^ work.y.0,
			Operation::Equals 		=> if work.x == work.y { 0xFF_FF_FF_FFu32 } else { 0u32 },
			Operation::Not 			=> !work.x.0,
			Operation::RotateRight 	=>
			{
				let rot = work.y.0 % 32;
				(work.x.0 >> rot) | (work.x.0 << rot)
			},
		});
	}
}

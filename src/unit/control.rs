use crate::types::*;

// The control unit encapsulates a status field.
// It contains various flags.
pub struct Status
{
	// The RUN flag indicates if the MiMA is running (true) or halted (false).
	pub run: Flag,

	// The TRA flag indicates if control has been handed over to an external device.
	pub tra: Flag,
}

impl Status
{
	pub fn new() -> Status
	{
		Status
		{
			run: Flag(true),
			tra: Flag(false),
		}
	}
}

pub struct Unit
{
	// "Instruktionsadressregister" (IAR)
	// Holds the memory address of the next instruction to be loaded and processed
	// Also known as PC (Program Counter) or IP (Instruction Pointer)
	pub iar: Word,

	// "Instruktionsregister" (IR)
	// Holds the instruction that is currently processed
	pub ir: Word,

	// The status field:
	status: Status,

	// The microcycle counter
	// It holds the index of the microcycle we are about to perform next (always in [1, 12]):
	microcycle: u8,

	// The current instruction (only available during microcycles [6, 12]):
	instruction: Option<Instruction>,
}

impl Unit
{
	pub fn new() -> Unit
	{
		Unit
		{
			iar: Word(0),
			ir: Word(0),
			status: Status::new(),
			microcycle: 1,
			instruction: None,
		}
	}

	pub fn status(&self) -> &Status
	{
		&self.status
	}

	pub fn microcycle(&self) -> u8
	{
		self.microcycle
	}

	pub fn instruction(&self) -> Option<Instruction>
	{
		self.instruction
	}

	pub fn is_running(&self) -> bool
	{
		self.status.run.0
	}
}

impl Unit
{
	pub(crate) fn end_microcycle(&mut self)
	{
		match self.microcycle
		{
			5 =>
			{
				// The fetch phase ends now. We can decode the instruction from IR.
				self.instruction = Some(Instruction::from(self.ir));
			},

			12 =>
			{
				// If the instruction is HLT, we reset the RUN flag now to stop the MiMA.
				if let Instruction::Halt = self.instruction.expect("Instruction must be present in execution phase!")
				{
					self.status.run = Flag(false);
				}

				// The execute phase ends now. Drop the instruction.
				self.instruction = None;
			},

			_ => ()
		}

		// Set the counter for the next microcycle:
		if self.microcycle == 12
		{
			//Back to fetching:
			self.microcycle = 1;
		}
		else
		{
			self.microcycle += 1;
		}
	}

	pub(crate) fn start_xfer(&mut self)
	{
		assert!(!self.status.tra.0, "A transfer is already in progress.");
		self.status.tra = Flag(true);
	}

	pub(crate) fn stop_xfer(&mut self)
	{
		assert!(self.status.tra.0, "No transfer is in progress.");
		self.status.tra = Flag(false)
	}
}

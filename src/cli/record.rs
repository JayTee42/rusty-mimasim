use mimasim::types::*;
use mimasim::microcycle::Descriptor as MicrocycleDescriptor;
use mimasim::unit::*;
use mimasim::mima::Mima;

// Information about register / flag values and changes are stored in enums to record changes:
#[derive(Copy, Clone)]
pub enum Value<T>
{
	Stasis(T),
	Change(T, T),
}

impl<T> Value<T>
{
	pub fn initial_value(&self) -> T
		where T: Copy
	{
		match self
		{
			Value::Change(w, _) 	=> *w,
			Value::Stasis(w) 		=> *w,
		}
	}

	pub fn final_value(&self) -> T
		where T: Copy
	{
		match self
		{
			Value::Stasis(w) 		=> *w,
			Value::Change(_, w) 	=> *w,
		}
	}
}

impl<T> Value<T>
	where T: PartialEq
{
	fn make_diff(self, new: T) -> Self
	{
		match self
		{
			Value::Stasis(old) 		=> if old != new { Value::Change(old, new) } else { Value::Stasis(old) },
			Value::Change(_, _) 	=> panic!("Tried to diff a value that is already marked as changed."),
		}
	}
}

pub type RegisterValue = Value<Word>;
pub type FlagValue = Value<Flag>;

// This struct allows to record a "flat" summary of all events that occur during a microcycle.
// For all registers, there are old and new values.
// We also include information about ALU and memory work and new operations at the end of the cycle.
pub struct MicrocycleSummary
{
	// Arithmetic unit registers (without "one", it is constant):
	pub acc: RegisterValue,
	pub x: RegisterValue,
	pub y: RegisterValue,
	pub z: RegisterValue,

	// ALU work (at the beginning of the microcycle):
	pub alu_work: Option<(ALUOperation, u8)>,

	// Control unit registers:
	pub iar: RegisterValue,
	pub ir: RegisterValue,

	// Control unit flags:
	pub run: FlagValue,
	pub tra: FlagValue,

	// Microcycle index and instruction:
	pub microcycle: u8,
	pub instruction: Option<Instruction>,

	// Memory unit registers:
	pub sar: RegisterValue,
	pub sir: RegisterValue,

	// Memory work (at the beginning of the microcycle):
	pub mem_work: Option<(MemoryType, MemoryAccess, u8)>,

	// The descriptor for this microcycle:
	pub descriptor: MicrocycleDescriptor
}

impl MicrocycleSummary
{
	pub fn record_microcycle(mima: &mut Mima) -> Option<MicrocycleSummary>
	{
		// Record the state before executing the microcycle (= everything still in stasis):
		let mut acc = RegisterValue::Stasis(mima.arithmetic_unit.acc);
		let mut x = RegisterValue::Stasis(mima.arithmetic_unit.x);
		let mut y = RegisterValue::Stasis(mima.arithmetic_unit.y);
		let mut z = RegisterValue::Stasis(mima.arithmetic_unit.z);
		let alu_work = mima.arithmetic_unit.work().map(|work| (work.op, work.remaining_cycles));

		let mut iar = RegisterValue::Stasis(mima.control_unit.iar);
		let mut ir = RegisterValue::Stasis(mima.control_unit.ir);
		let mut run = FlagValue::Stasis(mima.control_unit.status().run);
		let mut tra = FlagValue::Stasis(mima.control_unit.status().tra);
		let microcycle = mima.control_unit.microcycle();
		let instruction = mima.control_unit.instruction();

		let mut sar = RegisterValue::Stasis(mima.memory_unit.sar);
		let mut sir = RegisterValue::Stasis(mima.memory_unit.sir);
		let mem_work = mima.memory_unit.work().map(|work| (work.mem_type, work.access, work.remaining_cycles));

		// Now execute the cycle.
		// If it returns None because the MiMA is stopped, we are done.
		// Otherwise, we have the descriptor.
		if let Some(descriptor) = mima.perform_microcycle()
		{
			// Look for changes in the registers / flags:
			acc = acc.make_diff(mima.arithmetic_unit.acc);
			x = x.make_diff(mima.arithmetic_unit.x);
			y = y.make_diff(mima.arithmetic_unit.y);
			z = z.make_diff(mima.arithmetic_unit.z);

			iar = iar.make_diff(mima.control_unit.iar);
			ir = ir.make_diff(mima.control_unit.ir);
			run = run.make_diff(mima.control_unit.status().run);
			tra = tra.make_diff(mima.control_unit.status().tra);

			sar = sar.make_diff(mima.memory_unit.sar);
			sir = sir.make_diff(mima.memory_unit.sir);

			// Summarize everything^^
			Some(MicrocycleSummary
			{
				acc, x, y, z, alu_work,
				iar, ir, run, tra, microcycle, instruction,
				sar, sir, mem_work, descriptor,
			})
		}
		else
		{
			None
		}
	}

	pub fn is_bus_active(&self) -> bool
	{
		match self.descriptor.bus_xfer.as_ref()
		{
			Some(xfer) if xfer.is_acc_dependent() 	=> (self.acc.initial_value().0 & (1u32 << 31)) != 0,
			Some(_) 								=> true,
			None 									=> false,
		}
	}
}

pub struct CycleSummary
{
	// The accumulator and the program counter:
	pub acc: RegisterValue,
	pub iar: RegisterValue,

	// The flags:
	pub run: FlagValue,
	pub tra: FlagValue,

	// The instruction that has been executed:
	pub instruction: Instruction,
}

impl CycleSummary
{
	pub fn from_microcycle_summaries(start: &MicrocycleSummary, end: &MicrocycleSummary) -> CycleSummary
	{
		// The start must be in microcycle 1 and the end in microcycle 12:
		debug_assert!((start.microcycle == 1) && (end.microcycle == 12), "Microcycle summaries for cycle summary must be from microcycle 1 and 12.");

		// Calculate the state diff between the two cycles:
		let acc = RegisterValue::Stasis(start.acc.initial_value()).make_diff(end.acc.final_value());
		let iar = RegisterValue::Stasis(start.iar.initial_value()).make_diff(end.iar.final_value());
		let run = FlagValue::Stasis(start.run.initial_value()).make_diff(end.run.final_value());
		let tra = FlagValue::Stasis(start.tra.initial_value()).make_diff(end.tra.final_value());

		// Take the instruction from the end:
		let instruction = end.instruction.expect("Microcycle summary at the end must contain instruction.");

		CycleSummary
		{
			acc,
			iar,
			run,
			tra,
			instruction,
		}
	}
}

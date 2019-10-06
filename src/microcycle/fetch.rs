use crate::types::Registers as Regs;
use crate::unit::{ALUOperation::*, MemoryAccess::*};
use super::descriptor::Descriptor;

// Helper for a new, empty descriptor:
fn empty_desc() -> Descriptor
{
	Descriptor::empty()
}

// Return the microcycle descriptor for the microcycle in [1, 5]:
pub fn descriptor(microcycle: u8) -> Descriptor
{
	debug_assert!((1..=5).contains(&microcycle), "Fetch microcycles must be in [1, 5].");

	match microcycle
	{
		1 => empty_desc().with_bus_xfer(Regs::IAR, Regs::SAR | Regs::X).with_mem_access(Read),
		2 => empty_desc().with_bus_xfer(Regs::ONE, Regs::Y).with_alu_op(Add),
		4 => empty_desc().with_bus_xfer(Regs::Z, Regs::IAR),
		5 => empty_desc().with_bus_xfer(Regs::SIR, Regs::IR),
		_ => empty_desc(),
	}
}

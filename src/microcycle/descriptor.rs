use crate::types::{*, Registers as Regs};
use crate::bus::Xfer as BusXfer;
use crate::unit::{ALUOperation, MemoryAccess};

// A microcycle descriptor encapsulates an optional bus transfer, an optional ALU signal and an optional memory signal.
pub struct Descriptor
{
	pub bus_xfer: Option<BusXfer>,
	pub alu_op: Option<ALUOperation>,
	pub mem_access: Option<MemoryAccess>,
}

impl Descriptor
{
	// Builder pattern:
	pub(crate) fn empty() -> Descriptor
	{
		Descriptor
		{
			bus_xfer: None,
			alu_op: None,
			mem_access: None,
		}
	}

	// Full source bitmask:
	pub(crate) fn with_bus_xfer(mut self, source: Regs, destinations: Regs) -> Descriptor
	{
		self.bus_xfer = Some(BusXfer::new(source, destinations, BusXfer::SOURCE_BITMASK_FULL));
		self
	}

	// Custom source bitmask:
	pub(crate) fn with_masked_bus_xfer(mut self, source: Regs, destinations: Regs, source_bitmask: Word) -> Descriptor
	{
		self.bus_xfer = Some(BusXfer::new(source, destinations, source_bitmask));
		self
	}

	// Mark as accumulator-dependent:
	pub(crate) fn acc_dependent(mut self) -> Descriptor
	{
		self.bus_xfer.as_mut().expect("Create bus transfer first!").make_acc_dependent();
		self
	}

	pub(crate) fn with_alu_op(mut self, alu_op: ALUOperation) -> Descriptor
	{
		self.alu_op = Some(alu_op);
		self
	}

	pub(crate) fn with_mem_access(mut self, mem_access: MemoryAccess) -> Descriptor
	{
		self.mem_access = Some(mem_access);
		self
	}
}

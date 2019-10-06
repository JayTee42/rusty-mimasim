use crate::types::{*, Registers as Regs};
use crate::unit::*;
use crate::bus::Xfer as BusXfer;
use crate::microcycle::{self, Descriptor as MicrocycleDescriptor};

pub struct Mima
{
	// The units of the MiMA:
	pub arithmetic_unit: ArithmeticUnit,
	pub control_unit: ControlUnit,
	pub memory_unit: MemoryUnit,
}

impl Mima
{
	pub fn new() -> Mima
	{
		Mima
		{
			arithmetic_unit: ArithmeticUnit::new(),
			control_unit: ControlUnit::new(),
			memory_unit: MemoryUnit::new(),
		}
	}

	// Perform a microcycle.
	// Return the descriptor in the end to allow graphical output of the microcycle.
	pub fn perform_microcycle(&mut self) -> Option<MicrocycleDescriptor>
	{
		// Is the MiMA running?
		// Otherwise, we don't do anything.
		if !self.control_unit.is_running()
		{
			//TODO: Logging
			return None
		}

		// First, let arithmetic and memory unit continue pending work:
		self.arithmetic_unit.poll_work();
		self.memory_unit.poll_work();

		// Get the current microcycle index from the control unit:
		let microcycle = self.control_unit.microcycle();

		// Obtain the microcycle descriptor and process it.
		// If there is an instruction inside the control unit, we are already in the execute stage.
		// Otherwise, a fetch is in progress.
		let microcycle_desc = self.control_unit.instruction()
								.map(|instruction| microcycle::execute_descriptor(microcycle, instruction))
								.unwrap_or_else(|| microcycle::fetch_descriptor(microcycle));

		self.process_microcycle_descriptor(&microcycle_desc);

		// The control unit ends the microcycle by manipulating the instruction and incrementing the counter.
		self.control_unit.end_microcycle();

		// Return the descriptor to the caller for it to be rendered graphically.
		Some(microcycle_desc)
	}
}

impl Mima
{
	// Process the given microcycle descriptor.
	fn process_microcycle_descriptor(&mut self, microcycle_desc: &MicrocycleDescriptor)
	{
		// Is there a bus transfer?
		if let Some(bus_xfer) = &microcycle_desc.bus_xfer
		{
			self.perform_bus_xfer(bus_xfer)
		}

		// Do we have to signal the ALU?
		if let Some(alu_op) = microcycle_desc.alu_op
		{
			self.perform_alu_signal(alu_op);
		}

		// Do we have to signal the memory?
		if let Some(mem_access) = microcycle_desc.mem_access
		{
			self.perform_mem_signal(mem_access);
		}
	}

	fn perform_bus_xfer(&mut self, bus_xfer: &BusXfer)
	{
		// Cancel accumulator-dependent bus transfers that are not satisfied:
		if bus_xfer.is_acc_dependent() && ((self.arithmetic_unit.acc.0 & (1u32 << 31)) == 0)
		{
			return;
		}

		// Fetch the source and mask it accordingly:
		let value = Word(bus_xfer.source_bitmask().0 &
		(
			match bus_xfer.source()
			{
				Regs::ACC 	=> self.arithmetic_unit.acc,
				Regs::ONE 	=> self.arithmetic_unit.one,
				Regs::Z 	=> self.arithmetic_unit.z,
				Regs::IAR 	=> self.control_unit.iar,
				Regs::IR 	=> self.control_unit.ir,
				Regs::SIR 	=> self.memory_unit.sir,
			_ 			=> panic!("Unexpected bus source"),
			}
		).0);

		// Write it to all indicated destinations:
		for &dest in Regs::ALL_REGISTERS.iter().filter(|&&dest| bus_xfer.destinations().contains(dest))
		{
			match dest
			{
				Regs::ACC 	=> self.arithmetic_unit.acc = value,
				Regs::X 	=> self.arithmetic_unit.x = value,
				Regs::Y 	=> self.arithmetic_unit.y = value,
				Regs::IAR 	=> self.control_unit.iar = value,
				Regs::IR 	=> self.control_unit.ir = value,
				Regs::SAR 	=> self.memory_unit.sar = value,
				Regs::SIR 	=> self.memory_unit.sir = value,
				_ 			=> panic!("Unexpected bus destination"),
			}
		}
	}

	fn perform_alu_signal(&mut self, alu_op: ALUOperation)
	{
		self.arithmetic_unit.signal_alu(alu_op);
	}

	fn perform_mem_signal(&mut self, mem_access: MemoryAccess)
	{
		// If the memory access will be I/O, we have to frame it with the TRA bit:
		let is_xfer = match MemoryType::from_address(self.memory_unit.sar)
		{
			MemoryType::Linear 		=> false,
			MemoryType::DeviceIO 	=> true
		};

		if is_xfer
		{
			self.control_unit.start_xfer();
		}

		self.memory_unit.signal_memory(mem_access);

		if is_xfer
		{
			self.control_unit.stop_xfer();
		}
	}
}

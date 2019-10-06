use crate::types::{*, Registers as Regs};
use crate::bus::Xfer as BusXfer;
use crate::unit::{ALUOperation::*, MemoryAccess::*};
use super::descriptor::Descriptor;

// Helper for a new, empty descriptor:
fn empty_desc() -> Descriptor
{
	Descriptor::empty()
}

// Return the microcycle descriptor for the microcycle in [6, 12] and current instruction:
pub fn descriptor(microcycle: u8, instruction: Instruction) -> Descriptor
{
	debug_assert!((6..=12).contains(&microcycle), "Execution microcycles must be in [6, 12].");

	// Sub-methods for better structure.
	// Note: The enum payloads are not used at this point!
	// We will extract that information from the raw machine word via masked bus transfers.
	match instruction
	{
		Instruction::Add(_)				=> descriptor_add(microcycle),
		Instruction::And(_)				=> descriptor_and(microcycle),
		Instruction::Or(_) 				=> descriptor_or(microcycle),
		Instruction::Xor(_)				=> descriptor_xor(microcycle),
		Instruction::LoadValue(_) 		=> descriptor_load_value(microcycle),
		Instruction::StoreValue(_) 		=> descriptor_store_value(microcycle),
		Instruction::LoadConstant(_)	=> descriptor_load_constant(microcycle),
		Instruction::Jump(_)			=> descriptor_jump(microcycle),
		Instruction::JumpIfNegative(_) 	=> descriptor_jump_if_negative(microcycle),
		Instruction::Equals(_) 			=> descriptor_equals(microcycle),
		Instruction::Halt 				=> descriptor_halt(microcycle),
		Instruction::Not 				=> descriptor_not(microcycle),
		Instruction::RotateRight(_) 	=> descriptor_rotate_right(microcycle),
		Instruction::NoOperation 		=> descriptor_no_operation(microcycle),
	}
}

fn descriptor_add(microcycle: u8) -> Descriptor
{
	match microcycle
	{
		6 	=> empty_desc().with_masked_bus_xfer(Regs::IR, Regs::SAR, BusXfer::SOURCE_BITMASK_BASIC_PAYLOAD).with_mem_access(Read),
		7 	=> empty_desc().with_bus_xfer(Regs::ACC, Regs::X),
		10 	=> empty_desc().with_bus_xfer(Regs::SIR, Regs::Y).with_alu_op(Add),
		12 	=> empty_desc().with_bus_xfer(Regs::Z, Regs::ACC),
		_ 	=> empty_desc(),
	}
}

fn descriptor_and(microcycle: u8) -> Descriptor
{
	match microcycle
	{
		6 	=> empty_desc().with_masked_bus_xfer(Regs::IR, Regs::SAR, BusXfer::SOURCE_BITMASK_BASIC_PAYLOAD).with_mem_access(Read),
		7 	=> empty_desc().with_bus_xfer(Regs::ACC, Regs::X),
		10 	=> empty_desc().with_bus_xfer(Regs::SIR, Regs::Y).with_alu_op(And),
		12 	=> empty_desc().with_bus_xfer(Regs::Z, Regs::ACC),
		_ 	=> empty_desc(),
	}
}

fn descriptor_or(microcycle: u8) -> Descriptor
{
	match microcycle
	{
		6 	=> empty_desc().with_masked_bus_xfer(Regs::IR, Regs::SAR, BusXfer::SOURCE_BITMASK_BASIC_PAYLOAD).with_mem_access(Read),
		7 	=> empty_desc().with_bus_xfer(Regs::ACC, Regs::X),
		10 	=> empty_desc().with_bus_xfer(Regs::SIR, Regs::Y).with_alu_op(Or),
		12 	=> empty_desc().with_bus_xfer(Regs::Z, Regs::ACC),
		_ 	=> empty_desc(),
	}
}

fn descriptor_xor(microcycle: u8) -> Descriptor
{
	match microcycle
	{
		6 	=> empty_desc().with_masked_bus_xfer(Regs::IR, Regs::SAR, BusXfer::SOURCE_BITMASK_BASIC_PAYLOAD).with_mem_access(Read),
		7 	=> empty_desc().with_bus_xfer(Regs::ACC, Regs::X),
		10 	=> empty_desc().with_bus_xfer(Regs::SIR, Regs::Y).with_alu_op(Xor),
		12 	=> empty_desc().with_bus_xfer(Regs::Z, Regs::ACC),
		_ 	=> empty_desc(),
	}
}

fn descriptor_load_value(microcycle: u8) -> Descriptor
{
	match microcycle
	{
		6 	=> empty_desc().with_masked_bus_xfer(Regs::IR, Regs::SAR, BusXfer::SOURCE_BITMASK_BASIC_PAYLOAD).with_mem_access(Read),
		10 	=> empty_desc().with_bus_xfer(Regs::SIR, Regs::ACC),
		_ 	=> empty_desc(),
	}
}

fn descriptor_store_value(microcycle: u8) -> Descriptor
{
	match microcycle
	{
		6 => empty_desc().with_masked_bus_xfer(Regs::IR, Regs::SAR, BusXfer::SOURCE_BITMASK_BASIC_PAYLOAD),
		7 => empty_desc().with_bus_xfer(Regs::ACC, Regs::SIR).with_mem_access(Write),
		_ => empty_desc(),
	}
}

fn descriptor_load_constant(microcycle: u8) -> Descriptor
{
	match microcycle
	{
		6 => empty_desc().with_masked_bus_xfer(Regs::IR, Regs::ACC, BusXfer::SOURCE_BITMASK_BASIC_PAYLOAD),
		_ => empty_desc(),
	}
}

fn descriptor_jump(microcycle: u8) -> Descriptor
{
	match microcycle
	{
		6 => empty_desc().with_masked_bus_xfer(Regs::IR, Regs::IAR, BusXfer::SOURCE_BITMASK_BASIC_PAYLOAD),
		_ => empty_desc(),
	}
}

fn descriptor_jump_if_negative(microcycle: u8) -> Descriptor
{
	match microcycle
	{
		6 => empty_desc().with_masked_bus_xfer(Regs::IR, Regs::IAR, BusXfer::SOURCE_BITMASK_BASIC_PAYLOAD).acc_dependent(),
		_ => empty_desc(),
	}
}

fn descriptor_equals(microcycle: u8) -> Descriptor
{
	match microcycle
	{
		6 	=> empty_desc().with_masked_bus_xfer(Regs::IR, Regs::SAR, BusXfer::SOURCE_BITMASK_BASIC_PAYLOAD).with_mem_access(Read),
		7 	=> empty_desc().with_bus_xfer(Regs::ACC, Regs::X),
		10 	=> empty_desc().with_bus_xfer(Regs::SIR, Regs::Y).with_alu_op(Equals),
		12 	=> empty_desc().with_bus_xfer(Regs::Z, Regs::ACC),
		_ 	=> empty_desc(),
	}
}

fn descriptor_halt(microcycle: u8) -> Descriptor
{
	match microcycle
	{
		_ => empty_desc(),
	}
}

fn descriptor_not(microcycle: u8) -> Descriptor
{
	match microcycle
	{
		6 	=> empty_desc().with_bus_xfer(Regs::ACC, Regs::X).with_alu_op(Not),
		8 	=> empty_desc().with_bus_xfer(Regs::Z, Regs::ACC),
		_ 	=> empty_desc(),
	}
}

fn descriptor_rotate_right(microcycle: u8) -> Descriptor
{
	match microcycle
	{
		6 	=> empty_desc().with_bus_xfer(Regs::ACC, Regs::X),
		7 	=> empty_desc().with_masked_bus_xfer(Regs::IR, Regs::Y, BusXfer::SOURCE_BITMASK_EXTENDED_PAYLOAD).with_alu_op(RotateRight),
		9 	=> empty_desc().with_bus_xfer(Regs::Z, Regs::ACC),
		_ 	=> empty_desc(),
	}
}

fn descriptor_no_operation(microcycle: u8) -> Descriptor
{
	match microcycle
	{
		_ => empty_desc(),
	}
}

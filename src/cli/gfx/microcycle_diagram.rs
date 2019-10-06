use std::io::{stdout, Write};
use mimasim::types::{*, Registers as Regs};
use mimasim::unit::{ALUOperation, MemoryAccess, MemoryType};
use crate::cli::term::{color, cursor, style, ui};
use crate::cli::record::{MicrocycleSummary, RegisterValue as RegValue, FlagValue};

// Okay, I am pretty sure this is the messiest part of the whole MiMA simulator ...
// We use Termion to draw an ASCII-art circuit diagram of the MiMA to an ANSI-aware terminal.

pub enum Model { }

// Measures:
const HEX_WIDTH: u16 = 2 + 8;

const MIMA_X: u16 = 1;
const MIMA_Y: u16 = 1;

const MIMA_WIDTH: u16 = 101;
const MIMA_HEIGHT: u16 = ARITH_HEIGHT + 2;

const REG_WIDTH: u16 = 1 + 1 + HEX_WIDTH + 1 + 1;
const REG_HEIGHT: u16 = 1 + 2 + 1;

const FLAG_WIDTH: u16 = 5;
const FLAG_HEIGHT: u16 = 3;

const ARITH_X: u16 = 3;
const ARITH_Y: u16 = 2;
const ARITH_WIDTH: u16 = 2 + ALU_WIDTH + 2;
const ARITH_HEIGHT: u16 = 2 + ALU_HEIGHT + 1 + REG_HEIGHT + 2;

const ALU_X: u16 = ARITH_X + 2;
const ALU_Y: u16 = ARITH_Y + 2 + REG_HEIGHT + 2;
const ALU_WIDTH: u16 = (2 * REG_WIDTH) + 5;
const ALU_HEIGHT: u16 = (2 * REG_HEIGHT) + 4 + ALU_CENTER_HEIGHT;
const ALU_CENTER_WIDTH: u16 = 7;
const ALU_CENTER_HEIGHT: u16 = 3;

const CONTROL_X: u16 = MIMA_WIDTH - CONTROL_WIDTH - 1;
const CONTROL_Y: u16 = 2;
const CONTROL_WIDTH: u16 = 2 + REG_WIDTH + 1 + REG_WIDTH + 2;
const CONTROL_HEIGHT: u16 = 1 + REG_HEIGHT + 1 + FLAG_HEIGHT + 1;

const MEMORY_X: u16 = MIMA_WIDTH - MEMORY_WIDTH - 1;
const MEMORY_Y: u16 = MIMA_HEIGHT - MEMORY_HEIGHT;
const MEMORY_WIDTH: u16 = 2 + REG_WIDTH + 16 + MEMORY_MEM_WIDTH + 1;
const MEMORY_HEIGHT: u16 = 1 + REG_HEIGHT + 1 + REG_HEIGHT + 1;
const MEMORY_MEM_X: u16 = MEMORY_X + 1 + REG_WIDTH + 15;
const MEMORY_MEM_Y: u16 = MEMORY_Y + 1;
const MEMORY_MEM_WIDTH: u16 = 9;
const MEMORY_MEM_HEIGHT: u16 = REG_HEIGHT + 1 + REG_HEIGHT;

const BUS_X: u16 = (MIMA_WIDTH - BUS_WIDTH) / 2;
const BUS_Y: u16 = 3;
const BUS_WIDTH: u16 = 9;
const BUS_HEIGHT: u16 = MIMA_HEIGHT - 5;

const IO_BUS_X: u16 = (MIMA_WIDTH - IO_BUS_WIDTH) / 2;
const IO_BUS_Y: u16 = MIMA_HEIGHT + 1;
const IO_BUS_WIDTH: u16 = MIMA_WIDTH - 20;
const IO_BUS_HEIGHT: u16 = 3;

// Which role does a register play in the microcycle's bus transfer?
enum RegisterBusXFerRole
{
	Source,
	Destination,
}

impl RegisterBusXFerRole
{
	fn from_summary(summary: &MicrocycleSummary, reg: Regs) -> Option<RegisterBusXFerRole>
	{
		if let Some(xfer) = summary.descriptor.bus_xfer.as_ref()
		{
			if xfer.source() == reg
			{
				Some(RegisterBusXFerRole::Source)
			}
			else if xfer.destinations().contains(reg)
			{
				Some(RegisterBusXFerRole::Destination)
			}
			else
			{
				None
			}
		}
		else
		{
			None
		}
	}
}

// How do we attach a given register to the bus?
// Default is a simple horizontal line.
// In some cases, we need to go vertical first.
// To calculate this stuff at least a little bit automated, these hints are used.
enum RegisterAttachment
{
	Horizontal,
	VerticalUp(u16),
	VerticalDown(u16),
}

impl Model
{
	pub fn draw_from_summary(summary: &MicrocycleSummary, x: u16, y: u16)
	{
		// Draw the outer MiMA box:
		ui::draw_named_box(x + MIMA_X, y + MIMA_Y, MIMA_WIDTH, MIMA_HEIGHT, color::LightBlack, "MiMA", color::White, true);

		// Draw the bus:
		Model::draw_bus(summary.is_bus_active(), x, y);

		// Draw the units:
		Model::draw_arithmetic_unit(summary, x, y);
		Model::draw_control_unit(summary, x, y);
		Model::draw_memory_unit(summary, x, y);

		// Reset colors and style.
		// Then move the cursor below the model.
		print!("{color_reset}{style_reset}{goto}",
			color_reset = color::Fg(color::Reset),
			style_reset = style::Reset,
			goto = cursor::Goto(1, y + MIMA_HEIGHT + IO_BUS_HEIGHT + 1));

		// Flush the output:
		stdout().flush().expect("Failed to flush terminal.");
	}

	fn draw_register(reg_x: u16, reg_y: u16, x: u16, name: &str, attachment: RegisterAttachment, value: RegValue, xfer_role: Option<RegisterBusXFerRole>, is_bus_active: bool)
	{
		// Draw a box around the register:
		ui::draw_named_box(reg_x, reg_y, REG_WIDTH, REG_HEIGHT, color::LightBlack, name, color::White, false);

		// Write the content:
		match value
		{
			RegValue::Stasis(v) =>
			{
				print!("{goto0}{fg_color0}0x{value:08X}{goto1}{fg_color1} ────────── ",
					goto0 = cursor::Goto(reg_x + 2, reg_y + 1),
					fg_color0 = color::Fg(color::White),
					value = v.0,
					goto1 = cursor::Goto(reg_x + 1, reg_y + 2),
					fg_color1 = color::Fg(color::LightBlack));
			},
			RegValue::Change(old_v, new_v) =>
			{
				print!("{goto0}{fg_color0}0x{new_value:08X}{goto1}{fg_color1}0x{old_value:08X}",
					goto0 = cursor::Goto(reg_x + 2, reg_y + 1),
					fg_color0 = color::Fg(color::Green),
					new_value = new_v.0,
					goto1 = cursor::Goto(reg_x + 2, reg_y + 2),
					fg_color1 = color::Fg(color::LightBlack),
					old_value = old_v.0);
			},
		}

		// Attach the register to the bus:
		match attachment
		{
			RegisterAttachment::Horizontal 				=> Model::draw_register_attachment_horizontal(reg_x, reg_y, x, xfer_role, is_bus_active),
			RegisterAttachment::VerticalUp(offset) 		=> Model::draw_register_attachment_vertical(reg_x, reg_y, x, xfer_role, true, offset, is_bus_active),
			RegisterAttachment::VerticalDown(offset) 	=> Model::draw_register_attachment_vertical(reg_x, reg_y, x, xfer_role, false, offset, is_bus_active),
		}
	}

	fn draw_register_attachment_horizontal(reg_x: u16, reg_y: u16, x: u16, xfer_role: Option<RegisterBusXFerRole>, is_bus_active: bool)
	{
		// Draw a simple horizontal connector line at the vertical center of the register.
		// Attach it to the bus-facing edge.
		let bus_x = x + BUS_X;

		let (start_x, end_x, reg_connector, reg_connector_x, bus_connector, bus_connector_x, start_char, end_char) = if reg_x <= bus_x
		{
			let (start_char, end_char) = match xfer_role
			{
				None 									=> ('─', '─'),
				Some(RegisterBusXFerRole::Source) 		=> ('─', '>'),
				Some(RegisterBusXFerRole::Destination) 	=> ('<', '─'),
			};

			(reg_x + REG_WIDTH, bus_x - 1, '├', reg_x + REG_WIDTH - 1, '╢', bus_x, start_char, end_char)
		}
		else
		{
			let (start_char, end_char) = match xfer_role
			{
				None 									=> ('─', '─'),
				Some(RegisterBusXFerRole::Source) 		=> ('<', '─'),
				Some(RegisterBusXFerRole::Destination) 	=> ('─', '>'),
			};

			(bus_x + BUS_WIDTH, reg_x - 1, '┤', reg_x, '╟', bus_x + BUS_WIDTH - 1, start_char, end_char)
		};

		let start_y = reg_y + (REG_HEIGHT / 2) - 1;

		// Draw the line and the connectors:
		ui::draw_perpendicular_line(start_x, start_y, end_x, ui::LineDirection::Horizontal, start_char, '─', end_char, if xfer_role.is_some() { color::Green } else { color::LightBlack });
		ui::draw_char(reg_connector, reg_connector_x, start_y, color::LightBlack);
		ui::draw_char(bus_connector, bus_connector_x, start_y, if is_bus_active { color::Green } else { color::LightBlack });
	}

	fn draw_register_attachment_vertical(reg_x: u16, reg_y: u16, x: u16, xfer_role: Option<RegisterBusXFerRole>, up: bool, offset: u16, is_bus_active: bool)
	{
		assert!(offset >= 2, "The offset (= length of the vertical attachment) must be at least 2 to include connector and turn characters.");

		// Determine all the parameters -.-
		let bus_x = x + BUS_X;

		let (vert_x, vert_start_y, vert_end_y, vert_start, vert_end,
				horz_start_x, horz_end_x, horz_y, horz_start, horz_end,
				reg_connector_y, reg_connector, bus_connector_x, bus_connector) = match (up, &xfer_role)
		{
			(true, None) 										=> if reg_x <= bus_x { (reg_x + 2, reg_y - offset, reg_y - 1, '┌', '│', reg_x + 3, bus_x - 1, reg_y - offset, '─', '─', reg_y, '┴', bus_x, '╢') }
																   else              { (reg_x + REG_WIDTH - 3, reg_y - offset, reg_y - 1, '┐', '│', bus_x + BUS_WIDTH, reg_x + REG_WIDTH - 4, reg_y - offset, '─', '─', reg_y, '┴', bus_x + BUS_WIDTH - 1, '╟') },
			(true, Some(RegisterBusXFerRole::Source)) 			=> if reg_x <= bus_x { (reg_x + 2, reg_y - offset, reg_y - 1, '┌', '│', reg_x + 3, bus_x - 1, reg_y - offset, '─', '>', reg_y, '┴', bus_x, '╢') }
																   else              { (reg_x + REG_WIDTH - 3, reg_y - offset, reg_y - 1, '┐', '│', bus_x + BUS_WIDTH, reg_x + REG_WIDTH - 4, reg_y - offset, '<', '─', reg_y, '┴', bus_x + BUS_WIDTH - 1, '╟') },
			(true, Some(RegisterBusXFerRole::Destination)) 		=> if reg_x <= bus_x { (reg_x + 2, reg_y - offset, reg_y - 1, '┌', 'V', reg_x + 3, bus_x - 1, reg_y - offset, '─', '─', reg_y, '┴', bus_x, '╢') }
																   else              { (reg_x + REG_WIDTH - 3, reg_y - offset, reg_y - 1, '┐', 'V', bus_x + BUS_WIDTH, reg_x + REG_WIDTH - 4, reg_y - offset, '─', '─', reg_y, '┴', bus_x + BUS_WIDTH - 1, '╟') },
			(false, None) 										=> if reg_x <= bus_x { (reg_x + 2, reg_y + REG_HEIGHT, reg_y + REG_HEIGHT + offset - 1, '│', '└', reg_x + 3, bus_x - 1, reg_y + REG_HEIGHT + offset - 1, '─', '─', reg_y + REG_HEIGHT - 1, '┬', bus_x, '╢') }
																   else              { (reg_x + REG_WIDTH - 3, reg_y + REG_HEIGHT, reg_y + REG_HEIGHT + offset - 1, '│', '┘', bus_x + BUS_WIDTH, reg_x + REG_WIDTH - 4, reg_y + REG_HEIGHT + offset - 1, '─', '─', reg_y + REG_HEIGHT - 1, '┬', bus_x + BUS_WIDTH - 1, '╟') },
			(false, Some(RegisterBusXFerRole::Source)) 			=> if reg_x <= bus_x { (reg_x + 2, reg_y + REG_HEIGHT, reg_y + REG_HEIGHT + offset - 1, '│', '└', reg_x + 3, bus_x - 1, reg_y + REG_HEIGHT + offset - 1, '─', '>', reg_y + REG_HEIGHT - 1, '┬', bus_x, '╢') }
																   else              { (reg_x + REG_WIDTH - 3, reg_y + REG_HEIGHT, reg_y + REG_HEIGHT + offset - 1, '│', '┘', bus_x + BUS_WIDTH, reg_x + REG_WIDTH - 4, reg_y + REG_HEIGHT + offset - 1, '<', '─', reg_y + REG_HEIGHT - 1, '┬', bus_x + BUS_WIDTH - 1, '╟') },
			(false, Some(RegisterBusXFerRole::Destination)) 	=> if reg_x <= bus_x { (reg_x + 2, reg_y + REG_HEIGHT, reg_y + REG_HEIGHT + offset - 1, '^', '└', reg_x + 3, bus_x - 1, reg_y + REG_HEIGHT + offset - 1, '─', '─', reg_y + REG_HEIGHT - 1, '┬', bus_x, '╢') }
																   else              { (reg_x + REG_WIDTH - 3, reg_y + REG_HEIGHT, reg_y + REG_HEIGHT + offset - 1, '^', '┘', bus_x + BUS_WIDTH, reg_x + REG_WIDTH - 4, reg_y + REG_HEIGHT + offset - 1, '─', '─', reg_y + REG_HEIGHT - 1, '┬', bus_x + BUS_WIDTH - 1, '╟') },
		};

		// Draw the lines and the connectors:
		let line_color = if xfer_role.is_some() { color::Green } else { color::LightBlack };

		ui::draw_perpendicular_line(vert_x, vert_start_y, vert_end_y, ui::LineDirection::Vertical, vert_start, '│', vert_end, line_color);
		ui::draw_perpendicular_line(horz_start_x, horz_y, horz_end_x, ui::LineDirection::Horizontal, horz_start, '─', horz_end, line_color);

		ui::draw_char(reg_connector, vert_x, reg_connector_y, color::LightBlack);
		ui::draw_char(bus_connector, bus_connector_x, horz_y, if is_bus_active { color::Green } else { color::LightBlack });
	}

	fn draw_flag(flag_x: u16, flag_y: u16, name: &str, value: FlagValue)
	{
		// Draw a box around the flag:
		ui::draw_named_box(flag_x, flag_y, FLAG_WIDTH, FLAG_HEIGHT, color::LightBlack, name, color::White, false);

		// Write the content:
		let (color, text) = match value
		{
			FlagValue::Stasis(v) => (color::White, if v.0 { '1' } else { '0' }),
			FlagValue::Change(_, new_v) => if new_v.0 { (color::Green, '1') } else{ (color::Red, '0') },
		};

		print!("{goto}{fg_color}{value}",
			goto = cursor::Goto(flag_x + 2, flag_y + 1),
			fg_color = color::Fg(color),
			value = text);
	}

	fn draw_bus(is_active: bool, x: u16, y: u16)
	{
		let bus_x = x + BUS_X;
		let bus_y = y + BUS_Y;

		// Draw the box:
		let box_color = if is_active { color::Green } else { color::LightBlack };
		ui::draw_box(bus_x, bus_y, BUS_WIDTH, BUS_HEIGHT, box_color, true);

		// Label it:
		ui::draw_char('B', bus_x + (BUS_WIDTH / 2), bus_y + (BUS_HEIGHT / 2) - 1, box_color);
		ui::draw_char('U', bus_x + (BUS_WIDTH / 2), bus_y + (BUS_HEIGHT / 2), box_color);
		ui::draw_char('S', bus_x + (BUS_WIDTH / 2), bus_y + (BUS_HEIGHT / 2) + 1, box_color);
	}

	fn draw_arithmetic_unit(summary: &MicrocycleSummary, x: u16, y: u16)
	{
		// Draw the outer box:
		ui::draw_named_box(x + ARITH_X, y + ARITH_Y, ARITH_WIDTH, ARITH_HEIGHT, color::LightYellow, "Arithmetic Unit", color::LightYellow, true);

		// Draw the non-ALU registers:
		Model::draw_register(x + ARITH_X + 2 + 2, y + ARITH_Y + 1, x, "ONE", RegisterAttachment::VerticalDown(2), RegValue::Stasis(Word(1)), RegisterBusXFerRole::from_summary(summary, Regs::ONE), summary.is_bus_active());
		Model::draw_register(x + ARITH_X + 2 + 2 + REG_WIDTH + 1, y + ARITH_Y + 1, x, "ACC", RegisterAttachment::Horizontal, summary.acc, RegisterBusXFerRole::from_summary(summary, Regs::ACC), summary.is_bus_active());

		// Draw the ALU:
		Model::draw_alu(summary, x, y);
	}

	fn draw_alu(summary: &MicrocycleSummary, x: u16, y: u16)
	{
		// Draw the outer box around the ALU:
		ui::draw_named_box(x + ALU_X, y + ALU_Y, ALU_WIDTH, ALU_HEIGHT, color::LightYellow, "ALU", color::LightYellow, false);

		// Draw the registers X, Y and Z:
		let reg_x_x = x + ALU_X + 2;
		let reg_y_x = x + ALU_X + 2 + REG_WIDTH + 1;
		let reg_xy_y = y + ALU_Y + 1;
		let reg_z_x = x + ALU_X + ((ALU_WIDTH - REG_WIDTH) / 2);
		let reg_z_y = y + ALU_Y + 1 + REG_HEIGHT + 1 + ALU_CENTER_HEIGHT + 1;

		Model::draw_register(reg_x_x, reg_xy_y, x, "X", RegisterAttachment::VerticalUp(2), summary.x, RegisterBusXFerRole::from_summary(summary, Regs::X), summary.is_bus_active());
		Model::draw_register(reg_y_x, reg_xy_y, x, "Y", RegisterAttachment::Horizontal, summary.y, RegisterBusXFerRole::from_summary(summary, Regs::Y), summary.is_bus_active());
		Model::draw_register(reg_z_x, reg_z_y, x, "Z", RegisterAttachment::Horizontal, summary.z, RegisterBusXFerRole::from_summary(summary, Regs::Z), summary.is_bus_active());

		// Pre-calculate some positions:
		let center_x = x + ALU_X + ((ALU_WIDTH - ALU_CENTER_WIDTH) / 2);
		let center_y = y + ALU_Y + 1 + REG_HEIGHT + 1;

		let op_x = center_x + (ALU_CENTER_WIDTH / 2);
		let op_y = center_y + (ALU_CENTER_HEIGHT / 2);

		let signal_x_start = center_x - 7;
		let signal_x_end = center_x - 1;
		let signal_y = op_y;

		let reg_x_connector_x = reg_x_x + REG_WIDTH - 2;
		let reg_y_connector_x = reg_y_x + 1;
		let reg_xy_connector_y = y + ALU_Y + REG_HEIGHT;
		let reg_z_connector_x = op_x;
		let reg_z_connector_y = reg_z_y;

		// Draw the center and the attachment to Z.
		// Color, operation char and attachment end depend on the ALU work.
		let select_alu_op_char = |op| match op
		{
			ALUOperation::Add 			=> '+',
			ALUOperation::And 			=> '&',
			ALUOperation::Or 			=> '|',
			ALUOperation::Xor 			=> '^',
			ALUOperation::Equals 		=> '=',
			ALUOperation::Not 			=> '!',
			ALUOperation::RotateRight 	=> 'R',
		};

		let (alu_color, op_center, attachment_end_char) = if let Some((op, rem)) = summary.alu_work
		{
			let op_char = select_alu_op_char(op);
			if rem == 0 { (color::Green, Some((op_char, color::Green)), 'V') } else { (color::LightBlack, Some((op_char, color::Yellow)), '│') }
		}
		else
		{
			(color::LightBlack, None, '│')
		};

		// Box:
		ui::draw_box(center_x, center_y, ALU_CENTER_WIDTH, ALU_CENTER_HEIGHT, alu_color, false);

		if let Some((op_char, op_char_color)) = op_center
		{
			print!("{goto}{fg_color}{style}{op}{reset}",
				goto = cursor::Goto(op_x, op_y),
				fg_color = color::Fg(op_char_color),
				style = style::Bold,
				op = op_char,
				reset = style::Reset);
		}

		// Center -> Z attachment:
		ui::draw_perpendicular_line(reg_z_connector_x, center_y + ALU_CENTER_HEIGHT - 1, reg_z_connector_y - 1, ui::LineDirection::Vertical, '┬', '│', attachment_end_char, alu_color);

		// Connectors at the center to X and Y:
		ui::draw_char('┴', reg_x_connector_x, center_y, alu_color);
		ui::draw_char('┴', reg_y_connector_x, center_y, alu_color);

		// Draw the register connectors:
		ui::draw_char('┬', reg_x_connector_x, reg_xy_connector_y, color::LightBlack);
		ui::draw_char('┬', reg_y_connector_x, reg_xy_connector_y, color::LightBlack);
		ui::draw_char('┴', reg_z_connector_x, reg_z_connector_y, color::LightBlack);

		// Draw the ALU signal if there is one:
		if let Some(op) = summary.descriptor.alu_op
		{
			print!("{goto}{fg_color}{style}{op}{reset}",
				goto = cursor::Goto(signal_x_start, signal_y),
				fg_color = color::Fg(color::Green),
				style = style::Bold,
				op = select_alu_op_char(op),
				reset = style::Reset);

			ui::draw_perpendicular_line(signal_x_start + 2, signal_y, signal_x_end, ui::LineDirection::Horizontal, '├', '─', '>', color::Green);

			// (X, Y) -> Center attachment:
			ui::draw_perpendicular_line(reg_x_connector_x, reg_xy_connector_y + 1, center_y - 1, ui::LineDirection::Vertical, '│', '│', 'V', color::Green);
			ui::draw_perpendicular_line(reg_y_connector_x, reg_xy_connector_y + 1, center_y - 1, ui::LineDirection::Vertical, '│', '│', 'V', color::Green);
		}
		else
		{
			// (X, Y) -> Center attachment:
			ui::draw_perpendicular_line(reg_x_connector_x, reg_xy_connector_y + 1, center_y - 1, ui::LineDirection::Vertical, '│', '│', '│', color::LightBlack);
			ui::draw_perpendicular_line(reg_y_connector_x, reg_xy_connector_y + 1, center_y - 1, ui::LineDirection::Vertical, '│', '│', '│', color::LightBlack);
		}
	}

	fn draw_control_unit(summary: &MicrocycleSummary, x: u16, y: u16)
	{
		// Draw the outer box:
		ui::draw_named_box(x + CONTROL_X, y + CONTROL_Y, CONTROL_WIDTH, CONTROL_HEIGHT, color::Blue, "Control Unit", color::Blue, true);

		// Draw the registers:
		Model::draw_register(x + CONTROL_X + 2, y + CONTROL_Y + 1, x, "IAR", RegisterAttachment::Horizontal, summary.iar, RegisterBusXFerRole::from_summary(summary, Regs::IAR), summary.is_bus_active());
		Model::draw_register(x + CONTROL_X + 2 + REG_WIDTH + 1, y + CONTROL_Y + 1, x, "IR", RegisterAttachment::VerticalDown(4), summary.ir, RegisterBusXFerRole::from_summary(summary, Regs::IR), summary.is_bus_active());

		// Draw the flags:
		Model::draw_flag(x + CONTROL_X + 2, y + CONTROL_Y + REG_HEIGHT + 1, "RUN", summary.run);
		Model::draw_flag(x + CONTROL_X + 2 + FLAG_WIDTH, y + CONTROL_Y + REG_HEIGHT + 1, "TRA", summary.tra);

		// Draw the cycle:
		let cycle_x = x + CONTROL_X + 2 + FLAG_WIDTH + FLAG_WIDTH + 1;
		let cycle_y = y + CONTROL_Y + REG_HEIGHT + 1;

		ui::draw_named_box(cycle_x, cycle_y, 6, 3, color::LightBlack, "CYCL", color::White, false);

		print!("{goto}{fg_color}{cycle}",
			goto = cursor::Goto(cycle_x + 2, cycle_y + 1),
			fg_color = color::Fg(color::White),
			cycle = format!("{:02}", summary.microcycle));

		// Draw the command:
		let cmd_x = cycle_x + 7;
		let cmd_y = y + CONTROL_Y + REG_HEIGHT + 1;

		ui::draw_named_box(cmd_x, cmd_y, 7, 3, color::LightBlack, "INS", color::White, false);

		print!("{goto}{fg_color}{instr}",
			goto = cursor::Goto(cmd_x + 2, cmd_y + 1),
			fg_color = color::Fg(color::White),
			instr = summary.instruction.map_or("───", |i| i.format_opcode()));
	}

	fn draw_memory_unit(summary: &MicrocycleSummary, x: u16, y: u16)
	{
		// Draw the outer box:
		ui::draw_named_box(x + MEMORY_X, y + MEMORY_Y, MEMORY_WIDTH, MEMORY_HEIGHT, color::Red, "Memory Unit", color::Red, true);

		// Draw the registers:
		let reg_sir_x = x + MEMORY_X + 2;
		let reg_sir_y = y + MEMORY_Y + 1 + REG_HEIGHT + 1;
		let reg_sar_x = reg_sir_x + 7;
		let reg_sar_y = y + MEMORY_Y + 1;

		Model::draw_register(reg_sar_x, reg_sar_y, x, "SAR", RegisterAttachment::Horizontal, summary.sar, RegisterBusXFerRole::from_summary(summary, Regs::SAR), summary.is_bus_active());
		Model::draw_register(reg_sir_x, reg_sir_y, x, "SIR", RegisterAttachment::Horizontal, summary.sir, RegisterBusXFerRole::from_summary(summary, Regs::SIR), summary.is_bus_active());

		// Do we export from SAR and / or SIR?
		let (is_sar_lin_active, sar_lin_end, is_sar_io_active, sar_io_end,
				is_sir_lin_active, sir_lin_end, is_sir_io_active, sir_io_end,
				lin_mem_access, io_mem_access) = if let Some(mem_access) = summary.descriptor.mem_access
		{
			// Determine the memory type from the address that is accessed:
			let mem_type = MemoryType::from_address(summary.sar.final_value());

			match (mem_type, mem_access)
			{
				(MemoryType::Linear, MemoryAccess::Read) 		=> (true, '>', false, '│', false, '─', false, '│', Some(MemoryAccess::Read), None),
				(MemoryType::DeviceIO, MemoryAccess::Read) 		=> (false, '─', true, 'V', false, '─', false, '│', None, Some(MemoryAccess::Read)),
				(MemoryType::Linear, MemoryAccess::Write) 		=> (true, '>', false, '│', true, '>', false, '│', Some(MemoryAccess::Write), None),
				(MemoryType::DeviceIO, MemoryAccess::Write) 	=> (false, '─', true, 'V', false, '─', true, 'V', None, Some(MemoryAccess::Write)),
			}
		}
		else
		{
			(false, '─', false, '│', false, '─', false, '│', None, None)
		};

		// Do we import into SIR?
		let (is_lin_sir_active, sir_lin_start, is_io_sir_active, sir_io_start, lin_op, io_op) = match summary.mem_work
		{
			Some((MemoryType::Linear, MemoryAccess::Read, 0)) 		=> (true, '<', false, '│', Some('R'), None),
			Some((MemoryType::DeviceIO, MemoryAccess::Read, 0)) 	=> (false, '─', true, '^', None, Some('R')),
			Some((MemoryType::Linear, MemoryAccess::Read, _))		=> (false, '─', false, '│', Some('R'), None),
			Some((MemoryType::DeviceIO, MemoryAccess::Read, _))		=> (false, '─', false, '│', None, Some('R')),
			Some((MemoryType::Linear, MemoryAccess::Write, _))		=> (false, '─', false, '│', Some('W'), None),
			Some((MemoryType::DeviceIO, MemoryAccess::Write, _))	=> (false, '─', false, '│', None, Some('W')),
			_ 														=> (false, '─', false, '│', None, None),
		};

		// Measures:
		let mem_x = x + MEMORY_MEM_X;
		let mem_y = y + MEMORY_MEM_Y;

		let io_x = x + IO_BUS_X;
		let io_y = y + IO_BUS_Y;

		let sar_lin_connector_start_x = reg_sar_x + REG_WIDTH - 1;
		let sar_lin_connector_end_x = mem_x;
		let sar_lin_connector_y = reg_sar_y + (REG_HEIGHT / 2) - 1;

		let sir_lin_connector_start_x = reg_sir_x + REG_WIDTH - 1;
		let sir_lin_connector_end_x = mem_x;
		let sir_lin_connector_y = reg_sir_y + (REG_HEIGHT / 2) - 1;

		let sar_io_connector_x = reg_sar_x + REG_WIDTH - 4;
		let sar_io_connector_start_y = reg_sar_y + REG_HEIGHT - 1;
		let sar_io_connector_end_y = io_y;

		let sir_io_connector_x = reg_sir_x + (REG_WIDTH / 2);
		let sir_io_connector_start_y = reg_sir_y + REG_HEIGHT - 1;
		let sir_io_connector_end_y = io_y;

		let lin_op_x = mem_x + (MEMORY_MEM_WIDTH / 2);
		let lin_op_y = mem_y + (MEMORY_MEM_HEIGHT / 2);

		let io_op_x = io_x + IO_BUS_WIDTH - 3;
		let io_op_y = io_y + (IO_BUS_HEIGHT / 2);

		// Attach SIR and SAR to the linear memory:
		ui::draw_perpendicular_line(sar_lin_connector_start_x + 1, sar_lin_connector_y, sar_lin_connector_end_x - 1, ui::LineDirection::Horizontal, '─', '─', sar_lin_end, if is_sar_lin_active { color::Green } else { color::LightBlack });
		ui::draw_perpendicular_line(sir_lin_connector_start_x + 1, sir_lin_connector_y, sir_lin_connector_end_x - 1, ui::LineDirection::Horizontal, sir_lin_start, '─', sir_lin_end, if is_sir_lin_active || is_lin_sir_active { color::Green } else { color::LightBlack });

		// Attach SAR and SIR to the I/O bus:
		ui::draw_perpendicular_line(sar_io_connector_x, sar_io_connector_start_y + 1, sar_io_connector_end_y - 1, ui::LineDirection::Vertical, '│', '│', sar_io_end, if is_sar_io_active { color::Green } else { color::LightBlack });
		ui::draw_perpendicular_line(sir_io_connector_x, sir_io_connector_start_y + 1, sir_io_connector_end_y - 1, ui::LineDirection::Vertical, sir_io_start, '│', sir_io_end, if is_sir_io_active { color::Green } else { color::LightBlack });

		// Draw the connectors at the registers:
		ui::draw_char('├', sar_lin_connector_start_x, sar_lin_connector_y, color::LightBlack);
		ui::draw_char('├', sir_lin_connector_start_x, sir_lin_connector_y, color::LightBlack);
		ui::draw_char('┬', sar_io_connector_x, sar_io_connector_start_y, color::LightBlack);
		ui::draw_char('┬', sir_io_connector_x, sir_io_connector_start_y, color::LightBlack);

		// Draw the linear memory with connectors and signal:
		let (lin_color, lin_name_color, lin_op_color) = if is_lin_sir_active { (color::Green, color::Green, color::Green) } else { (color::LightBlack, color::White, color::Yellow) };

		ui::draw_named_box(mem_x, mem_y, MEMORY_MEM_WIDTH, MEMORY_MEM_HEIGHT, lin_color, "MEM", lin_name_color, false);
		ui::draw_char('┤', sar_lin_connector_end_x, sar_lin_connector_y, lin_color);
		ui::draw_char('┤', sir_lin_connector_end_x, sir_lin_connector_y, lin_color);

		if let Some(lin_op) = lin_op
		{
			print!("{goto}{fg_color}{style}{op}{reset}",
				goto = cursor::Goto(lin_op_x, lin_op_y),
				fg_color = color::Fg(lin_op_color),
				style = style::Bold,
				op = lin_op,
				reset = style::Reset);
		}

		if let Some(access) = lin_mem_access
		{
			let signal_x_start = mem_x - 7;
			let signal_x_end = mem_x - 1;
			let signal_y = mem_y + (MEMORY_MEM_HEIGHT / 2);

			print!("{goto}{fg_color}{style}{signal}{reset}",
				goto = cursor::Goto(signal_x_start, signal_y),
				fg_color = color::Fg(color::Green),
				style = style::Bold,
				signal = match access
				{
					MemoryAccess::Read 		=> 'R',
					MemoryAccess::Write 	=> 'W',
				},
				reset = style::Reset);

			ui::draw_perpendicular_line(signal_x_start + 2, signal_y, signal_x_end, ui::LineDirection::Horizontal, '├', '─', '>', color::Green);
		}

		// Draw the IO memory with connectors and signal (yeah, technically, that one is located outside of the memory unit ...):
		let (io_color, io_op_color) = if is_io_sir_active { (color::Green, color::Green) } else { (color::LightBlack, color::Yellow) };

		ui::draw_box(io_x, io_y, IO_BUS_WIDTH, IO_BUS_HEIGHT, io_color, true);
		ui::draw_char('╧', sar_io_connector_x, sar_io_connector_end_y, io_color);
		ui::draw_char('╧', sir_io_connector_x, sir_io_connector_end_y, io_color);

		ui::draw_char('I', io_x + (IO_BUS_WIDTH / 2) - 1, io_y + (IO_BUS_HEIGHT / 2), io_color);
		ui::draw_char('/', io_x + (IO_BUS_WIDTH / 2), io_y + (IO_BUS_HEIGHT / 2), io_color);
		ui::draw_char('O', io_x + (IO_BUS_WIDTH / 2) + 1, io_y + (IO_BUS_HEIGHT / 2), io_color);

		if let Some(io_op) = io_op
		{
			print!("{goto}{fg_color}{style}{op}{reset}",
				goto = cursor::Goto(io_op_x, io_op_y),
				fg_color = color::Fg(io_op_color),
				style = style::Bold,
				op = io_op,
				reset = style::Reset);
		}

		if let Some(access) = io_mem_access
		{
			let signal_x_start = io_x + IO_BUS_WIDTH;
			let signal_x_end = signal_x_start + 4;
			let signal_y = io_y + (IO_BUS_HEIGHT / 2);

			print!("{goto}{fg_color}{style}{signal}{reset}",
				goto = cursor::Goto(signal_x_end + 2, signal_y),
				fg_color = color::Fg(color::Green),
				style = style::Bold,
				signal = match access
				{
					MemoryAccess::Read 		=> 'R',
					MemoryAccess::Write 	=> 'W',
				},
				reset = style::Reset);

			ui::draw_perpendicular_line(signal_x_start, signal_y, signal_x_end, ui::LineDirection::Horizontal, '<', '─', '┤', color::Green);
		}
	}
}

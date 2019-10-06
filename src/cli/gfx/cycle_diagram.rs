use std::io::{stdout, Write};
use mimasim::types::{*, Registers as Regs};
use crate::cli::term::{color, cursor, ui};
use crate::cli::record::{CycleSummary, RegisterValue as RegValue, FlagValue};

pub enum Model { }

// Measures:
const HEX_WIDTH: u16 = 2 + 8;

const REG_WIDTH: u16 = 1 + 1 + HEX_WIDTH + 1 + 1;
const REG_HEIGHT: u16 = 1 + 2 + 1;

const FLAG_WIDTH: u16 = 5;
const FLAG_HEIGHT: u16 = 3;

const ACC_X: u16 = 0;
const ACC_Y: u16 = 0;

const IAR_X: u16 = ACC_X + REG_WIDTH;
const IAR_Y: u16 = ACC_Y;

const RUN_X: u16 = IAR_X + REG_WIDTH;
const RUN_Y: u16 = IAR_Y;

const TRA_X: u16 = RUN_X + FLAG_WIDTH;
const TRA_Y: u16 = RUN_Y;

impl Model
{
	pub fn draw_from_summary(summary: &CycleSummary, x: u16, y: u16)
	{
		// Draw accumulator and flags as named boxes:
		Model::draw_register(x + ACC_X, y + ACC_Y, "ACC", summary.acc);
		Model::draw_flag(x + RUN_X, y + RUN_Y, "RUN", summary.run);
		Model::draw_flag(x + TRA_X, y + TRA_Y, "TRA", summary.tra);
		Model::draw_register(x + IAR_X, y + IAR_Y, "IAR", summary.iar);

		// Flush the output:
		stdout().flush().expect("Failed to flush terminal.");
	}

	fn draw_register(reg_x: u16, reg_y: u16, name: &str, value: RegValue)
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
	}

	fn draw_flag(x: u16, y: u16, name: &str, value: FlagValue)
	{
		// Draw a box around the flag:
		ui::draw_named_box(x, y, FLAG_WIDTH, FLAG_HEIGHT, color::LightBlack, name, color::White, false);

		// Write the content:
		let (color, text) = match value
		{
			FlagValue::Stasis(v) => (color::White, if v.0 { '1' } else { '0' }),
			FlagValue::Change(_, new_v) => if new_v.0 { (color::Green, '1') } else{ (color::Red, '0') },
		};

		print!("{goto}{fg_color}{value}",
			goto = cursor::Goto(x + 2, y + 1),
			fg_color = color::Fg(color),
			value = text);
	}
}

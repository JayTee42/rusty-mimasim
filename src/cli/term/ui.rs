use termion::cursor;
use crate::cli::term::color;

// How to draw a perpendicular line?
pub enum LineDirection
{
	Horizontal,
	Vertical,
}

pub fn draw_char(c: char, x: u16, y: u16, color: color::Color)
{
	print!("{color}{goto}{chr}",
		color = color::Fg(color),
		goto = cursor::Goto(x, y),
		chr = c);
}

pub fn draw_perpendicular_line(start_x: u16, start_y: u16, end_xy: u16, dir: LineDirection, start: char, inner: char, end: char, color: color::Color)
{
	match dir
	{
		LineDirection::Horizontal =>
		{
			draw_char(start, start_x, start_y, color);

			for _ in (start_x + 1)..end_xy
			{
				print!("{:}", inner);
			}

			print!("{:}", end);
		},
		LineDirection::Vertical =>
		{
			draw_char(start, start_x, start_y, color);

			// We always need to position the cursor here!
			for y in (start_y + 1)..end_xy
			{
				print!("{goto}{inner_char}",
					goto = cursor::Goto(start_x, y),
					inner_char = inner);
			}

			print!("{goto}{end_char}",
				goto = cursor::Goto(start_x, end_xy),
				end_char = end);
		},
	}
}

pub fn draw_box(x: u16, y: u16, width: u16, height: u16, color: color::Color, thick: bool)
{
	// Select the charset:
	let (lower_left, lower_right, upper_left, upper_right, horz_inner, vert_inner) = if thick
	{
		('╚', '╝', '╔', '╗', '═', '║')
	}
	else
	{
		('└', '┘', '┌', '┐', '─', '│')
	};

	// Draw four lines.
	// The horizontal lines contain the corner characters.
	draw_perpendicular_line(x, y, x + width - 1, LineDirection::Horizontal, upper_left, horz_inner, upper_right, color);
	draw_perpendicular_line(x, y + height - 1, x + width - 1, LineDirection::Horizontal, lower_left, horz_inner, lower_right, color);
	draw_perpendicular_line(x, y + 1, y + height - 2, LineDirection::Vertical, vert_inner, vert_inner, vert_inner, color);
	draw_perpendicular_line(x + width - 1, y + 1, y + height - 2, LineDirection::Vertical, vert_inner, vert_inner, vert_inner, color);
}

pub fn draw_named_box(x: u16, y: u16, width: u16, height: u16, border_color: color::Color, name: &str, name_color: color::Color, thick: bool)
{
	// Draw the box itself:
	draw_box(x, y, width, height, border_color, thick);

	// Write the box name to the top:
	let name_x = x + (width - (name.len() as u16)) / 2;

	print!("{goto}{name_color}{name}",
		goto = cursor::Goto(name_x, y),
		name_color = color::Fg(name_color),
		name = name);
}

use std::fmt;
use termion::color;

// A color enum for Termion.
// color::Color itself is a trait, therefore, color::Blue, color::Green and friends all have different types.
// To avoid trait objects, we define our own enum type that invokes the trait methods of the colors.
// With this enum type, we can use colors as result of if-conditions, store them in constants, ...
// The dead code warning suppression is necessary because we don't use all the colors, but might need them in the future.

// Allow to use this instead of termion::color:
pub use color::{Bg, Fg, Reset};

// Use all the color variants so we can e. g. type "color::Green":
pub use Color::*;

#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
pub enum Color
{
	LightBlack, Black,
	LightBlue, Blue,
	LightCyan, Cyan,
	LightGreen, Green,
	LightMagenta, Magenta,
	LightRed, Red,
	LightWhite, White,
	LightYellow, Yellow,
}

impl color::Color for Color
{
	// Background:
    fn write_bg(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
    	match self
    	{
			Color::LightBlack 	=> color::LightBlack.write_bg(f),
			Color::Black 		=> color::Black.write_bg(f),
			Color::LightBlue 	=> color::LightBlue.write_bg(f),
			Color::Blue 		=> color::Blue.write_bg(f),
			Color::LightCyan 	=> color::LightCyan.write_bg(f),
			Color::Cyan 		=> color::Cyan.write_bg(f),
			Color::LightGreen 	=> color::LightGreen.write_bg(f),
			Color::Green 		=> color::Green.write_bg(f),
			Color::LightMagenta => color::LightMagenta.write_bg(f),
			Color::Magenta 		=> color::Magenta.write_bg(f),
			Color::LightRed 	=> color::LightRed.write_bg(f),
			Color::Red 			=> color::Red.write_bg(f),
			Color::LightWhite 	=> color::LightWhite.write_bg(f),
			Color::White 		=> color::White.write_bg(f),
			Color::LightYellow 	=> color::LightYellow.write_bg(f),
			Color::Yellow 		=> color::Yellow.write_bg(f),
    	}
    }

    // Foreground:
    fn write_fg(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
    	match self
    	{
			Color::LightBlack 	=> color::LightBlack.write_fg(f),
			Color::Black 		=> color::Black.write_fg(f),
			Color::LightBlue 	=> color::LightBlue.write_fg(f),
			Color::Blue 		=> color::Blue.write_fg(f),
			Color::LightCyan 	=> color::LightCyan.write_fg(f),
			Color::Cyan 		=> color::Cyan.write_fg(f),
			Color::LightGreen 	=> color::LightGreen.write_fg(f),
			Color::Green 		=> color::Green.write_fg(f),
			Color::LightMagenta => color::LightMagenta.write_fg(f),
			Color::Magenta 		=> color::Magenta.write_fg(f),
			Color::LightRed 	=> color::LightRed.write_fg(f),
			Color::Red 			=> color::Red.write_fg(f),
			Color::LightWhite 	=> color::LightWhite.write_fg(f),
			Color::White 		=> color::White.write_fg(f),
			Color::LightYellow 	=> color::LightYellow.write_fg(f),
			Color::Yellow 		=> color::Yellow.write_fg(f),
    	}
    }
}

// This is a wrapper module around termion that adds some own creations / simplifications.

// Our own modules:
pub mod color;
pub mod ui;

// Import the other termion modules we need here, too.
// This allows us to completely elide termion module uses.
pub use termion::{clear, cursor, style};

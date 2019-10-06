use crate::types::{*, Registers as Regs};

// Sources and destinations for bus transfers.
// Those are always 1:n (i. e. "Transfer the word in IAR to SAR and to X.").
// Not all registers are allowed as source and destination.
// The Transfer type checks those constraints.
pub use Xfer as BusXfer;

// A bus transfer holds a source and 1...n destinations:
pub struct Xfer
{
	source: Regs,
	destinations: Regs,

	// Some bus transfers require to apply a source bitmask.
	// This mask defaults to 0xFFFFFFFFu32, but can be changed if needed.
	// We always propagate (source & bitmask) on the bus.
	// This is restricted by the MiMA's hardware capabilities (see "validate_source_bitmask").
	source_bitmask: Word,

	// A bus transfer can be executed "accumulator-dependent".
	// This is specifically important for the JMN instruction and translates to:
	// "Execute the transfer if and only if ACC < 0."
	// By default, bus transfers are not accumulator-dependent.
	// Call "make_acc_dependent()" on them to change that.
	is_acc_dependent: bool
}

impl Xfer
{
	// Potential source bitmasks:
	pub(crate) const SOURCE_BITMASK_FULL: Word = Word(0xFF_FF_FF_FFu32);
	pub(crate) const SOURCE_BITMASK_BASIC_PAYLOAD: Word = Word(0x0F_FF_FF_FFu32);
	pub(crate) const SOURCE_BITMASK_EXTENDED_PAYLOAD: Word = Word(0x00_FF_FF_FFu32);

	pub(crate) fn new(source: Regs, destinations: Regs, source_bitmask: Word) -> Xfer
	{
		// Validate counts:
		let source_count = Regs::ALL_REGISTERS.iter().filter(|&&curr_source| source.contains(curr_source)).count();

		assert!(source_count == 1, "Bus source registers must contain exactly one register.");
		assert!(!destinations.is_empty(), "Bus destination registers must not be empty.");

		// Validate registers themselves:
		let valid_source_regs 		= Regs::ACC | Regs::ONE | Regs::Z | Regs::IAR | Regs::IR | Regs::SIR;
		let valid_destination_regs 	= Regs::ACC | Regs::X   | Regs::Y | Regs::IAR | Regs::IR | Regs::SAR | Regs::SIR;

		assert!(valid_source_regs.contains(source), "Invalid bus source registers: {}", source);
		assert!(valid_destination_regs.contains(destinations), "Invalid bus destination registers: {}", destinations);

		// Validate source bitmask:
		assert!(Xfer::validate_source_bitmask(source, source_bitmask), "Invalid source bitmask: {:08X} for {}", source_bitmask.0, source);

		Xfer
		{
			source,
			destinations,
			source_bitmask,
			is_acc_dependent: false,
		}
	}

	pub fn source(&self) -> Regs
	{
		self.source
	}

	pub fn destinations(&self) -> Regs
	{
		self.destinations
	}

	pub fn source_bitmask(&self) -> Word
	{
		self.source_bitmask
	}

	pub fn is_acc_dependent(&self) -> bool
	{
		self.is_acc_dependent
	}

	pub(crate) fn make_acc_dependent(&mut self)
	{
		self.is_acc_dependent = true;
	}

	fn validate_source_bitmask(source: Regs, source_bitmask: Word) -> bool
	{
		match source
		{
			Regs::IR 	=> (source_bitmask == Xfer::SOURCE_BITMASK_BASIC_PAYLOAD) || (source_bitmask == Xfer::SOURCE_BITMASK_EXTENDED_PAYLOAD),
			_ 			=> (source_bitmask == Xfer::SOURCE_BITMASK_FULL),
		}
	}
}

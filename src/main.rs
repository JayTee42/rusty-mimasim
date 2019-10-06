mod cli;

use std::time::Duration;
use std::thread;
use mimasim::{assembly::ObjectCode, mima::Mima};
use crate::cli::{gfx::{CycleDiagram, MicrocycleDiagram}, record::{CycleSummary, MicrocycleSummary}, term::clear};

fn main()
{
	let (object_code, _) = ObjectCode::assemble("

		jmp loop

		last: DAT 0 # starts undefined
		curr: DAT 0
		next: DAT 1

		count: DAT 6 # starts undefined
		decr: DAT -1

		# Read count
		# TODO

		loop:

		# Check if done
		LDV count
		ADD decr
		JMN out
		STV count

		# Print curr
		LDV curr
		# TODO

		# curr -> last
		STV last

		# next -> curr
		LDV next
		STV curr

		# last + curr -> next
		ADD last
		STV next

		# Next iteration
		JMP loop

		# End of program
		out:
		HLT

	").unwrap();

	let mut mima = Mima::new();
	mima.memory_unit.load_code(&object_code).unwrap();

	let mut start_summary = None;

	while let Some(microcycle_summary) = MicrocycleSummary::record_microcycle(&mut mima)
	{
		println!("{clear}", clear = clear::All);

		MicrocycleDiagram::draw_from_summary(&microcycle_summary, 1, 4);

		if microcycle_summary.microcycle == 1
		{
			start_summary = Some(microcycle_summary);
		}
		else if microcycle_summary.microcycle == 12
		{
			let cycle_summary = CycleSummary::from_microcycle_summaries(start_summary.as_ref().unwrap(), &microcycle_summary);
			CycleDiagram::draw_from_summary(&cycle_summary, 1, 1);
		}

		thread::sleep(Duration::from_millis(500));
	}
}

use anyhow::{Context, Result, anyhow};
use std::{env::args, fs::File, io::Read};

use crate::{io::parse_xml, simul::Simulation};

mod io;
mod simul;

fn main() -> Result<()> {
	let arg = args()
		.nth(1)
		.ok_or(anyhow!("Please specify the filename"))?;
	let file = File::open(arg).context("Error reading file")?;
	let mut decompressed = String::new();
	flate2::read::DeflateDecoder::new(file)
		.read_to_string(&mut decompressed)
		.context("Error decompressing file")?;
	let parsed = parse_xml(&decompressed)?;
	println!("{parsed:?}");
	let mut simul: Simulation = parsed.into();

	let mut inputs = simul.get_inputs_mut();
	**inputs.get_mut("x").unwrap() = false;
	**inputs.get_mut("y").unwrap() = false;
	simul.update_until_done(100);
	simul.print_outputs();

	let mut inputs = simul.get_inputs_mut();
	**inputs.get_mut("x").unwrap() = true;
	**inputs.get_mut("y").unwrap() = false;
	simul.update_until_done(100);
	simul.print_outputs();

	let mut inputs = simul.get_inputs_mut();
	**inputs.get_mut("x").unwrap() = false;
	**inputs.get_mut("y").unwrap() = true;
	simul.update_until_done(100);
	simul.print_outputs();

	let mut inputs = simul.get_inputs_mut();
	**inputs.get_mut("x").unwrap() = true;
	**inputs.get_mut("y").unwrap() = true;
	simul.update_until_done(100);
	simul.print_outputs();
	
	Ok(())
}

#![allow(dead_code)]
use anyhow::{Context, Result, anyhow};
use std::{env::args, fs::File, io::Read};

use crate::{io::parse_xml, simul::Simulation};

mod io;
mod simul;
mod util;

fn main() -> Result<()> {
	let arg = args()
		.nth(1)
		.ok_or(anyhow!("Please specify the filename"))?;
	let file = File::open(arg).context("Error reading file")?;
	let mut decompressed = String::new();
	flate2::read::DeflateDecoder::new(file)
		.read_to_string(&mut decompressed)
		.context("Error decompressing file")?;
	println!("{}", decompressed);
	let parsed = parse_xml(&decompressed)?;
	// println!("{parsed}");
	let mut simul: Simulation = parsed.into();
	// println!("{simul}");

	// simul.get_outputs(HashMap::from_iter([("x", false), ("y", false)].into_iter()), 100);
	simul.print_truth_table(1000);

	Ok(())
}

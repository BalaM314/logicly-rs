use anyhow::{Context, Result, anyhow};
use std::{env::args, fs::File, io::Read};

use crate::io::parse_xml;

mod io;

fn main() -> Result<()> {
	let arg = args()
		.nth(1)
		.ok_or(anyhow!("Please specify the filename"))?;
	let file = File::open(arg).context("Error reading file")?;
	let mut decompressed = String::new();
	flate2::read::DeflateDecoder::new(file).read_to_string(&mut decompressed).context("Error decompressing file")?;
	let parsed = parse_xml(&decompressed).unwrap();
	println!("{parsed:?}");
	Ok(())
}

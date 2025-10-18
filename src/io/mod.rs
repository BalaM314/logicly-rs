use std::collections::HashMap;

use anyhow::{anyhow, Result};
use serde::{Deserialize};




#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename = "logicly")]
pub struct RawCircuit {
	#[serde(rename = "@xmlns")]
	xmlns: Option<String>,
	#[serde(rename = "object")]
	objects: Vec<RawObject>,
	#[serde(rename = "connection")]
	connections: Vec<RawConnection>,
	#[serde(rename = "setting")]
	settings: Vec<Setting>,
	#[serde(rename = "custom")]
	customs: Option<Vec<CustomCircuitWrapper>>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct RawObject {
	#[serde(rename = "@type")]
	kind: String,
	#[serde(rename = "@uid")]
	uid: String,
	#[serde(rename = "@x")]
	x: f64,
	#[serde(rename = "@y")]
	y: f64,
	#[serde(rename = "@rotation")]
	rotation: u16,
	#[serde(rename = "@exportName")]
	export_name: Option<String>,
	#[serde(rename = "@outputs")]
	outputs: Option<String>,
	#[serde(rename = "@inputs")]
	inputs: Option<u32>,
	#[serde(rename = "@text")]
	text: Option<String>,
	#[serde(rename = "@functionIndex")]
	function_index: Option<u8>
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct RawConnection {
	#[serde(rename = "@inputUID")]
	input_uid: String,
	#[serde(rename = "@outputUID")]
	output_uid: String,
	#[serde(rename = "@inputIndex")]
	input_index: u32,
	#[serde(rename = "@outputIndex")]
	output_index: u32,
	#[serde(rename = "@points")]
	points: Option<String>
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Setting {
	#[serde(rename = "@name")]
	name: String,
	#[serde(rename = "@value")]
	value: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct CustomCircuitWrapper {
	#[serde(rename = "logicly")]
	inner: CustomCircuitData,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct CustomCircuitData {
	#[serde(rename = "object")]
	objects: Vec<RawObject>,
	#[serde(rename = "connection")]
	connections: Vec<RawConnection>,
	#[serde(rename = "setting")]
	locations: Vec<Location>,
	#[serde(rename = "custom")]
	customs: Vec<CustomCircuitWrapper>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Location {
	#[serde(rename = "@id")]
	id: String,
	#[serde(rename = "@uids")]
	uids: String,
}

pub fn parse_xml(input:&str) -> Result<RawCircuit> {
	serde_xml_rs::from_str(input).map_err(|e| anyhow!(e))
}


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
	#[serde(rename = "location")]
	locations: Vec<Location>,
	#[serde(rename = "custom")]
	customs: Option<Vec<CustomCircuitWrapper>>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Location {
	#[serde(rename = "@id")]
	id: String,
	#[serde(rename = "@uids")]
	uids: String,
}
#[derive(Debug, PartialEq)]
pub struct Circuit {
	pub objects: Vec<Object>,
}
impl Display for Circuit {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for (i, obj) in self.objects.iter().enumerate() {
			writeln!(f, "({i}) {obj}")?;
		}
		Ok(())
	}
}

#[derive(Debug, Eq, PartialEq)]
pub enum Rotation {
	Right,
	Down,
	Left,
	Up
}

impl TryFrom<u16> for Rotation {
	type Error = String;
	fn try_from(value: u16) -> Result<Self, Self::Error> {
		Ok(match value {
			0 => Rotation::Right,
			90 => Rotation::Down,
			180 => Rotation::Left,
			270 => Rotation::Up,
			_ => return Err(format!("Unsupported rotation {value}"))
		})
	}
}

#[derive(Debug, PartialEq)]
pub struct Object {
	uid: String,
	x: f64,
	y: f64,
	rotation: Rotation,
	pub inner: ObjectInner,
}
impl Object {
	pub fn is_output(&self) -> bool {
		matches!(self.inner, ObjectInner::Output { .. })
	}
	/// Must be an Output or Input
	pub fn export_name_or_uid(&self) -> &str {
		match &self.inner {
			ObjectInner::Output { export_name, .. } | ObjectInner::Input { export_name, .. } => export_name.as_ref().unwrap_or(&self.uid),
			_ => panic!("Not an Output or Input")
		}
	}
}
impl Display for Object {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		fn print_connections(connections: &Vec<Option<(u32, usize)>>) -> String {
			connections.iter().map(|x| match x {
				Some((ind, ptr)) if *ind == 0 => format!("{ptr}"),
				Some((ind, ptr)) => format!("{ptr}#{ind}"),
				None => format!("NUL")
			}).collect::<Vec<_>>().join(", ")
		}
		match &self.inner {
			ObjectInner::SimpleGate { kind, connections, .. } => write!(f, "Gate {kind} [{}]", print_connections(connections)),
			ObjectInner::Output { export_name, connections } => write!(f, "Output({}) {}", export_name.clone().unwrap_or("?".to_string()), print_connections(connections)),
			ObjectInner::Input { export_name, kind, value } => write!(f, "Input({}) {kind} {value}", export_name.clone().unwrap_or("?".to_string())),
			ObjectInner::Label { text } => write!(f, "Label: {text}"),
		}
	}
}
impl TryFrom<RawObject> for Object {
	type Error = String;
	fn try_from(value: RawObject) -> Result<Self, Self::Error> {
		Ok(match &value.kind[..] {
			"switch@logic.ly" | "push_button@logic.ly" | "constant_high@logic.ly" | "constant_low@logic.ly" => match value {
				RawObject { kind, uid, x, y, rotation, export_name, outputs, inputs: None, text: None, function_index: None } => Self {
					uid, x, y,
					rotation: rotation.try_into()?,
					inner: ObjectInner::Input {
						export_name,
						kind: kind[..].try_into()?,
						value: match &outputs {
							Some(str) => match &str[..] {
								"false" => false, "true" => true,
								x => return Err(format!("invalid output field in object: expected 'true' or 'false', not {x}"))
							},
							None if matches!(&kind[..], "constant_high@logic.ly" | "constant_low@logic.ly") =>
								kind == "constant_high@logic.ly",
							None => return Err(format!("Invalid gate"))
						},
					}
				},
				_ => return Err(format!("Invalid gate: unexpected property")),
			},
			"light_bulb@logic.ly" | "digit@logic.ly" => match value {
				RawObject { uid, x, y, rotation, export_name, outputs: None, inputs: None, text: None, function_index: None, kind: _ } => Self {
					uid, x, y,
					rotation: rotation.try_into()?,
					inner: ObjectInner::Output {
						export_name,
						connections: vec![None; if value.kind == "light_bulb@logic.ly" { 1 } else { 4 }],
					}
				},
				_ => return Err(format!("Invalid light bulb")),
			},
			"label@logic.ly" => match value {
				RawObject { uid, x, y, rotation, export_name: None, outputs: None, inputs: None, text: Some(text), function_index: None, kind: _ } => Self {
					uid, x, y,
					rotation: rotation.try_into()?,
					inner: ObjectInner::Label { text }
				},
				_ => return Err(format!("Invalid label")),
			},
			"buffer@logic.ly" | "not@logic.ly" |
			"and@logic.ly" | "nand@logic.ly" |
			"or@logic.ly" | "nor@logic.ly" |
			"xor@logic.ly" | "xnor@logic.ly" => match value {
				RawObject { uid, x, y, kind, rotation, export_name: None, outputs: None, inputs: Some(inputs), text: None, function_index } => Self {
					uid, x, y,
					rotation: rotation.try_into()?,
					inner: ObjectInner::SimpleGate {
						connections: vec![None; inputs as usize],
						kind: kind[..].try_into()?,
						xor_type: match function_index {
							Some(1) => XorType::One,
							_ => XorType::Odd,
						},
					}
				},
				_ => return Err(format!("Invalid label")),
			},
			x => return Err(format!("Unsupported object type {x}"))
		})
	}
}
#[derive(Debug, PartialEq)]
pub enum ObjectInner {
	SimpleGate {
		xor_type: XorType,
		kind: SimpleGateType,
		connections: Vec<Option<(u32, usize)>>,
	},
	Output {
		export_name: Option<String>,
		connections: Vec<Option<(u32, usize)>>,
	},
	Input {
		export_name: Option<String>,
		kind: InputType,
		/// unused
		value: bool,
	},
	Label {
		text: String,
	},
}
#[derive(Debug, Eq, PartialEq)]
pub enum InputType {
	Switch, Button, True, False
}
impl TryFrom<&str> for InputType {
	type Error = String;
	fn try_from(value: &str) -> Result<Self, Self::Error> {
		Ok(match value {
			"switch@logic.ly" => Self::Switch,
			"push_button@logic.ly" => Self::Button,
			"constant_high@logic.ly => " => Self::True,
			"constant_low@logic.ly" => Self::False,
			_ => return Err(format!("invalid type {value}"))
		})
	}
}
impl Display for InputType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", match self {
			InputType::Switch => "Switch",
			InputType::Button => "Button",
			InputType::True => "True",
			InputType::False => "False",
		})
	}
}
#[derive(Debug, Eq, PartialEq)]
pub enum SimpleGateType {
	Buffer, Not,
	And, Nand,
	Or, Nor,
	Xor, Xnor,
}
impl TryFrom<&str> for SimpleGateType {
	type Error = String;
	fn try_from(value: &str) -> Result<Self, Self::Error> {
		use SimpleGateType as S;
		Ok(match value {
			"buffer@logic.ly" => S::Buffer,
			"not@logic.ly" => S::Not,
			"and@logic.ly" => S::And,
			"nand@logic.ly" => S::Nand,
			"or@logic.ly" => S::Or,
			"nor@logic.ly" => S::Nor,
			"xor@logic.ly" => S::Xor,
			"xnor@logic.ly" => S::Xnor,
			_ => return Err(format!("invalid type for simple gate: {value}"))
		})
	}
}
impl Display for SimpleGateType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", match self {
			SimpleGateType::Buffer => "Buffer",
			SimpleGateType::Not => "Not",
			SimpleGateType::And => "And",
			SimpleGateType::Nand => "Nand",
			SimpleGateType::Or => "Or",
			SimpleGateType::Nor => "Nor",
			SimpleGateType::Xor => "Xor",
			SimpleGateType::Xnor => "Xnor",
		})
	}
}
#[derive(Debug, Eq, PartialEq)]
pub enum XorType {
	Odd, One
}
impl TryFrom<RawCircuit> for Circuit {
	type Error = String;
	fn try_from(value: RawCircuit) -> Result<Self, Self::Error> {
		let mut objects = value.objects.into_iter()
			.map(|o| Object::try_from(o))
			.collect::<Result<Vec<_>, String>>()?;
		let uid_to_index: HashMap::<String, usize> = objects.iter().enumerate().map(|(i, o)| (o.uid.clone(), i)).collect();
		for obj in &value.connections {
			let output = *uid_to_index.get(&obj.output_uid)
				.ok_or(String::from("UUID does not correspond to any known object"))?;
			let input = *uid_to_index.get(&obj.input_uid)
				.ok_or(String::from("UUID does not correspond to any known object"))?;
			match &mut objects[input].inner {
				ObjectInner::SimpleGate { connections, .. } | ObjectInner::Output { connections, .. } =>
					connections[obj.input_index as usize] = Some((obj.output_index, output)),
				ObjectInner::Input {..} | ObjectInner::Label {..} =>
					return Err(String::from("Invalid connection: cannot connect an output or a label to something else")),
			}
		}
		Ok(Self { objects })
	}
}

pub fn parse_xml(input:&str) -> Result<Circuit> {
	let raw: RawCircuit = serde_xml_rs::from_str(input)?;
	Circuit::try_from(raw).map_err(|e| anyhow!(e))
}


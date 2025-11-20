use std::collections::{HashMap, HashSet};
use std::fmt::Display;

use anyhow::{anyhow, Result};
use itertools::Itertools;
use serde::{Deserialize};
use uuid::Uuid;




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

#[derive(Debug, Clone, Deserialize, PartialEq)]
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

#[derive(Clone, Debug, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct CustomCircuitWrapper {
	#[serde(rename = "@name")]
	name: String,
	#[serde(rename = "@type")]
	uid: String,
	#[serde(rename = "@label")]
	label: String,
	#[serde(rename = "logicly")]
	inner: RawCustomCircuit,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct RawCustomCircuit {
	#[serde(rename = "object")]
	objects: Vec<RawObject>,
	#[serde(rename = "connection")]
	connections: Vec<RawConnection>,
	#[serde(rename = "location")]
	locations: Vec<Location>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Location {
	#[serde(rename = "@id")]
	id: String,
	#[serde(rename = "@uids")]
	uids: String,
}
#[derive(Debug, PartialEq)]
pub struct Circuit {
	pub objects: Vec<Object>,
	/// If present, the circuits must be in a valid dependency order,
	/// so that all circuits must come after their dependencies.
	pub customs: Option<Vec<CustomCircuit>>,
}
impl Circuit {
	fn process_objects(
		objects: Vec<RawObject>,
		connections: Vec<RawConnection>,
		customs: &Vec<CustomCircuit>
	) -> Result<Vec<Object>, String> {
		let customs: HashMap<_, _> = customs.iter().map(|c| (c.uid.clone(), c)).collect();
		let mut objects = objects.into_iter()
			.map(|o| Object::try_from(o, &customs))
			.collect::<Result<Vec<_>, String>>()?;
		let uid_to_index: HashMap::<String, usize> = objects.iter().enumerate().map(|(i, o)| (o.uid.clone(), i)).collect();
		for obj in connections {
			let output = *uid_to_index.get(&obj.output_uid)
				.ok_or(String::from("UUID does not correspond to any known object"))?;
			let input = *uid_to_index.get(&obj.input_uid)
				.ok_or(String::from("UUID does not correspond to any known object"))?;
			match &mut objects[input].inner {
				ObjectInner::SimpleGate { connections, .. } | ObjectInner::CustomGate { connections, .. } | ObjectInner::Output { connections, .. } =>
					connections[obj.input_index as usize] = Some((obj.output_index, output)),
				ObjectInner::Input {..} | ObjectInner::Label {..} =>
					return Err(String::from("Invalid connection: cannot connect an output or a label to something else")),
			}
		}
		Ok(objects)
	}
}
impl Display for Circuit {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for (i, obj) in self.objects.iter().enumerate() {
			writeln!(f, "({i}) {obj}")?;
		}
		Ok(())
	}
}

#[derive(Debug, PartialEq)]
pub struct CustomCircuit {
	pub objects: Vec<Object>,
	pub name: String,
	pub uid: String,
	pub label: String,
	pub locations: Vec<Location>,
}

impl CustomCircuit {
	fn try_from(CustomCircuitWrapper {
		name, uid, label, inner: RawCustomCircuit {
			objects, connections, locations
		}
	}: CustomCircuitWrapper, customs: &Vec<CustomCircuit>) -> Result<Self, String> {
		Ok(Self {
			name, uid, label, locations,
			objects: Circuit::process_objects(objects, connections, customs)?,
		})
	}
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
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
	pub fn is_named_output(&self) -> bool {
		matches!(self.inner, ObjectInner::Output { export_name: Some(_), .. })
	}
	pub fn is_named_input(&self) -> bool {
		matches!(self.inner, ObjectInner::Input { export_name: Some(_), .. })
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
			ObjectInner::CustomGate { uuid, connections, .. } => write!(f, "CustomGate {uuid} [{}]", print_connections(connections)),
			ObjectInner::Output { export_name, connections } => write!(f, "Output({}) {}", export_name.clone().unwrap_or("?".to_string()), print_connections(connections)),
			ObjectInner::Input { export_name, kind, value } => write!(f, "Input({}) {kind} {value}", export_name.clone().unwrap_or("?".to_string())),
			ObjectInner::Label { text } => write!(f, "Label: {text}"),
		}
	}
}
impl Object {
	fn try_from(value: RawObject, customs: &HashMap<String, &CustomCircuit>) -> Result<Self, String> {
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
				_ => return Err(format!("Invalid label: attributes are invalid")),
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
				_ => return Err(format!("Invalid gate: attributes are invalid")),
			},
			uuid if Uuid::try_parse(uuid).is_ok() => match value {
				RawObject { uid, x, y, rotation, export_name: None, outputs: None, inputs: None, text: None, .. } => Self {
					inner: {
						let gate = customs.get(uuid).ok_or(format!("Unknown custom circuit {uid}"))?;
						let num_inputs = gate.objects.iter().filter(|o| o.is_named_input()).count();
						let num_outputs = gate.objects.iter().filter(|o| o.is_named_output()).count() as u32;
						ObjectInner::CustomGate {
							connections: vec![None; num_inputs as usize],
							num_outputs,
							uuid: uuid.to_string(),
						}
					},
					uid, x, y,
					rotation: rotation.try_into()?,
				},
				_ => return Err(format!("Invalid label: attributes are invalid, {value:?}")),
			},
			x => return Err(format!("Unsupported object type {x}"))
		})
	}
}
#[derive(Clone, Debug, PartialEq)]
pub enum ObjectInner {
	SimpleGate {
		xor_type: XorType,
		kind: SimpleGateType,
		connections: Vec<Option<(u32, usize)>>,
	},
	CustomGate {
		uuid: String,
		num_outputs: u32,
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputType {
	Switch, Button, True, False
}
impl TryFrom<&str> for InputType {
	type Error = String;
	fn try_from(value: &str) -> Result<Self, Self::Error> {
		Ok(match value {
			"switch@logic.ly" => Self::Switch,
			"push_button@logic.ly" => Self::Button,
			"constant_high@logic.ly" => Self::True,
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum XorType {
	Odd, One
}
impl TryFrom<RawCircuit> for Circuit {
	type Error = String;
	fn try_from(RawCircuit { connections, customs, objects, .. }: RawCircuit) -> Result<Self, Self::Error> {
		let customs: Option<Vec<CustomCircuit>> = match customs {
			Some(c) => {
				let c = order_dependency_graph(c)?;
				let mut customs = vec![];
				for custom in c {
					customs.push(CustomCircuit::try_from(custom, &customs)?);
				}
				Some(customs)
			},
			None => None,
		};
		let objects = Circuit::process_objects(
			objects, connections, customs.as_ref().unwrap_or(&vec![])
		)?;
		Ok(Self {
			objects,
			customs,
		})
	}
}

pub fn order_dependency_graph(items: Vec<CustomCircuitWrapper>) -> Result<Vec<CustomCircuitWrapper>, String> {
	let mut items_deps: Vec<_> = items.into_iter().map(|item|{
		let deps: HashSet<_> = item.inner.objects.iter().filter_map(|o| match Uuid::try_parse(&o.kind) {
			Ok(_) => Some(o.kind.clone()),
			Err(_) => None
		}).collect();
		Some((item, deps))
	}).collect();
	// let mapping: HashMap<_, _> = items.iter().enumerate().map(|(i, x)| (&x.uid[..], i)).collect();
	let mut output = Vec::with_capacity(items_deps.len());
	//O(n^2) toposort
	while output.len() != output.capacity() {
		let mut i = 0;
		let mut removed_any = false;
		while i < items_deps.len() {
			if let Some((_, deps)) = &items_deps[i] {
				if deps.is_empty() {
					removed_any = true;
					let (removed, _) = items_deps[i].take().unwrap();
					for x in items_deps.iter_mut() {
						if let Some((_, deps)) = x {
							deps.remove(&removed.uid);
						}
					}
					output.push(removed);
				}
			}
			i += 1;
		}
		if !removed_any {
			//Find the dependency cycle
			let mut edges: Vec<&String> = vec![];
			let mut i = 0;
			let mut updated = false;
			loop {
				if let Some((item, deps)) = &items_deps[i] {
					if let Some((j, _)) = edges.iter().enumerate().find(|(_, x)| ***x == item.uid) {
						let mut cycle = edges[j..].to_vec();
						cycle.push(&item.uid);
						return Err(format!("Circuit contains a dependency cycle: {}", cycle.iter().join(" -> ")));
					}
					if deps.contains(&item.uid) {
						return Err(format!("Circuit contains a dependency cycle: {} -> {}", item.uid, item.uid));
					}
					edges.push(&item.uid);
					updated = true;
					if let Some((next_i, _)) = items_deps.iter().enumerate().find(|(_, x)|
						x.as_ref().is_some_and(|(y, _)| y.uid == *deps.iter().next().unwrap())
					) {
						if i == next_i {
							return Err(format!("Circuit contains a dependency cycle: {} -> {}", item.uid, item.uid));
						}
						i = next_i;
					} else {
						return Err(format!("Circuit contains a dependency cycle: failed to find it"));
					}
				}
				if i >= items_deps.len() {
					if !updated {
						return Err(format!("Circuit contains a dependency cycle: failed to find it"));
					}
					i = 0;
					updated = false;
				}
			}
		}
	}
	Ok(output)
}

pub fn parse_xml(input:&str) -> Result<Circuit> {
	let raw: RawCircuit = serde_xml_rs::from_str(input)?;
	Circuit::try_from(raw).map_err(|e| anyhow!(e))
}

#[cfg(test)]
mod tests {
	use crate::io::*;

	fn name_to_uuid(name: &str) -> Uuid {
		let mut name = name.as_bytes().to_vec();
		name.resize(16, 0);
		Uuid::from_bytes(name.try_into().unwrap())
	}
	fn make_circuit(name: &'static str, deps: Vec<&'static str>) -> CustomCircuitWrapper {
		CustomCircuitWrapper {
			label: String::from(""),
			uid: name_to_uuid(name).to_string(),
			name: name.to_string(),
			inner: RawCustomCircuit {
				objects: deps.into_iter().map(|s| RawObject {
					kind: name_to_uuid(s).to_string(),
					uid: Uuid::new_v4().to_string(),
					x: 0.,
					y: 0.,
					rotation: 0,
					export_name: None,
					outputs: None,
					inputs: None,
					text: None,
					function_index: None,
				}).collect(),
				connections: vec![],
				locations: vec![]
			}
		}
	}
	#[test]
	fn orderdeps_ordered_1(){
		let a = make_circuit("a", vec![]);
		let b = make_circuit("b", vec![]);
		let c = make_circuit("c", vec![]);
		let d = make_circuit("d", vec![]);
		let deps = vec![a, b, c, d];
		assert_eq!(order_dependency_graph(deps.clone()), Ok(deps));
	}
	#[test]
	fn orderdeps_ordered_2(){
		let a = make_circuit("a", vec![]);
		let b = make_circuit("b", vec!["a"]);
		let c = make_circuit("c", vec!["b"]);
		let d = make_circuit("d", vec!["c"]);
		let deps = vec![a, b, c, d];
		assert_eq!(order_dependency_graph(deps.clone()), Ok(deps));
	}
	#[test]
	fn orderdeps_ordered_3(){
		let a = make_circuit("a", vec![]);
		let b = make_circuit("b", vec!["a"]);
		let c = make_circuit("c", vec!["a", "b"]);
		let d = make_circuit("d", vec!["b", "c"]);
		let deps = vec![a, b, c, d];
		assert_eq!(order_dependency_graph(deps.clone()), Ok(deps));
	}
	#[test]
	fn orderdeps_reorder_1(){
		let a = make_circuit("a", vec!["c"]);
		let b = make_circuit("b", vec![]);
		let c = make_circuit("c", vec!["b"]);
		let d = make_circuit("d", vec![]);
		let e = make_circuit("e", vec!["b"]);
		let deps = vec![a.clone(), b.clone(), c.clone(), d.clone(), e.clone()];
		assert_eq!(order_dependency_graph(deps.clone()), Ok(vec![b, c, d, e, a]));
	}
	#[test]
	fn orderdeps_reorder_2(){
		let a = make_circuit("a", vec!["c", "e"]);
		let b = make_circuit("b", vec!["d", "c"]);
		let c = make_circuit("c", vec![]);
		let d = make_circuit("d", vec!["c"]);
		let e = make_circuit("e", vec!["b"]);
		let deps = vec![a.clone(), b.clone(), c.clone(), d.clone(), e.clone()];
		assert_eq!(order_dependency_graph(deps.clone()), Ok(vec![c, d, b, e, a]));
	}
	#[test]
	fn orderdeps_cycle_1(){
		let a = make_circuit("a", vec!["a"]);
		let deps = vec![a.clone()];
		assert_eq!(order_dependency_graph(deps.clone()), Err(format!("Circuit contains a dependency cycle: {} -> {}", a.uid, a.uid)));
	}
	#[test]
	fn orderdeps_cycle_2(){
		let a = make_circuit("a", vec!["b"]);
		let b = make_circuit("b", vec!["a"]);
		let deps = vec![a.clone(), b.clone()];
		assert_eq!(order_dependency_graph(deps.clone()), Err(format!("Circuit contains a dependency cycle: {} -> {} -> {}", a.uid, b.uid, a.uid)));
	}
	#[test]
	fn orderdeps_cycle_3(){
		let a = make_circuit("a", vec!["b"]);
		let b = make_circuit("b", vec!["c"]);
		let c = make_circuit("c", vec!["d"]);
		let d = make_circuit("d", vec!["a"]);
		let deps = vec![d.clone(), c.clone(), b.clone(), a.clone()];
		assert_eq!(order_dependency_graph(deps.clone()), Err(format!(
			"Circuit contains a dependency cycle: {} -> {} -> {} -> {} -> {}",
			d.uid, a.uid, b.uid, c.uid, d.uid
		)));
	}
}

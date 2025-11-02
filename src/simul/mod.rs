use std::{collections::HashMap, fmt::Display, ops::Deref};
use crate::io::{Circuit, InputType, Object, ObjectInner, SimpleGateType, XorType};


#[derive(Debug, Clone, PartialEq)]
pub struct Simulation {
	objects: Vec<SObject>,
}
impl From<Circuit> for Simulation {
	fn from(value: Circuit) -> Self {
		Self { objects: value.objects.into_iter().map(SObject::from).collect() }
	}
}
impl Simulation {
	pub fn print_outputs(&self){
		for obj in &self.objects {
			if obj.is_output() || matches!(obj.object.inner, ObjectInner::Input { .. }) {
				println!("{}: {:?}", obj.export_name_or_uid(), obj.values)
			}
		}
	}
	/// Returns a mutable reference to all inputs with an export name, in the form of a hash map.
	/// Panics if multiple inputs have the same export name.
	pub fn get_inputs_mut(&mut self) -> HashMap<&str, &mut bool> {
		let mut map = HashMap::new();
		for obj in &mut self.objects {
			match &mut obj.object.inner {
				ObjectInner::Input {
					export_name: Some(name),
					kind: InputType::Button | InputType::Switch,
					..
				} => { map.insert(&name[..], obj.values.get_mut(0).unwrap()); },
				_ => {}
			}
		}
		map
	}
	pub fn inputs_mut(&mut self) -> impl Iterator<Item = &mut SObject> {
		self.objects.iter_mut().flat_map(|o| match &mut o.object.inner {
			ObjectInner::Input { export_name: Some(_), .. } => Some(o),
			_ => None
		})
	}
	pub fn outputs(&self) -> impl Iterator<Item = &SObject> {
		self.objects.iter().flat_map(|o| match &o.object.inner {
			ObjectInner::Output { export_name: Some(_), .. } => Some(o),
			_ => None
		})
	}
	/// Returns if any changes were made.
	pub fn update_all_once(&mut self) -> bool {
		let mut changed = false;
		for i in 0..self.objects.len() {
			let obj = &self.objects[i];
			if let Some(new_val) = obj.get_new_value(&self.objects) {
				if new_val != self.objects[i].values { changed = true }
				self.objects[i].values = new_val;
			}
		}
		changed
	}
	pub fn update_until_done(&mut self, limit: u128){
		for _ in 1..limit {
			if !self.update_all_once() { break }
		}
	}
	/// Sets all non-constant objects to false.
	pub fn reset_state(&mut self){
		for obj in &mut self.objects {
			match obj.inner {
				ObjectInner::Input { kind: InputType::Button | InputType::Switch, .. }
				| ObjectInner::SimpleGate { .. } | ObjectInner::Output { .. } => {
					for val in &mut obj.values { *val = false; }
				},
				_ => continue,
			}
		}
	}
	/// Resets the state, then finds the outputs of this simulation given some inputs.
	pub fn get_outputs(&mut self, inputs: &HashMap<&str, bool>, limit: u128) -> HashMap<String, bool> {
		self.reset_state();
		for obj in &mut self.objects {
			match &mut obj.object.inner {
				ObjectInner::Input {
					export_name: Some(name),
					kind: InputType::Button | InputType::Switch,
					..
				} => {
					if let Some(&val) = inputs.get(&name[..]) {
						obj.values[0] = val;
					}
				},
				_ => {}
			}
		}
		self.update_until_done(limit);
		self.objects.iter().flat_map(|f| match &f.inner {
			ObjectInner::Output { export_name: Some(name), .. } => Some((name.clone(), f.values[0])),
			_ => None
		}).collect()
	}
	pub fn print_truth_table(&mut self, limit: u128){
		let mut input_names: Vec<_> = self.objects.iter().flat_map(|o| match &o.inner {
			ObjectInner::Input { export_name: Some(name), .. } => Some(name.clone()),
			_ => None,
		}).collect();
		input_names.sort();
		let mut output_names: Vec<_> = self.objects.iter().flat_map(|o| match &o.inner {
			ObjectInner::Output { export_name: Some(name), .. } => Some(name.clone()),
			_ => None,
		}).collect();
		output_names.sort();
		let mut inputs: HashMap<_, _> = input_names.iter().map(|w| (&w[..], false)).collect();
		let header = input_names.iter().chain(output_names.iter()).map(|s| &s[..]).collect::<Vec<_>>();
		let header_str = header.join("|");
		println!("{}", header_str);
		println!("{}", "-".repeat(header_str.len()));
		for i in 0..2u32.pow(input_names.len() as u32) {
			for (bit, input) in input_names.iter().rev().enumerate() {
				let value = (i >> bit) & 1 == 1;
				inputs.insert(&input[..], value);	
			}
			let outputs = self.get_outputs(&inputs, limit);
			let line = input_names.iter().map(|inp| inputs.get(&inp[..]).unwrap()).chain(
				output_names.iter().map(|out| outputs.get(&out[..]).unwrap())
			).enumerate().map(|(i, val)| format!("{:^width$}", match val {
				true => "T",
				false => "F"
			}, width = header[i].len())).collect::<Vec<_>>().join("|");
			println!("{line}");
		}
	}
	fn get_values(connections: &Vec<Option<(u32, usize)>>, objects: &Vec<SObject>) -> Vec<bool> {
		connections.iter().map(|c| match c {
			&Some((idx, ptr)) => objects[ptr].values[idx as usize],
			None => false,
		}).collect()
	}
}
impl Display for Simulation {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for (i, obj) in self.objects.iter().enumerate() {
			writeln!(f, "({i}) {} | {:?}", obj.object, obj.values)?;
		}
		Ok(())
	}
}
#[derive(Debug, Clone, PartialEq)]
pub struct SObject {
	object: Object,
	values: Vec<bool>,
}
impl SObject {
	/// Returns None if the object does not support updating.
	fn get_new_value(&self, objects: &Vec<SObject>) -> Option<Vec<bool>> {
		use SimpleGateType as S;
		return match &self.object.inner {
			ObjectInner::SimpleGate { xor_type, kind, connections } => {
				let inputs = Simulation::get_values(connections, objects);
				Some(vec![match kind {
					S::Buffer => inputs[0],
					S::Not => !inputs[0],
					S::And => inputs.iter().all(|x| *x),
					S::Nand => !inputs.iter().all(|x| *x),
					S::Or => inputs.iter().any(|x| *x),
					S::Nor => !inputs.iter().any(|x| *x),
					S::Xor | S::Xnor => (match xor_type {
						XorType::Odd => inputs.iter().filter(|x| **x).count() % 2 == 1,
						XorType::One => inputs.iter().filter(|x| **x).count() == 1,
					} == (*kind == S::Xor)),
				}])
			},
			crate::io::ObjectInner::Output { connections, .. } =>
				Some(Simulation::get_values(connections, objects)),
			ObjectInner::Input { .. } => None, // Inputs do not change themselves
			ObjectInner::Label { .. } => None,
		};
	}
}
impl Deref for SObject {
	type Target = Object;
	fn deref(&self) -> &Self::Target {
		&self.object
	}
}
impl From<Object> for SObject {
	fn from(object: Object) -> Self {
		let values = match &object.inner {
			// For now all gates have only 1 output
			ObjectInner::SimpleGate { .. } => 1,
			ObjectInner::Output { .. } => 1,
			ObjectInner::Input { .. } => 1,
			ObjectInner::Label { .. } => 0,
		};
		let value = match &object.inner {
			ObjectInner::Input { value, .. } => *value,
			_ => false,
		};
		Self {
			object,
			values: vec![value; values],
		}
	}
}



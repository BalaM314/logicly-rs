use std::{collections::HashMap, ops::Deref};

use crate::io::{Circuit, InputType, Object, ObjectInner, SimpleGateType, XorType};


#[derive(Debug, PartialEq)]
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
	fn get_values(connections: &Vec<Option<(u32, usize)>>, objects: &Vec<SObject>) -> Vec<bool> {
		connections.iter().map(|c| match c {
			&Some((idx, ptr)) => objects[ptr].values[idx as usize],
			None => false,
		}).collect()
	}
}
#[derive(Debug, PartialEq)]
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
					S::Xor | S::Xnor => match xor_type {
						XorType::Odd => inputs.iter().filter(|x| **x).count() % 2 == 1,
						XorType::One => inputs.iter().filter(|x| **x).count() == 1,
					},
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
		Self {
			object,
			values: vec![false; values],
		}
	}
}



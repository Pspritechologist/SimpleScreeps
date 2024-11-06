use std::str::FromStr;

use js_sys::Object;
use vecmap::VecMap;
use wasm_bindgen::{JsCast, JsValue};

use super::parsing::tokens::VmValue;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum Num {
	Float(f32),
	Int(i32),
}

impl From<&VmValue> for Object {
	fn from(value: &VmValue) -> Self {
		match value {
			VmValue::Null => wasm_bindgen::JsValue::null().into(),
			VmValue::Bool(value) => js_sys::Boolean::from(*value).into(),
			VmValue::Number(value) => match value {
				Num::Float(f) => js_sys::Number::from(*f),
				Num::Int(i) => js_sys::Number::from(*i)
			}.into(),
			VmValue::String(value) => js_sys::JsString::from_str(value).unwrap().into(),
			VmValue::List(value) => {
				let array = js_sys::Array::new_with_length(value.len() as u32);
				for (i, value) in value.iter().enumerate() {
					array.set(i as u32, Object::from(value).into());
				}
				array.into()
			}
			VmValue::Map(value) => {
				let object = Object::new();
				for (key, value) in value.into_iter() {
					js_sys::Reflect::set(&object, &js_sys::JsString::from_str(key).unwrap(), &Object::from(value).into()).unwrap();
				}
				object
			}
		}
	}
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum EndState {
	Success,
	Failure,
	Running,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum Operation {
	SetBlackBoard { key: String },
	GetBlackBoard { key: String },
	Push { value: VmValue },
	Pop { count: u32 },
	IsNull,
	Has { key: String },
	CallMethod { func: String, args: u32 },
	If { skip: usize },
	Skip { skip: usize },
	EndTick(EndState),
	Add,
	Sub,
	Mul,
	Div,
	Mod,
	And,
	Or,
	Eq,
	Neq,
	Lt,
	Gt,
	Lte,
	Gte,
	Not,
}

#[derive(Debug)]
pub enum Error {
	StackUnderflow,
	FuncNotFound(String),
	FuncCallFailed(wasm_bindgen::JsValue),
	BadType,
	OutOfOps,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct HtnVm {
	#[serde(with = "blackboard_serde")]
	blackboard: VecMap<String, Object>,
	ops: Vec<Operation>,
	index: usize,
}

#[derive(Debug, Clone, Default)]
struct Stack(Vec<Object>);
impl std::ops::Deref for Stack {
	type Target = Vec<Object>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl std::ops::DerefMut for Stack {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}
impl AsRef<Vec<Object>> for Stack {
	fn as_ref(&self) -> &Vec<Object> {
		&self.0
	}
}
impl AsMut<Vec<Object>> for Stack {
	fn as_mut(&mut self) -> &mut Vec<Object> {
		&mut self.0
	}
}
impl Stack {
	fn pop(&mut self) -> Result<Object, Error> {
		self.0.pop().ok_or(Error::StackUnderflow)
	}

	fn peek(&self) -> Result<&Object, Error> {
		self.0.last().ok_or(Error::StackUnderflow)
	}
}

mod blackboard_serde {
	use serde::{ser::SerializeMap, Deserialize, Deserializer, Serializer};
	use vecmap::VecMap;

	pub fn serialize<S>(value: &VecMap<String, js_sys::Object>, serializer: S) -> Result<S::Ok, S::Error>
	where S: Serializer,
	{
		let mut map = serializer.serialize_map(Some(value.len()))?;
		for (key, value) in value.iter() {
			match js_sys::JSON::stringify(value) {
				Ok(value) => map.serialize_entry(key, &value.as_string().unwrap())?,
				_ => map.serialize_entry(key, &())?,
			}
		}

		map.end()
	}

	pub fn deserialize<'de, D>(deserializer: D) -> Result<VecMap<String, js_sys::Object>, D::Error>
	where D: Deserializer<'de>,
	{
		let map: VecMap<String, String> = Deserialize::deserialize(deserializer)?;
		let result = map.into_iter().map(|(key, value)| 
			(key, js_sys::JSON::parse(&value).unwrap().into())
		).collect();
		
		Ok(result)
	}
}

impl HtnVm {
	async fn execute(mut self) -> Result<EndState, Error> {
		use Operation::*;

		let mut stack = Stack::default();

		while self.index < self.ops.len() {
			match &self.ops[self.index] {
				SetBlackBoard { key } => {
					let value = stack.pop()?;
					self.blackboard.insert(key.clone(), value);
				}
				GetBlackBoard { key } => {
					let value = self.blackboard.get::<str>(key.as_ref()).cloned().unwrap_or_else(Object::new);
					stack.push(value.clone());
				}
				Push { value } => {
					stack.push(value.into());
				}
				Pop { count } => {
					if *count > stack.len() as u32 {
						return Err(Error::StackUnderflow);
					}

					(0..*count).for_each(|_| { stack.pop(); } );
				}
				IsNull => {
					let is_null = stack.peek()?.is_null();
					stack.push(js_sys::Boolean::from(is_null).into())
				}
				Has { key } => {
					let value = stack.peek()?;
					let result = js_sys::Reflect::has(value, &js_sys::JsString::from_str(key).unwrap()).map_err(|_| Error::BadType)?;
					stack.push(js_sys::Boolean::from(result).into());
				}
				CallMethod { func, args } => {
					let args = js_sys::Array::new_with_length(*args);
					for _ in 0..args.length() {
						args.push(&stack.pop()?.into());
					}

					let target = stack.pop()?;

					let js_func = js_sys::Reflect::get(&target, &js_sys::JsString::from_str(func).unwrap()).map_err(|_| Error::FuncNotFound(func.clone()))?;
					let js_func: js_sys::Function = js_func.dyn_into().map_err(|_| Error::BadType)?;

					let result = js_sys::Reflect::apply(&js_func, &target, &args).map_err(Error::FuncCallFailed)?;
					
					stack.push(result.into());
				}
				If { skip } => {
					let value = js_sys::Boolean::from(stack.pop()?.is_truthy());
					if value.as_bool().unwrap() {
						self.index += skip;
					}
				}
				Skip { skip } => {
					self.index += skip;
				}
				EndTick(state) => {
					return Ok(*state);
				}
				Add => { let (a, b) = (stack.pop()?, stack.pop()?); stack.push((JsValue::from(a) + JsValue::from(b)).into()); }
				Sub => { let (a, b) = (stack.pop()?, stack.pop()?); stack.push((JsValue::from(a) - JsValue::from(b)).into()); }
				Mul => { let (a, b) = (stack.pop()?, stack.pop()?); stack.push((JsValue::from(a) * JsValue::from(b)).into()); }
				Div => { let (a, b) = (stack.pop()?, stack.pop()?); stack.push((JsValue::from(a) / JsValue::from(b)).into()); }
				Mod => { let (a, b) = (stack.pop()?, stack.pop()?); stack.push((JsValue::from(a) % JsValue::from(b)).into()); }
				And => { let (a, b) = (stack.pop()?, stack.pop()?); stack.push((JsValue::from(a) & JsValue::from(b)).into()); }
				Or => { let (a, b) = (stack.pop()?, stack.pop()?); stack.push((JsValue::from(a) | JsValue::from(b)).into()); }
				Eq => { let (a, b) = (stack.pop()?, stack.pop()?); stack.push(JsValue::from_bool(JsValue::from(a) == JsValue::from(b)).into()); }
				Neq => { let (a, b) = (stack.pop()?, stack.pop()?); stack.push(JsValue::from_bool(JsValue::from(a) != JsValue::from(b)).into()); }
				Lt => { let (a, b) = (stack.pop()?, stack.pop()?); stack.push(JsValue::from_bool(JsValue::from(a).lt(&JsValue::from(b))).into()); }
				Gt => { let (a, b) = (stack.pop()?, stack.pop()?); stack.push(JsValue::from_bool(JsValue::from(a).gt(&JsValue::from(b))).into()); }
				Lte => { let (a, b) = (stack.pop()?, stack.pop()?); stack.push(JsValue::from_bool(JsValue::from(a).le(&JsValue::from(b))).into()); }
				Gte => { let (a, b) = (stack.pop()?, stack.pop()?); stack.push(JsValue::from_bool(JsValue::from(a).ge(&JsValue::from(b))).into()); }
				Not => { let t = stack.pop()?; stack.push(JsValue::from_bool(!JsValue::from(t).is_truthy()).into()); }
			}

			self.index += 1;
		}

		Err(Error::OutOfOps)
	}
}

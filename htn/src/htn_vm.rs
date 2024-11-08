use std::str::FromStr;

use js_sys::{JsString, Object};
use vecmap::VecMap;
use wasm_bindgen::{JsCast, JsValue};

use crate::parsing::tokens::VecMapWrapper;

use super::parsing::tokens::VmValue;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, self_rust_tokenize::SelfRustTokenize)]
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
		}
	}
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, self_rust_tokenize::SelfRustTokenize)]
pub enum EndState {
	Success,
	Failure,
	Running,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, self_rust_tokenize::SelfRustTokenize)]
pub enum Operation {
	SetBlackBoard { key: String },
	GetBlackBoard { key: String },
	GetGlobal { key: String },
	Push { value: VmValue },
	DefineObj { fields: Vec<String> },
	Pop { count: u32 },
	IsNull,
	Has { key: String },
	Access { key: String },
	Call { args: u32, method: bool },
	SkipIf { skip: u32, invert: bool },
	Skip { skip: u32 },
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
	Dbg,
}

#[derive(Debug)]
pub enum Error {
	StackUnderflow,
	FuncNotFound(String),
	FuncCallFailed(wasm_bindgen::JsValue),
	BadType,
	OutOfOps,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct HtnVm {
	pub blackboard: VecMap<String, ObjectWrapper>,
	// #[serde(skip)]
	pub ops: Vec<Operation>,
	#[serde(skip)]
	pub index: usize,
}

// pub enum VmType {
// 	Object(Object),
// 	Num(Num),
// 	Boolean(bool),
// }

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

#[derive(Debug, Clone)]
pub struct ObjectWrapper(Object);
impl std::ops::Deref for ObjectWrapper {
	type Target = Object;
	fn deref(&self) -> &Self::Target { &self.0 }
}
impl std::ops::DerefMut for ObjectWrapper {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}
impl AsRef<Object> for ObjectWrapper {
	fn as_ref(&self) -> &Object { &self.0 }
}
impl AsMut<Object> for ObjectWrapper {
	fn as_mut(&mut self) -> &mut Object { &mut self.0 }
}
impl AsRef<JsValue> for ObjectWrapper {
	fn as_ref(&self) -> &JsValue { &self.0 }
}
impl From<Object> for ObjectWrapper {
	fn from(value: Object) -> Self { Self(value) }
}
impl From<ObjectWrapper> for Object {
	fn from(value: ObjectWrapper) -> Self { value.0 }
}
impl From<JsValue> for ObjectWrapper {
	fn from(value: JsValue) -> Self { Object::from(value).into() }
}
impl From<ObjectWrapper> for JsValue {
	fn from(value: ObjectWrapper) -> Self { value.0.into() }
}
impl wasm_bindgen::JsCast for ObjectWrapper {
	fn instanceof(val: &JsValue) -> bool {
		Object::instanceof(val)
	}

	fn unchecked_from_js(val: JsValue) -> Self {
		Self(val.unchecked_into())
	}

	fn unchecked_from_js_ref(val: &JsValue) -> &Self {
		unsafe { &*(val as *const JsValue as *const Self) }
	}
}
impl serde::Serialize for ObjectWrapper {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
		let json = match js_sys::JSON::stringify(&self.0) {
			Ok(json) => json,
			Err(err) => {
				log::error!("Error serializing object: {:?}", err.as_string().unwrap_or_default());
				// return Err(serde::ser::Error::custom(format!("Error serializing object: {:?}", err)))
				JsString::from_str("null").unwrap()
			},
		};
		// let Some(string) = json.as_string() else {
		// 	log::error!("JSON value was not a string!");
		// 	return Err(serde::ser::Error::custom("JSON value was not a string"));
		// };
		match json.as_string() {
			Some(string) => string,
			None => {
				log::error!("Error serializing object");
				String::from_str("null").unwrap()
			},	
		}.serialize(serializer)
	}
}
impl<'de> serde::Deserialize<'de> for ObjectWrapper {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
		let json: String = serde::Deserialize::deserialize(deserializer)?;
		// Ok(Self(match js_sys::JSON::parse(&json) {
		// 	Ok(value) => value,
		// 	Err(err) => {
		// 		log::error!("Error deserializing object: {:?}", err.as_string().unwrap_or_default());
		// 		return Err(serde::de::Error::custom(format!("Error deserializing object: {:?}", err)))
		// 	},
		// }.into()))
		Ok(Self(match js_sys::JSON::parse(&json) {
			Ok(value) => value,
			Err(err) => {
				log::error!("Error deserializing object: {:?}", err.as_string().unwrap_or_default());
				Default::default()
			},
		}.into()))
	}
}
impl ObjectWrapper {
	fn new() -> Self { Self(Object::new()) }
}

impl HtnVm {
	pub fn execute(&mut self) -> Result<EndState, Error> {
		use Operation::*;

		let mut stack = Stack::default();

		log::trace!("Executing VM with op: {:?}", self.ops);

		while self.index < self.ops.len() {
			let op = &self.ops[self.index];

			log::trace!("Executing op: {:?}", op);

			match op {
				SetBlackBoard { key } => {
					let value = stack.pop()?;
					self.blackboard.insert(key.clone(), value.into());
				}
				GetBlackBoard { key } => {
					let value = self.blackboard.get::<str>(key.as_ref()).cloned().unwrap_or_else(ObjectWrapper::new);
					stack.push(value.clone().into());
				}
				GetGlobal { key } => {
					let value = js_sys::Reflect::get(&js_sys::global(), &js_sys::JsString::from_str(key).unwrap()).map_err(|_| Error::BadType)?;
					stack.push(value.into());
				}
				Push { value } => {
					stack.push(value.into());
				}
				DefineObj { fields } => {
					let obj = js_sys::Object::new();
					for key in fields {
						js_sys::Reflect::set(&obj, &key.into(), &stack.pop()?.into()).map_err(|_| Error::BadType)?;
					}
					stack.push(obj);
				}
				Pop { count } => {
					if *count > stack.len() as u32 {
						return Err(Error::StackUnderflow);
					}

					(0..*count).for_each(|_| { stack.pop(); } );
				}
				IsNull => {
					let is_null = /* stack.peek()?.is_null() ||  */stack.peek()?.is_undefined();
					stack.push(js_sys::Boolean::from(is_null).into())
				}
				Has { key } => {
					let value = stack.peek()?;
					let result = js_sys::Reflect::has(value, &js_sys::JsString::from_str(key).unwrap()).map_err(|_| Error::BadType)?;
					stack.push(js_sys::Boolean::from(result).into());
				}
				Access { key } => {
					let value = stack.pop()?;
					let result = js_sys::Reflect::get(&value, &js_sys::JsString::from_str(key).unwrap()).map_err(|_| Error::BadType)?;
					stack.push(result.into());
				}
				Call { args, method } => {
					let (js_func, target) = if *method {
						(stack.pop()?, stack.pop()?.into())
					} else {
						(stack.pop()?, JsValue::undefined())
					};

					let args = js_sys::Array::new_with_length(*args);
					for _ in 0..args.length() {
						args.push(&stack.pop()?.into());
					}

					// let js_func = js_sys::Reflect::get(&target, &js_sys::JsString::from_str(func).unwrap()).map_err(|_| Error::FuncNotFound(func.clone()))?;
					let js_func: js_sys::Function = js_func.dyn_into().map_err(|_| Error::BadType)?;

					let result = js_sys::Reflect::apply(&js_func, &target, &args).map_err(Error::FuncCallFailed)?;
					
					stack.push(result.into());
				}
				SkipIf { skip, invert } => {
					let value = js_sys::Boolean::from(stack.pop()?.is_truthy());
					if value.as_bool().unwrap() != *invert {
						self.index += usize::try_from(*skip).unwrap(); // ! ??
					}
				}
				Skip { skip } => {
					self.index += usize::try_from(*skip).unwrap(); // ! ??
				}
				EndTick(state) => {
					return Ok(*state);
				}
				Add => { let (b, a) = (stack.pop()?, stack.pop()?); stack.push((JsValue::from(a) + JsValue::from(b)).into()); }
				Sub => { let (b, a) = (stack.pop()?, stack.pop()?); stack.push((JsValue::from(a) - JsValue::from(b)).into()); }
				Mul => { let (b, a) = (stack.pop()?, stack.pop()?); stack.push((JsValue::from(a) * JsValue::from(b)).into()); }
				Div => { let (b, a) = (stack.pop()?, stack.pop()?); stack.push((JsValue::from(a) / JsValue::from(b)).into()); }
				Mod => { let (b, a) = (stack.pop()?, stack.pop()?); stack.push((JsValue::from(a) % JsValue::from(b)).into()); }
				And => { let (b, a) = (stack.pop()?, stack.pop()?); stack.push((JsValue::from(a) & JsValue::from(b)).into()); }
				Or => { let (b, a) = (stack.pop()?, stack.pop()?); stack.push((JsValue::from(a) | JsValue::from(b)).into()); }
				Eq => { let (b, a) = (stack.pop()?, stack.pop()?); stack.push(JsValue::from_bool(JsValue::from(a) == JsValue::from(b)).into()); }
				Neq => { let (b, a) = (stack.pop()?, stack.pop()?); stack.push(JsValue::from_bool(JsValue::from(a) != JsValue::from(b)).into()); }
				Lt => { let (b, a) = (stack.pop()?, stack.pop()?); stack.push(JsValue::from_bool(JsValue::from(a).lt(&JsValue::from(b))).into()); }
				Gt => { let (b, a) = (stack.pop()?, stack.pop()?); stack.push(JsValue::from_bool(JsValue::from(a).gt(&JsValue::from(b))).into()); }
				Lte => { let (b, a) = (stack.pop()?, stack.pop()?); stack.push(JsValue::from_bool(JsValue::from(a).le(&JsValue::from(b))).into()); }
				Gte => { let (b, a) = (stack.pop()?, stack.pop()?); stack.push(JsValue::from_bool(JsValue::from(a).ge(&JsValue::from(b))).into()); }
				Not => { let t = stack.pop()?; stack.push(JsValue::from_bool(!JsValue::from(t).is_truthy()).into()); }
				Dbg => {
					log::debug!("DBG: {}", js_sys::JSON::stringify(stack.peek()?).unwrap_or_else(|_| "INVALID_JSON".to_string().into()))
				}
			}

			self.index += 1;
		}

		Err(Error::OutOfOps)
	}
}

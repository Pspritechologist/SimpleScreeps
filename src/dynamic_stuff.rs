use crate::state::{*, general_states::*, harvester::*, upgrader::*, builder::*, seppuku::*};
use std::{any::Any, fmt::Debug};
use serde::ser::SerializeMap;

pub type BoxedState = Box<dyn DynStateTrait>;

pub trait DynStateTrait: erased_serde::Serialize + Debug + Any {
	fn run(&mut self, creep: &Creep, data: &mut crate::memory::CreepData) -> StateResult<Box<dyn Debug>, Box<dyn Debug>>;
	fn as_any(&self) -> &dyn Any;
	fn as_any_mut(&mut self) -> &mut dyn Any;
}

erased_serde::serialize_trait_object!(DynStateTrait);

impl<T: State<Error = E, Return = R> + serde::Serialize + Debug + Any, E: Debug + 'static, R: Debug + 'static> DynStateTrait for T {
	fn run(&mut self, creep: &Creep, data: &mut crate::memory::CreepData) -> StateResult<Box<dyn Debug>, Box<dyn Debug>> {
		let result = T::run(self, creep, data);
		match result {
			StateResult::Working => StateResult::Working,
			StateResult::Finished(r) => StateResult::Finished(Box::new(r)),
			StateResult::Failed(e) => StateResult::Failed(Box::new(e)),
		}
	}

	fn as_any(&self) -> &dyn Any { self }

	fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl<T: State<Error = E, Return = R> + serde::Serialize + Debug + Any, E: Debug + 'static, R: Debug + 'static> From<T> for BoxedState {
	fn from(state: T) -> Self {
		Box::new(state)
	}
}

#[derive(Debug)]
pub struct DynState {
	pub state: BoxedState,
	pub flag: StateFlag,
}

impl DynState {
	pub fn cast<T: State<Error = E, Return = R> + 'static, E: Debug + 'static, R: Debug + 'static>(&self) -> Option<&T> {
		self.state.as_any().downcast_ref::<T>()
	}

	pub fn cast_mut<T: State<Error = E, Return = R> + 'static, E: Debug + 'static, R: Debug + 'static>(&mut self) -> Option<&mut T> {
		self.state.as_any_mut().downcast_mut::<T>()
	}

	pub fn new<T: DynStateTrait>(state: T, flag: StateFlag) -> Self {
		Self { state: Box::new(state), flag }
	}
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum StateFlag {
	Idle,
	Seppuku,
	Harvesting,
	Transfer,
	Withdraw,
	Move,
	HarvesterJob,
	UpgraderJob,
	BuilderJob,
}

impl<'de> serde::Deserialize<'de> for DynState {
	fn deserialize<D>(deserializer: D) -> Result<DynState, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct DynStateVisitor;

		impl<'de> serde::de::Visitor<'de> for DynStateVisitor {
			type Value = DynState;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("a valid DynState")
			}

			fn visit_map<A>(self, mut map: A) -> Result<DynState, A::Error>
			where
				A: serde::de::MapAccess<'de>,
			{
				let Some(flag) = map.next_key()? else {
					return Err(serde::de::Error::custom("no key found"));
				};

				let state: BoxedState = match flag {
					StateFlag::Idle => Box::new(map.next_value::<StateIdle>()?),
					StateFlag::Seppuku => Box::new(map.next_value::<StateSeppuku>()?),
					StateFlag::Harvesting => Box::new(map.next_value::<StateHarvesting>()?),
					StateFlag::Transfer => Box::new(map.next_value::<StateTransfer>()?),
					StateFlag::Withdraw => Box::new(map.next_value::<StateWithdraw>()?),
					StateFlag::Move => Box::new(map.next_value::<StateMove>()?),
					StateFlag::HarvesterJob => Box::new(map.next_value::<StateHarvesterJob>()?),
					StateFlag::UpgraderJob => Box::new(map.next_value::<StateUpgraderJob>()?),
					StateFlag::BuilderJob => Box::new(map.next_value::<StateBuilderJob>()?),
				};

				Ok(DynState { state, flag })
			}
		}

		deserializer.deserialize_map(DynStateVisitor)
	}
}

impl serde::Serialize for DynState {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let mut entry = serializer.serialize_map(Some(1))?;
		entry.serialize_entry(&self.flag, &self.state)?;
		entry.end()
	}
}

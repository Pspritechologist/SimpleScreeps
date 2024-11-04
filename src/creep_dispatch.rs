use std::error::Error;

use screeps::{find::{self}, Creep, ErrorCode, HasId, HasPosition, ResourceType, SharedCreepProperties, StructureController, StructureObject};
use vecmap::VecMap;
use screeps::prelude::*;

use crate::{handle_err, handle_warn, temp::{CreepData, Duty, Target}, utils::{self, chance}};

#[derive(Debug)]
pub struct DispatchContext {
	fulfilment: VecMap<Duty, usize>,
}

impl DispatchContext {
	pub fn new(count: usize) -> Self {
		let ratios = Duty::get_ratios();
		let max: usize = ratios.iter().map(|(_, ratio)| *ratio).sum();
		
		let mut fulfilment = VecMap::new();
		for (duty, ratio) in ratios {
			fulfilment.insert(duty, (count * ratio).div_ceil(max));
		}

		Self { fulfilment, }
	}

	pub fn get_fulfilment(&self) -> &VecMap<Duty, usize> {
		&self.fulfilment
	}

	fn get_fulfil(&self, duty: &Duty) -> usize {
		*self.fulfilment.get(duty).unwrap()
	}

	fn get_fulfil_mut(&mut self, duty: &Duty) -> &mut usize {
		self.fulfilment.get_mut(duty).unwrap()
	}

	pub fn unique_task(&mut self, creep: &Creep, creep_state: &mut CreepData, task: &crate::unique_tasks::UniqueTask) -> Result<(), Box<dyn Error>> {
		// use crate::unique_tasks::UniqueTask::*;
		// enum Res {
		// 	Controller(StructureController),
		// 	RoomOffset((i8, i8)),
		// }

		// match task {
		// 	SignMessage(msg) => {
		// 		let movement = match creep_state.target.as_ref().ok_or(format!("No target for task: {:?}", task))? {
		// 			Target::Room(room_name) => {
		// 				// screeps::game::rooms().get(*room_name).or_else(|| utils::get_relative_coords_from_room(room_name, creep.room()?.name())).ok_or(format!("Room not found: {:?}", room_name))?.controller().ok_or(format!("Room has no controller: {:?}", room_name))?,
		// 				if let Some(room) = screeps::game::rooms().get(*room_name) {
		// 					Res::Controller(room.controller().ok_or(format!("Room has no controller: {:?}", room_name))?)
		// 				} else {
		// 					Res::RoomOffset(utils::get_relative_coords_from_room(*room_name, creep.room().unwrap().name()).ok_or(format!("Room not found: {:?}", room_name))?)
		// 				}
		// 			},
		// 			Target::Controller(id) => Res::Controller(id.resolve().ok_or(format!("Controller not found: {:?}", id))?),
		// 			_ => return Err(format!("Invalid target for task: {:?}", task).into()),
		// 		};

		// 		// Get the controller if we have one, or are in the same room as one.
		// 		// Otherwise, move to where we need to go and return OK
		// 		let controller = match movement {
		// 			Res::Controller(controller) => controller,
		// 			Res::RoomOffset((x, y)) => {
		// 				// X and Y are relative to the current room, that many rooms over.
		// 				let room = creep.room().unwrap();
		// 				let x_func = |x: i8| {
		// 					room.exi
		// 				};
		// 				let y_func = |y: i8| {

		// 				};

		// 				if x + y > 0 {
		// 					if x > y {
		// 						x_func(x);
		// 					} else {
		// 						y_func(y);
		// 					}
		// 				} else {
		// 					todo!("Get controller")
		// 				}

		// 				panic!()
		// 			}
		// 		};

		// 		match creep.sign_controller(&controller, msg) {
		// 			Err(ErrorCode::NotInRange) => handle_warn!(creep.move_to(&controller)),
		// 			Ok(_) => {
		// 				creep_state.unique_task = None;
		// 				creep_state.target = None;
		// 			},
		// 			Err(e) => log::warn!("[{}:{}:{}]: {:?}", file!(), line!(), column!(), &e),
		// 		}
		// 	},
		// 	Arbitrary(msg) => {
		// 		log::error!("Unknown unique task: {}", msg);
		// 	}
		// }

		Ok(())
	}

	#[allow(clippy::collapsible_if)]
	pub fn dispatch_creep(&mut self, creep: &Creep, creep_state: &mut CreepData) -> Option<()> {

		// log::info!("Tick fulfilment: {:?}", self.fulfilment);

		if let Some(task) = creep_state.unique_task.take() {
			if let Err(err) = self.unique_task(creep, creep_state, &task) {
				log::warn!("Creep {} Failed to complete unique task '{:?}':\n{}", creep.name(), task, err);
			}
		}

		if matches!(creep_state.current_task, Some(Duty::Harvest)) || (creep_state.duties.contains(&Duty::Harvest) && self.get_fulfil(&Duty::Harvest) > 0) {
			if self.dispatch_harvester(creep, creep_state).is_some() {
				*self.get_fulfil_mut(&Duty::Harvest) -= 1;
				return Some(());
			}
		}

		if matches!(creep_state.current_task, Some(Duty::Build)) || (creep_state.duties.contains(&Duty::Build) && self.get_fulfil(&Duty::Build) > 0) {
			if self.dispatch_builder(creep, creep_state).is_some() {
				*self.get_fulfil_mut(&Duty::Build) -= 1;
				return Some(());
			}
		}

		if matches!(creep_state.current_task, Some(Duty::Repair)) || (creep_state.duties.contains(&Duty::Repair) && self.get_fulfil(&Duty::Repair) > 0) {
			if self.dispatch_repairer(creep, creep_state).is_some() {
				*self.get_fulfil_mut(&Duty::Repair) -= 1;
				return Some(());
			}
		}

		if matches!(creep_state.current_task, Some(Duty::Upgrade)) || (creep_state.duties.contains(&Duty::Upgrade) && self.get_fulfil(&Duty::Upgrade) > 0) {
			if self.dispatch_upgrader(creep, creep_state).is_some() {
				*self.get_fulfil_mut(&Duty::Upgrade) -= 1;
				return Some(());
			}
		}

		if chance(15) {
			handle_warn!(creep.say("Idle...", false));
		}

		Some(())
	}

	pub fn dispatch_harvester(&mut self, creep: &Creep, creep_state: &mut CreepData) -> Option<()> {
		let room = creep.room()?;

		if creep.store().get_free_capacity(None) > (creep.store().get_capacity(None) / 2).try_into().unwrap() {
			creep_state.target = None;

			let Some(spawn) = creep_state.spawn.as_ref() else {
				log::warn!("Creep {} has no Spawn", creep.name());
				handle_err!(creep.say("no home :(", false));
				return None;
			};

			creep_state.target = Some()

			match creep.transfer(spawn, ResourceType::Energy, None) {
				Err(ErrorCode::NotInRange) => handle_warn!(creep.move_to(spawn)),
				Err(ErrorCode::Full) => {
					creep_state.current_task = Some(Duty::Upgrade);
				},
				Ok(_) => {
					creep_state.current_task = None;
					creep_state.target = None;
				},
				Err(e) => log::warn!("[{}:{}:{}]: {:?}", file!(), line!(), column!(), &e),
			}

			return Some(());
		}

		let target = if let Some(Target::Source(id)) = creep_state.target {
			id.resolve()?
		} else {
			room.find(find::SOURCES, None).into_iter().max_by(|a, b| a.energy().cmp(&b.energy()))?
		};

		if creep.harvest(&target).is_err() {
			handle_err!(creep.move_to(&target));
		}

		creep_state.target = Some(Target::Source(target.id()));
		creep_state.current_task = Some(Duty::Harvest);

		Some(())
	}

	pub fn dispatch_builder(&mut self, creep: &Creep, creep_state: &mut CreepData) -> Option<()> {
		None
	}

	pub fn dispatch_repairer(&mut self, creep: &Creep, creep_state: &mut CreepData) -> Option<()> {
		None
	}

	pub fn dispatch_upgrader(&mut self, creep: &Creep, creep_state: &mut CreepData) -> Option<()> {
		match creep_state.target {
			Some(Target::Controller(id)) if id.resolve().is_some() => {
				let controller = id.resolve()?;
				if !creep.pos().in_range_to(controller.pos(), 3) {
					handle_warn!(creep.move_to(&controller));
				}

				handle_warn!(creep.upgrade_controller(&controller));

				if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
					creep_state.target = None;
					creep_state.current_task = None;
				}
			},
			Some(Target::EnergyStorage(id)) if id.resolve().is_some() => {
				let structure = StructureObject::from(id.resolve()?);
				let container = structure.as_withdrawable()?;

				match creep.withdraw(container, ResourceType::Energy, None) {
					Err(ErrorCode::NotInRange) => handle_warn!(creep.move_to(container)),
					Ok(_) => creep_state.target = Some(Target::Controller(creep_state.spawn.as_ref().and_then(|s| s.room()?.controller()).unwrap().id())),
					Err(e) => log::warn!("[{}:{}:{}]: {:?}", file!(), line!(), column!(), &e),
				}
			},
			_ => {
				let room = creep.room()?;
				if creep.store().get_used_capacity(Some(ResourceType::Energy)) > creep.store().get_capacity(Some(ResourceType::Energy)) / 2 {
					let controller = creep_state.spawn.as_ref().and_then(|s| s.room()?.controller()).unwrap_or(room.controller()?);
					creep_state.target = Some(Target::Controller(controller.id()));
				} else {
					let structures = creep_state.spawn.as_ref().and_then(|s| Some(s.room()?.find(find::STRUCTURES, None)))?;
					let mut withdrawable = structures.into_iter().filter(|obj| obj.as_withdrawable().is_some());
					
					//TODO: This uses next- make it do something smart.
					creep_state.target = Some(Target::EnergyStorage(withdrawable.next()?.as_structure().id()));
				}
			}
		}

		creep_state.current_task = Some(Duty::Upgrade);

		Some(())
	}
}

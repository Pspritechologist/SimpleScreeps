use screeps::{find, Creep, HasId, HasPosition, SharedCreepProperties};
use vecmap::VecMap;

use crate::{handle_err, temp::{CreepData, Duty, Target}};

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

	pub fn dispatch_creep(&mut self, creep: Creep, creep_state: &mut CreepData) -> Option<()> {
		let room = creep.room()?;

		if creep.store().get_free_capacity(None) == 0 {
			creep_state.target = None;

			let Some(spawn) = creep_state.spawn.as_ref() else {
				log::warn!("Creep {} has no home", creep.name());
				handle_err!(creep.say("no home :(", false));
				return None;
			};

			if creep.pos().is_near_to(spawn.pos()) {
				handle_err!(creep.transfer(spawn, screeps::ResourceType::Energy, None));
			} else {
				handle_err!(creep.move_to(spawn.pos()));
			}

			return Some(());
		}

		let target = if let Some(Target::Source(id)) = creep_state.target {
			id.resolve()?
		} else {
			room.find(find::SOURCES, None).into_iter().max_by(|a, b| a.energy().cmp(&b.energy()))?
		};

		if creep.pos().is_near_to(target.pos()) {
			handle_err!(creep.harvest(&target));
		} else {
			handle_err!(creep.move_to(&target));
		}

		creep_state.target = Some(Target::Source(target.id()));

		Some(())
	}
}

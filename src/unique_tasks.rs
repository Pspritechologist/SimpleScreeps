use screeps::{Part, Position, RoomCoordinate, RoomName};

use crate::temp::CreepData;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum UniqueTask {
	SignMessage(String),
	Arbitrary(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum QueuedUniqueTask {
	SignMessage(String, RoomName),
	Arbitrary(String),
}

pub fn handle_task(task: QueuedUniqueTask) -> (CreepData, &'static [Part]) {
	use QueuedUniqueTask::*;
	match task {
		SignMessage(msg, room) => {
			(
				CreepData {
					duties: Vec::new(),
					current_task: None,
					unique_task: Some(UniqueTask::SignMessage(msg)),
					target: Some(crate::temp::Target::Room(room)),
					idle_pos: Position::new(RoomCoordinate::new(4).unwrap(), RoomCoordinate::new(4).unwrap(), room),
					spawn: None,
				},
				&[ screeps::Part::Move; 5 ]
			)
		},
		Arbitrary(msg) => {
			todo!()
		},
	}
}

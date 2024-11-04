use screeps::RoomName;

pub trait IterEnum {
	fn variants() -> &'static [Self] where Self: std::marker::Sized;
}

pub fn chance(percent: u8) -> bool {
	fastrand::u8(..100) < percent
}

#[macro_export]
macro_rules! handle_err {
	($e:expr) => {
		if let Err(err) = $e {
			log::warn!(
                "[{}:{}:{}]: {:?}\n\tsrc = {}", 
                file!(), 
                line!(), 
                column!(), 
                &err,
                {
                    let src = stringify!($e);
                    if src.len() > 45 {
                        format!("{}...", &src[..40])
                    } else {
                        src.to_string()
                    }
                }
            );
		}
	};
}

#[macro_export]
macro_rules! handle_warn {
    ($e:expr) => {
		if let Err(err) = $e {
			log::debug!(
                "[{}:{}:{}]: {:?}", 
                file!(), 
                line!(), 
                column!(), 
                &err,
            );
		}
    };
}

// pub fn get_relative_coords_from_room(target: RoomName, current: RoomName) -> Option<(i8, i8)> {
//     let target = target.to_array_string();
//     let mut target = target.chars();
//     let current = current.to_array_string();
//     let mut current = current.chars();
//     let (target_x, target_y) = (target.next().is_some().then(|| target.next()).flatten()?, target.next().is_some().then(|| target.next()).flatten()?);
//     let (current_x, current_y) = (current.next().is_some().then(|| current.next()).flatten()?, current.next().is_some().then(|| current.next()).flatten()?);

//     let (target_x, target_y) = (target_x.to_digit(10)? as i8, target_y.to_digit(10)? as i8);
//     let (current_x, current_y) = (current_x.to_digit(10)? as i8, current_y.to_digit(10)? as i8);

//     Some((target_x - current_x, target_y - current_y))
// }

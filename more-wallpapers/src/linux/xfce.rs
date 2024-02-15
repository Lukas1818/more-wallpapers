use super::check_command_error;
use crate::{error::CommandError, load_env_var, Environment, Mode, Screen, WallpaperBuilder, WallpaperError};
use std::{collections::HashMap, process::Command};

fn load_property(property: &str) -> Result<String, WallpaperError> {
	let mut command = Command::new("xfconf-query");
	command.args(["--channel", "xfce4-desktop", "p"]);
	command.arg(property);
	let output = check_command_error(command.output(), "xfconf-query")?;
	let output = String::from_utf8(output).unwrap();
	Ok(output)
}

pub(crate) fn get_screens() -> Result<Vec<Screen>, WallpaperError> {
	let mut command = Command::new("xfconf-query");
	command.args(["--channel", "xfce4-desktop", "--list"]);
	let output = check_command_error(command.output(), "xfconf-query")?;
	let output = String::from_utf8(output).unwrap();
	//	the outpult looks like the following:
	//
	//	/backdrop/screen0/monitor0/image-style
	//	/backdrop/screen0/monitor0/last-image
	//	/backdrop/screen0/monitor0/last-single-image
	//	/backdrop/screen0/monitorVirtual-1/workspace0/color-style
	//	/backdrop/screen0/monitorVirtual-1/workspace0/image-style
	//	/backdrop/screen0/monitorVirtual-1/workspace0/last-image
	//	/backdrop/screen0/monitorVirtual-1/workspace1/color-style
	//	/backdrop/screen0/monitorVirtual-1/workspace1/image-style
	//	/backdrop/screen0/monitorVirtual-1/workspace1/last-image
	let mut screens: HashMap<String, Screen> = Default::default();
	for line in output.lines().filter_map(|s| s.strip_prefix("/backdrop/")) {
		let mut split = line.split('/');
		let first = split.next();
		let second = split.next();
		let third = split.next();
		if split.next().is_some() {
			//to long -> wrong key
			break;
		}
		let (Some(first), Some(second)) = (first, second) else {
			//to short -> wrong key
			break;
		};
		let (screen_name, key_type, active) = if let Some(third) = third {
			// if name exist out of two part, the screen is active.
			// Otherwise it is default for new workspaces
			(format!("{}/{}", first, second), third, true)
		} else {
			(first.to_owned(), second, false)
		};
		if !(key_type == "last_image" || key_type == "image_style") {
			// wrong key
			break;
		}
		let value = load_property(line)?;
		let screen = screens.entry(screen_name.clone()).or_insert_with(|| Screen {
			name: screen_name,
			wallpaper: None,
			mode: None,
			active,
		});
		if key_type == "last_image" {
			screen.wallpaper = Some(value.into());
		} else {
			let mode = match value.as_str() {
				"0" => None, //single color background is used instead of a image
				"1" => Some(Mode::Center),
				"2" => Some(Mode::Tile),
				"3" => Some(Mode::Stretch),
				"4" => Some(Mode::Fit),
				"5" => Some(Mode::Crop),
				_ => return Err(WallpaperError::UnknownMode(value)),
			};
			screen.mode = mode;
		}
	}
	Ok(screens.into_values().collect())
}
use crate::{Mode, Screen};
use dbus::blocking::Connection;
use serde::Deserialize;
use serde_json;
use std::time::Duration;

#[derive(Deserialize)]
struct KDE_DESKTOP {
	screen: i32,
	id: u32,
}

fn plasmashell(command: &str) -> String {
	let destination = "org.kde.plasmashell";
	let interface = "org.kde.PlasmaShell";
	let path = "/PlasmaShell";
	let method = "evaluateScript";
	let args = (command,);
	let timeout = Duration::from_millis(5000);
	let conn = Connection::new_session().unwrap();
	let proxy = conn.with_proxy(destination, path, timeout);
	let (ret,): (String,) = proxy.method_call(interface, method, args).unwrap();
	ret
}

pub(crate) fn get_screens() -> Vec<Screen> {
	let desktops: Vec<KDE_DESKTOP> = serde_json::from_str(&plasmashell("print(JSON.stringify(desktops()));")).unwrap();
	let mut screens = std::vec::Vec::new();
	for desktop in desktops {
		if desktop.screen >= 0 {
			screens.push(Screen {
				name: desktop.id.to_string(),
				wallpaper: None,
				mode: None,
			});
		}
	}
	screens
}

pub(crate) fn set_screens(screens: Vec<Screen>) {
	let mut command = r#"
for (const desktop of desktops()) {
	desktop.currentConfigGroup = ["Wallpaper", "org.kde.image", "General"];"#
		.to_owned();
	for screen in screens {
		let mode = match screen.mode.unwrap() {
			Mode::Center => 6,
			Mode::Crop => 2,
			Mode::Fit => 1,
			Mode::Stretch => 0,
			Mode::Tile => 3,
		};
		command += &format!(
			r#"
	if (desktop.id === {}){{
		desktop.writeConfig("FillMode", {});
		desktop.writeConfig("Image", {:?});
	}}"#,
			screen.name,
			mode,
			screen.wallpaper.unwrap()
		);
	}
	command += r#"
}"#;
	println!("{}", command);
	plasmashell(&command);
}
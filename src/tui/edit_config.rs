use crate::config::{Config, Location};
use crate::tui::UserData;
use cursive::traits::{Nameable, Resizable};
use cursive::views::{Button, Dialog, DummyView, EditView, LinearLayout, ListView, NamedView};
use cursive::Cursive;
use geocoding::Openstreetmap;
use validator::{Validate, ValidationErrors};

const DAY_BRIGHTNESS: &str = "EDIT_CONFIG_DAY_BRIGHTNESS";
const NIGHT_BRIGHTNESS: &str = "EDIT_CONFIG_NIGHT_BRIGHTNESS";
const TRANSITION_MINS: &str = "EDIT_CONFIG_TRANSITION_MINS";
const LATITUDE: &str = "EDIT_CONFIG_LATITUDE";
const LONGITUDE: &str = "EDIT_CONFIG_LONGITUDE";

pub fn create<F>(config: Config, completion: F) -> Dialog
where
    F: 'static + Fn(&mut Cursive) + Clone,
{
    let day_brightness = EditView::new()
        .max_content_width(3)
        .content(format!("{}", config.brightness_day))
        .on_submit(on_submit_field);
    let night_brightness = EditView::new()
        .max_content_width(3)
        .content(format!("{}", config.brightness_night))
        .on_submit(on_submit_field);
    let transition_mins = EditView::new()
        .max_content_width(3)
        .content(format!("{}", config.transition_mins))
        .on_submit(on_submit_field);
    let latitude = EditView::new()
        .content(format!(
            "{:.5}",
            config.location.as_ref().map(|l| l.latitude).unwrap_or(0.0)
        ))
        .on_submit(on_submit_field);
    let longitude = EditView::new()
        .content(format!(
            "{:.5}",
            config.location.as_ref().map(|l| l.longitude).unwrap_or(0.0)
        ))
        .on_submit(on_submit_field);

    let x = ListView::new()
        .child(
            "Day Brightness:",
            NamedView::new(DAY_BRIGHTNESS, day_brightness).fixed_width(10),
        )
        .child(
            "Night Brightness:",
            NamedView::new(NIGHT_BRIGHTNESS, night_brightness),
        )
        .child(
            "Transition minutes:",
            NamedView::new(TRANSITION_MINS, transition_mins),
        )
        .child("Latitude:", NamedView::new(LATITUDE, latitude))
        .child("Longitude:", NamedView::new(LONGITUDE, longitude));

    let c2 = completion.clone();
    Dialog::around(
        LinearLayout::vertical()
            .child(DummyView)
            .child(x)
            .child(DummyView)
            .child(Button::new("Find location", geocode_dialog))
            .child(DummyView)
            .child(Button::new("Save", move |cursive| {
                if attempt_save(cursive) {
                    c2(cursive);
                }
            }))
            .child(DummyView)
            .child(Button::new("Cancel", completion)),
    )
    .title("Edit Configuration")
}

fn attempt_save(cursive: &mut Cursive) -> bool {
    match create_config(cursive) {
        Ok(cfg) => {
            let ud = cursive.user_data::<UserData>().unwrap();
            ud.state.write().unwrap().set_config(cfg.clone());
            match cfg.save() {
                Ok(_) => return true,
                Err(e) => {
                    cursive.add_layer(Dialog::info(e.to_string()));
                }
            };
        }
        Err(e) => {
            cursive.add_layer(Dialog::info(e));
        }
    }
    false
}

fn create_config(cursive: &mut Cursive) -> Result<Config, String> {
    let ud = cursive.user_data::<UserData>().unwrap();
    let mut config = ud.state.read().unwrap().get_config().clone();
    config.brightness_day = cursive
        .find_name::<EditView>(DAY_BRIGHTNESS)
        .unwrap()
        .get_content()
        .parse()
        .map_err(|_| "Day Brightness must be a number".to_owned())?;
    config.brightness_night = cursive
        .find_name::<EditView>(NIGHT_BRIGHTNESS)
        .unwrap()
        .get_content()
        .parse()
        .map_err(|_| "Night Brightness must be a number".to_owned())?;
    config.transition_mins = cursive
        .find_name::<EditView>(TRANSITION_MINS)
        .unwrap()
        .get_content()
        .parse()
        .map_err(|_| "Transition minutes must be a number".to_owned())?;
    config.location = Some(Location {
        latitude: cursive
            .find_name::<EditView>(LATITUDE)
            .unwrap()
            .get_content()
            .parse()
            .map_err(|_| "Latitude must be a number".to_owned())?,
        longitude: cursive
            .find_name::<EditView>(LONGITUDE)
            .unwrap()
            .get_content()
            .parse()
            .map_err(|_| "Longitude must be a number".to_owned())?,
    });
    config
        .validate()
        .map_err(|e: ValidationErrors| e.to_string())?;
    Ok(config)
}

fn on_apply(cursive: &mut Cursive) {
    let cfg = create_config(cursive);
    match &cfg {
        Err(e) => {
            cursive.add_layer(Dialog::info(e));
        }
        _ => {}
    };
}

fn on_submit_field(cursive: &mut Cursive, _: &str) {
    on_apply(cursive);
}

const ADDRESS: &str = "EDIT_CONFIG_ADDRESS";

fn geocode_dialog(cursive: &mut Cursive) {
    cursive.add_layer(
        Dialog::around(
            LinearLayout::vertical().child(DummyView).child(
                EditView::new()
                    .on_submit(|x, _| find_address(x))
                    .with_name(ADDRESS),
            ),
        )
        .title("Search for town/city")
        .button("Find", find_address)
        .dismiss_button("Cancel"),
    );
}

fn find_address(cursive: &mut Cursive) {
    let address = cursive
        .find_name::<EditView>(ADDRESS)
        .unwrap()
        .get_content();
    match Location::geocode_address(Openstreetmap::new(), address.as_str()) {
        Ok(l) => {
            cursive
                .find_name::<EditView>(LATITUDE)
                .unwrap()
                .set_content(format!("{:.5}", l.latitude));
            cursive
                .find_name::<EditView>(LONGITUDE)
                .unwrap()
                .set_content(format!("{:.5}", l.longitude));
            on_apply(cursive);
            cursive.pop_layer();
        }
        Err(e) => {
            cursive.add_layer(Dialog::info(e));
        }
    }
}

use crate::config::{Config, Location};
use cursive::Cursive;
use cursive::views::{Dialog, LinearLayout, Button, ListView, EditView, NamedView, DummyView};
use cursive::traits::{Resizable, Nameable};
use crate::tui::UserData;
use validator::{Validate, ValidationErrors};
use crate::brightness::BrightnessMessage;
use geocoding::Openstreetmap;

const DAY_BRIGHTNESS: &str = "EDIT_CONFIG_DAY_BRIGHTNESS";
const NIGHT_BRIGHTNESS: &str = "EDIT_CONFIG_NIGHT_BRIGHTNESS";
const TRANSITION_MINS: &str = "EDIT_CONFIG_TRANSITION_MINS";
const LATITUDE: &str = "EDIT_CONFIG_LATITUDE";
const LONGITUDE: &str = "EDIT_CONFIG_LONGITUDE";

pub fn create<F>(config: Config, completion: F) -> Dialog
    where F: 'static + Fn(&mut Cursive)
{
    let day_brightness = EditView::new().max_content_width(3).content(format!("{}", config.brightness_day)).on_submit(on_submit_field);
    let night_brightness = EditView::new().max_content_width(3).content(format!("{}", config.brightness_night)).on_submit(on_submit_field);
    let transition_mins = EditView::new().max_content_width(3).content(format!("{}", config.transition_mins)).on_submit(on_submit_field);
    let latitude = EditView::new().content(format!("{:.5}", config.location.latitude)).on_submit(on_submit_field);
    let longitude = EditView::new().content(format!("{:.5}", config.location.longitude)).on_submit(on_submit_field);

    let x = ListView::new()
        .child("Day Brightness:", NamedView::new(DAY_BRIGHTNESS, day_brightness).fixed_width(10))
        .child("Night Brightness:", NamedView::new(NIGHT_BRIGHTNESS, night_brightness))
        .child("Transition minutes:", NamedView::new(TRANSITION_MINS, transition_mins))
        .child("Latitude:", NamedView::new(LATITUDE, latitude))
        .child("Longitude:", NamedView::new(LONGITUDE, longitude));

    Dialog::around(
        LinearLayout::vertical()
            .child(DummyView)
            .child(x)
            .child(DummyView)
            .child(Button::new("Find location", geocode_dialog))
            .child(DummyView)
            .child(Button::new("Apply", on_apply))
            .child(DummyView)
            .child(Button::new("Back", completion))
    ).title("Edit Configuration")
}

fn on_apply(cursive: &mut Cursive) {
    fn create_config(cursive: &mut Cursive) -> Result<Config, String> {
        let ud = cursive.user_data::<UserData>().unwrap();
        let mut config = ud.status.read().unwrap().config.clone();
        config.brightness_day = cursive.find_name::<EditView>(DAY_BRIGHTNESS).unwrap()
            .get_content().parse().map_err(|_| "Day Brightness must be a number".to_owned())?;
        config.brightness_night = cursive.find_name::<EditView>(NIGHT_BRIGHTNESS).unwrap()
            .get_content().parse().map_err(|_| "Night Brightness must be a number".to_owned())?;
        config.transition_mins = cursive.find_name::<EditView>(TRANSITION_MINS).unwrap()
            .get_content().parse().map_err(|_| "Transition minutes must be a number".to_owned())?;
        config.location.latitude = cursive.find_name::<EditView>(LATITUDE).unwrap()
            .get_content().parse().map_err(|_| "Latitude must be a number".to_owned())?;
        config.location.longitude = cursive.find_name::<EditView>(LONGITUDE).unwrap()
            .get_content().parse().map_err(|_| "Longitude must be a number".to_owned())?;
        config.validate().map_err(|e: ValidationErrors| e.to_string())?;
        Ok(config)
    }
    let x = create_config(cursive);
    match x {
        Err(e) => {
            cursive.add_layer(Dialog::info(e));
        },
        Ok(c) => {
            let ud = cursive.user_data::<UserData>().unwrap();
            ud.status.write().unwrap().config = c;
            ud.brightness.send(BrightnessMessage::NewConfig).unwrap();
        }
    };
}

fn on_submit_field(cursive: &mut Cursive, _: &str) {
    on_apply(cursive);
}

const ADDRESS: &str = "EDIT_CONFIG_ADDRESS";

fn geocode_dialog(cursive: &mut Cursive) {
    cursive.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(DummyView)
                .child(EditView::new().on_submit(|x, _| find_address(x)).with_name(ADDRESS)))
            .title("Search for town/city")
            .button("Find", find_address)
            .dismiss_button("Cancel")
    );
}

fn find_address(cursive: &mut Cursive) {
    let address = cursive.find_name::<EditView>(ADDRESS).unwrap().get_content();
    match Location::geocode_address(Openstreetmap::new(), address.as_str()) {
        Ok(l) => {
            cursive.find_name::<EditView>(LATITUDE).unwrap().set_content(format!("{:.5}", l.latitude));
            cursive.find_name::<EditView>(LONGITUDE).unwrap().set_content(format!("{:.5}", l.longitude));
            on_apply(cursive);
            cursive.pop_layer();
        },
        Err(e) => {
            cursive.add_layer(Dialog::info(e));
        },
    }
}


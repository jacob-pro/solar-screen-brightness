use cursive::views::{Dialog, TextView};
use cursive::Cursive;
use crate::brightness::LastUpdate;
use chrono::{DateTime, Local};

const STATUS_PAGE: &str = "STATUS_PAGE";

pub fn create<F>(completion: F, update: LastUpdate) -> Dialog
where F: FnMut()
{
    Dialog::around(
        TextView::new(update_to_string(&update))
    ).title("Current Status")
}

pub fn update_to_string(update: &LastUpdate) -> String {
    let mut s = String::new();
    s.push_str(format!("Current Brightness: {}\n", update.brightness).as_str());
    s.push_str(format!("Calculated at: {}\n", DateTime::<Local>::from(update.time).format("%H:%M:%S")).as_str());
    s.push_str(format!("Expires at: {}\n", DateTime::<Local>::from(update.expiry).format("%H:%M:%S")).as_str());
    if update.visible {
        s.push_str(format!("Sunrise was at: {}\n", DateTime::<Local>::from(update.sunrise).format("%H:%M:%S")).as_str());
        s.push_str(format!("Sunset is at: {}\n", DateTime::<Local>::from(update.sunset).format("%H:%M:%S")).as_str());
    } else {
        s.push_str(format!("Sunset was at: {}\n", DateTime::<Local>::from(update.sunset).format("%H:%M:%S")).as_str());
        s.push_str(format!("Sunrise is at: {}\n", DateTime::<Local>::from(update.sunrise).format("%H:%M:%S")).as_str());
    }
    s
}

pub fn status_update(cursive: &mut Cursive, update: LastUpdate) {

}

use cursive::views::{Dialog, TextView, LinearLayout, Button};
use cursive::Cursive;
use cursive::traits::Nameable;
use crate::brightness::LastUpdate;
use chrono::{DateTime, Local};

const STATUS_TEXT: &str = "STATUS_TEXT";

pub fn create<F>(completion: F) -> Dialog
where F: 'static + Fn(&mut Cursive)
{
    Dialog::around(
        LinearLayout::vertical()
            .child(TextView::new("null").with_name(STATUS_TEXT))
            .child(Button::new("OK", completion))
    ).title("Current Status")
}

const DATE_FORMAT: &str = "%H:%M %P";

pub fn status_update(cursive: &mut Cursive, update: LastUpdate) {
    let mut s = String::new();
    s.push_str(format!("Brightness: {}%\n", update.brightness).as_str());
    s.push_str(format!("Changes at: {}\n", DateTime::<Local>::from(update.expiry).format(DATE_FORMAT)).as_str());
    if update.visible {
        s.push_str(format!("Sunrise was at: {}\n", DateTime::<Local>::from(update.sunrise).format(DATE_FORMAT)).as_str());
        s.push_str(format!("Sunset is at: {}\n", DateTime::<Local>::from(update.sunset).format(DATE_FORMAT)).as_str());
    } else {
        s.push_str(format!("Sunset was at: {}\n", DateTime::<Local>::from(update.sunset).format(DATE_FORMAT)).as_str());
        s.push_str(format!("Sunrise is at: {}\n", DateTime::<Local>::from(update.sunrise).format(DATE_FORMAT)).as_str());
    }
    cursive.call_on_name(STATUS_TEXT, |x: &mut TextView| {
        x.set_content(s);
    });
}

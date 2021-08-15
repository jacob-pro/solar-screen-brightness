use crate::runner::LastCalculation;
use chrono::{DateTime, Local};
use cursive::traits::Nameable;
use cursive::views::{Button, Dialog, DummyView, LinearLayout, TextView};
use cursive::Cursive;

const STATUS_TEXT: &str = "SHOW_STATUS_TEXT";

pub fn create<F>(completion: F) -> Dialog
where
    F: 'static + Fn(&mut Cursive),
{
    Dialog::around(
        LinearLayout::vertical()
            .child(DummyView)
            .child(TextView::new("null").with_name(STATUS_TEXT))
            .child(DummyView)
            .child(Button::new("Back", completion)),
    )
    .title("Current Status")
}

const DATE_FORMAT: &str = "%H:%M %P";

pub fn status_update(cursive: &mut Cursive, update: LastCalculation) {
    let mut s = String::new();
    s.push_str(format!("Brightness: {}%\n", update.brightness).as_str());
    s.push_str(
        format!(
            "Changes at: {}\n",
            DateTime::<Local>::from(update.expiry).format(DATE_FORMAT)
        )
        .as_str(),
    );
    if update.visible {
        s.push_str(
            format!(
                "Sunrise was at: {}\n",
                DateTime::<Local>::from(update.sunrise).format(DATE_FORMAT)
            )
            .as_str(),
        );
        s.push_str(
            format!(
                "Sunset is at: {}\n",
                DateTime::<Local>::from(update.sunset).format(DATE_FORMAT)
            )
            .as_str(),
        );
    } else {
        s.push_str(
            format!(
                "Sunset was at: {}\n",
                DateTime::<Local>::from(update.sunset).format(DATE_FORMAT)
            )
            .as_str(),
        );
        s.push_str(
            format!(
                "Sunrise is at: {}\n",
                DateTime::<Local>::from(update.sunrise).format(DATE_FORMAT)
            )
            .as_str(),
        );
    }
    cursive.call_on_name(STATUS_TEXT, |x: &mut TextView| {
        x.set_content(s);
    });
}

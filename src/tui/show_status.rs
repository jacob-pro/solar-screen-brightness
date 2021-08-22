use crate::controller::apply::{ApplyResult, SolarAndBrightnessResults};
use chrono::{DateTime, Local};
use cursive::traits::Nameable;
use cursive::views::{Button, Dialog, DummyView, LinearLayout, ScrollView, TextView};
use cursive::Cursive;

const STATUS_TEXT: &str = "SHOW_STATUS_TEXT";

pub fn create<F>(completion: F) -> Dialog
where
    F: 'static + Fn(&mut Cursive),
{
    Dialog::around(
        LinearLayout::vertical()
            .child(DummyView)
            .child(ScrollView::new(
                TextView::new("null").with_name(STATUS_TEXT),
            ))
            .child(DummyView)
            .child(Button::new("Back", completion)),
    )
    .title("Current Status")
}

const DATE_FORMAT: &str = "%H:%M %P";

pub fn status_update(cursive: &mut Cursive, update: ApplyResult) {
    let mut s = String::new();

    match update {
        ApplyResult::None => {
            s.push_str("Waiting for brightness controller to start...");
        }
        ApplyResult::Error(e) => {
            s.push_str("Error applying brightness:\n");
            s.push_str(format!("{}", e).as_str());
        }
        ApplyResult::Skipped(b) => {
            s.push_str("(Currently disabled)\n\n");
            push_brightness_results(&mut s, b);
        }
        ApplyResult::Applied(b, e) => {
            push_brightness_results(&mut s, b);
            if e.len() > 0 {
                s.push_str("\nErrors occurred during applying:\n");
            }
            for e in &*e {
                s.push_str(format!("{}\n", e).as_str());
            }
        }
    };

    cursive.call_on_name(STATUS_TEXT, |x: &mut TextView| {
        x.set_content(s);
    });
}

pub fn push_brightness_results(s: &mut String, res: SolarAndBrightnessResults) {
    s.push_str(format!("Base brightness: {}%\n", res.base_brightness).as_str());
    s.push_str(
        format!(
            "Changes at: {}\n",
            DateTime::<Local>::from(res.expiry).format(DATE_FORMAT)
        )
        .as_str(),
    );
    if res.visible {
        s.push_str(
            format!(
                "Sunrise was at: {}\n",
                DateTime::<Local>::from(res.sunrise).format(DATE_FORMAT)
            )
            .as_str(),
        );
        s.push_str(
            format!(
                "Sunset is at: {}\n",
                DateTime::<Local>::from(res.sunset).format(DATE_FORMAT)
            )
            .as_str(),
        );
    } else {
        s.push_str(
            format!(
                "Sunset was at: {}\n",
                DateTime::<Local>::from(res.sunset).format(DATE_FORMAT)
            )
            .as_str(),
        );
        s.push_str(
            format!(
                "Sunrise is at: {}\n",
                DateTime::<Local>::from(res.sunrise).format(DATE_FORMAT)
            )
            .as_str(),
        );
    }
}

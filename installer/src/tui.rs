use crate::common::APP_NAME;
use crate::cursive::align::HAlign;
use crate::cursive::event::Event;
use crate::cursive::views::{
    Dialog, DummyView, HideableView, LinearLayout, ScrollView, SelectView,
};
use crate::cursive::{Cursive, CursiveExt};
use enum_iterator::IntoEnumIterator;

pub fn launch_cursive() -> anyhow::Result<()> {
    let mut siv = Cursive::default();
    siv.clear_global_callbacks(Event::CtrlChar('c'));
    siv.add_layer(create());
    siv.run();
    Ok(())
}

#[derive(IntoEnumIterator)]
pub enum MainMenuChoice {
    Install = 0,
    Uninstall,
    Exit,
}

impl MainMenuChoice {
    fn title(&self) -> &'static str {
        match self {
            MainMenuChoice::Install => "Install",
            MainMenuChoice::Uninstall => "Uninstall",
            MainMenuChoice::Exit => "Exit",
        }
    }
}

pub fn create() -> HideableView<Dialog> {
    let mut select = SelectView::new().h_align(HAlign::Left).autojump();
    for (idx, item) in MainMenuChoice::into_enum_iter().enumerate() {
        select.add_item(format!("{} {}", idx + 1, item.title()), item);
    }
    select.set_on_submit(on_submit);
    HideableView::new(
        Dialog::around(ScrollView::new(
            LinearLayout::vertical().child(DummyView).child(select),
        ))
        .title(format!("{} Installer", APP_NAME)),
    )
}

fn on_submit(cursive: &mut Cursive, choice: &MainMenuChoice) {
    match choice {
        MainMenuChoice::Install => {
            cursive.add_layer(Dialog::text("Installing..."));
            if let Err(e) = crate::install::install() {
                log::error!("{:#}", e);
                cursive.pop_layer();
                cursive.add_layer(Dialog::info(format!("\n{:#}", e)).title("Installation Failed"))
            } else {
                cursive.pop_layer();
                cursive.add_layer(Dialog::text("Success").button("Launch app", |cursive| {
                    if let Err(e) = crate::install::launch() {
                        log::error!("{:#}", e);
                    }
                    cursive.quit();
                }))
            }
        }
        MainMenuChoice::Uninstall => {
            cursive.add_layer(Dialog::text("Uninstalling..."));
            if let Err(e) = crate::uninstall::uninstall() {
                log::error!("{:#}", e);
                cursive.pop_layer();
                cursive.add_layer(Dialog::info(format!("\n{:#}", e)).title("Uninstallation Failed"))
            } else {
                cursive.pop_layer();
                cursive.add_layer(Dialog::text("Success").button("Exit", |cursive| {
                    cursive.quit();
                }))
            }
        }
        MainMenuChoice::Exit => {
            cursive.quit();
        }
    }
}

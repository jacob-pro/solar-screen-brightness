use crate::assets::Assets;
use crate::controller::BrightnessController;
use qt_core::{qs, SlotNoArgs};
use qt_gui::{QIcon, QPixmap};
use qt_widgets::{QAction, QApplication, QMenu, QSystemTrayIcon, SlotOfActivationReason};
use std::process::Command;

// Blocking call, runs on this thread
pub fn run(controller: BrightnessController) {
    QApplication::init(|_| unsafe {
        assert!(QSystemTrayIcon::is_system_tray_available());
        let tray = QSystemTrayIcon::new();

        // Set up the icon
        let mut asset = Assets::get("icon-256.png")
            .expect("Icon missing")
            .into_owned();
        let pixmap = QPixmap::new();
        assert!(pixmap.load_from_data_uchar_uint(asset.as_ptr(), asset.len() as u32));
        let qicon = QIcon::new();
        qicon.add_pixmap_1a(&pixmap);
        tray.set_icon(&qicon);

        // Set up the menu
        let menu = QMenu::new();
        let action = menu.add_action_q_string(&qs("Open console"));
        tray.set_context_menu(&menu);

        let c2 = controller.clone();
        action.triggered().connect(&SlotNoArgs::new(&tray, move || {
            let t = Command::new("gnome-terminal")
                .spawn()
                .expect("failed to execute process");

            // let x = c2.clone();
            // std::thread::spawn(move || {
            //     crate::tui::run(Box::new(|s| {
            //
            //     }), x);
            // });
            // println!("helloooo");
        }));

        tray.show();
        QApplication::exec()
    });
}

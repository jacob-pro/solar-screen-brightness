use crate::assets::Assets;
// use crate::console::Console;
use crate::controller::{BrightnessController, StateRef};
use qt_widgets::{QSystemTrayIcon, QApplication, QMenu, SlotOfActivationReason, QAction};
use qt_gui::{QIcon, QPixmap};
use qt_core::{SlotNoArgs, qs};


#[derive(Debug)]
pub enum TrayMessage {
    CloseConsole,
    ExitApplication,
}

pub type TrayMessageSender = Box<dyn Fn(TrayMessage) + Send + Sync>;


// Blocking call, runs on this thread
pub fn run(controller: &BrightnessController) {
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

        let c2 = controller.state.clone();
        action.triggered().connect(&SlotNoArgs::new(&tray, move || {
            let x = c2.clone();
            std::thread::spawn(move || {
                crate::tui::run(Box::new(|s| {

                }), x);
            });
            println!("helloooo");
        }));

        tray.show();
        QApplication::exec()
    });
}

//                 match &app.console {
//                     Some(c) => {
//                         c.show();
//                     }
//                     None => {
//                         app.console = Some(Console::create(
//                             Box::new(move |msg| msg.send(hwnd)),
//                             app.state.clone(),
//                         ));
//                     }
//                 }
//             }
//             _ => {}
//         },
//         CLOSE_CONSOLE_MSG => {
//             let app = get_user_data::<WindowData>(&hwnd).unwrap();
//             app.console.as_ref().unwrap().hide();
//         }
//         EXIT_APPLICATION_MSG => {
//             PostQuitMessage(0);
//         }


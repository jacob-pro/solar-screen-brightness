use crate::assets::Assets;
use crate::controller::BrightnessController;
use cpp_core::{Ptr, StaticUpcast};
use qt_core::{qs, slot, QBox, QObject, QPtr, SlotOfBool};
use qt_gui::{QIcon, QPixmap};
use qt_widgets::{QAction, QApplication, QMenu, QSystemTrayIcon};
use std::process::Command;
use std::rc::Rc;

struct TrayApplication {
    tray: QBox<QSystemTrayIcon>,
    menu: QBox<QMenu>,
    action: QPtr<QAction>,
}

impl StaticUpcast<QObject> for TrayApplication {
    unsafe fn static_upcast(ptr: Ptr<Self>) -> Ptr<QObject> {
        ptr.tray.as_ptr().static_upcast()
    }
}

impl TrayApplication {
    unsafe fn new() -> Rc<Self> {
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
        tray.show();

        let this = Rc::new(Self { tray, menu, action });
        this.action
            .triggered()
            .connect(&this.slot_on_action_triggered());
        this
    }

    #[slot(SlotOfBool)]
    unsafe fn on_action_triggered(self: &Rc<Self>, _: bool) {
        println!("console opened");
    }
}

// Blocking call, runs on this thread
pub fn run(controller: BrightnessController) {
    QApplication::init(|_| unsafe {
        assert!(QSystemTrayIcon::is_system_tray_available());
        let _tray = TrayApplication::new();
        QApplication::exec()
    });
}

pub struct TrayApplicationHandle;

impl TrayApplicationHandle {
    pub fn close_console(&self) {}

    pub fn exit_application(&self) {}
}

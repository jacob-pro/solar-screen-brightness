use crate::assets::Assets;
use crate::console::Console;
use crate::controller::BrightnessController;
use crate::lock::{ApplicationLock, ShowConsoleWatcher};
use crate::tray::TrayApplicationHandle;
use cpp_core::{Ptr, StaticUpcast};
use cursive::Cursive;
use qt_core::{qs, slot, QBox, QObject, QPtr, SlotOfBool};
use qt_gui::{QIcon, QPixmap};
use qt_widgets::{QAction, QApplication, QMenu, QSystemTrayIcon};
use std::rc::Rc;
use std::sync::mpsc::{sync_channel, SyncSender};
use std::sync::Arc;

#[allow(unused)]
struct TrayApplication {
    tray: QBox<QSystemTrayIcon>,
    menu: QBox<QMenu>,
    action: QPtr<QAction>,
    sender: SyncSender<Message>,
}

impl StaticUpcast<QObject> for TrayApplication {
    unsafe fn static_upcast(ptr: Ptr<Self>) -> Ptr<QObject> {
        ptr.tray.as_ptr().static_upcast()
    }
}

enum Message {
    ShowConsole,
    CloseConsole,
    ExitApplication,
}

impl TrayApplication {
    unsafe fn new(sender: SyncSender<Message>) -> Rc<Self> {
        let tray_icon = QSystemTrayIcon::new();

        // Set up the icon
        let asset = Assets::get("icon-256.png")
            .expect("Icon missing")
            .into_owned();
        let pixmap = QPixmap::new();
        assert!(pixmap.load_from_data_uchar_uint(asset.as_ptr(), asset.len() as u32));
        let qicon = QIcon::new();
        qicon.add_pixmap_1a(&pixmap);
        tray_icon.set_icon(&qicon);

        // Set up the menu
        let menu = QMenu::new();
        let action = menu.add_action_q_string(&qs("Open console"));
        tray_icon.set_context_menu(&menu);
        tray_icon.show();

        let this = Rc::new(Self {
            tray: tray_icon,
            menu,
            action,
            sender,
        });

        this.action
            .triggered()
            .connect(&this.slot_on_action_triggered());

        this
    }

    #[slot(SlotOfBool)]
    unsafe fn on_action_triggered(self: &Rc<Self>, _: bool) {
        self.sender.send(Message::ShowConsole).unwrap();
    }
}

pub fn run(controller: Arc<BrightnessController>, _: ApplicationLock, launch_console: bool) {
    let (sync_sender, receiver) = sync_channel(0);

    let ss2 = sync_sender.clone();
    let _watcher = ShowConsoleWatcher::start(move || {
        ss2.send(Message::ShowConsole).unwrap();
    });

    let handle = TrayApplicationHandle(Handle(sync_sender.clone()));
    let mut console = Console::new(handle, controller);
    if launch_console {
        console.show();
    }

    let ss3 = sync_sender.clone();
    std::thread::spawn(move || {
        QApplication::init(|_| unsafe {
            assert!(QSystemTrayIcon::is_system_tray_available());
            let _tray = TrayApplication::new(ss3);
            QApplication::exec()
        });
    });

    loop {
        match receiver.recv().unwrap() {
            Message::CloseConsole => {
                console.hide();
            }
            Message::ExitApplication => {
                break;
            }
            Message::ShowConsole => {
                console.show();
            }
        }
    }
}

#[derive(Clone)]
pub(super) struct Handle(SyncSender<Message>);

impl Handle {
    pub(super) fn close_console(&self, cursive: &mut Cursive) {
        self.0.send(Message::CloseConsole).unwrap();
        cursive.quit();
    }

    pub(super) fn exit_application(&self) {
        self.0.send(Message::ExitApplication).unwrap();
    }
}

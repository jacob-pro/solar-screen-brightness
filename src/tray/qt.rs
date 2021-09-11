use crate::assets::Assets;
use crate::console::Console;
use crate::controller::BrightnessController;
use crate::lock::ApplicationLock;
use crate::tray::TrayApplicationHandle;
use cpp_core::{Ptr, StaticUpcast};
use cursive::Cursive;
use qt_core::{qs, slot, QBox, QCoreApplication, QObject, QPtr, QTimer, SlotNoArgs, SlotOfBool};
use qt_gui::{QIcon, QPixmap};
use qt_widgets::{QAction, QApplication, QMenu, QSystemTrayIcon};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TryRecvError};

#[allow(unused)]
struct TrayApplication {
    tray: QBox<QSystemTrayIcon>,
    menu: QBox<QMenu>,
    action: QPtr<QAction>,
    rx: Receiver<Message>,
    console: RefCell<Console>,
    lock: ApplicationLock,
}

impl StaticUpcast<QObject> for TrayApplication {
    unsafe fn static_upcast(ptr: Ptr<Self>) -> Ptr<QObject> {
        ptr.tray.as_ptr().static_upcast()
    }
}

impl TrayApplication {
    unsafe fn new(
        controller: BrightnessController,
        lock: ApplicationLock,
        launch_console: bool,
    ) -> Rc<Self> {
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

        let (tx, rx) = sync_channel::<Message>(0);
        let handle = TrayApplicationHandle(Handle(tx));
        let mut console = Console::new(handle, controller);
        if launch_console {
            console.show();
        }

        let this = Rc::new(Self {
            tray: tray_icon,
            menu,
            action,
            rx,
            console: RefCell::new(console),
            lock,
        });

        let timer = QTimer::new_1a(&this.tray);
        this.action
            .triggered()
            .connect(&this.slot_on_action_triggered());
        timer.timeout().connect(&this.slot_on_event_loop());
        timer.start_1a(0);

        this
    }

    #[slot(SlotOfBool)]
    unsafe fn on_action_triggered(self: &Rc<Self>, _: bool) {
        self.console.borrow_mut().show();
    }

    #[slot(SlotNoArgs)]
    unsafe fn on_event_loop(self: &Rc<Self>) {
        if self.lock.should_show_console() {
            self.console.borrow_mut().show();
        }
        match self.rx.try_recv() {
            Ok(message) => match message {
                Message::CloseConsole => {
                    self.console.borrow_mut().hide();
                }
                Message::ExitApplication => {
                    QCoreApplication::quit();
                }
            },
            Err(e) => match e {
                TryRecvError::Empty => {}
                TryRecvError::Disconnected => {
                    panic!("Tray Handle disconnected");
                }
            },
        }
    }
}

pub fn run(controller: BrightnessController, lock: ApplicationLock, launch_console: bool) {
    QApplication::init(|_| unsafe {
        assert!(QSystemTrayIcon::is_system_tray_available());
        let _tray = TrayApplication::new(controller, lock, launch_console);
        QApplication::exec()
    });
}

enum Message {
    CloseConsole,
    ExitApplication,
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

pub fn show_console_in_another_process() {
    log::error!("show_console_in_another_process() is not yet implemented for Unix");
    // TODO: Implement this for Unix
}

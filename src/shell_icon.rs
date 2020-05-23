use winapi::um::winuser::{DefWindowProcW, CreateWindowExW, WNDCLASSW, RegisterClassW, CW_USEDEFAULT, WS_OVERLAPPEDWINDOW, CreateIconFromResource, WM_APP, WM_LBUTTONUP, WM_RBUTTONUP};
use winapi::shared::minwindef::{UINT, WPARAM, LPARAM, LRESULT, TRUE, HINSTANCE, LOWORD, DWORD};
use winapi::shared::windef::{HWND, HMENU, HICON};
use winapi::um::shellapi::{NIM_ADD, NOTIFYICONDATAW, NOTIFYICONDATAW_u, Shell_NotifyIconW, NIF_MESSAGE, NIF_ICON, NIF_TIP, NOTIFYICON_VERSION_4, NIM_SETVERSION, NIN_SELECT};
use winapi::shared::ntdef::{NULL};
use winapi::um::libloaderapi::GetModuleHandleW;

use crate::assets::Assets;
use crate::str_ext::StrExt;
use winapi::um::winnt::LPCWSTR;

const CALLBACK_MSG: UINT = WM_APP + 1;

pub fn create_shell_icon() {
    unsafe {
        let hinstance = GetModuleHandleW( NULL as LPCWSTR );
        if hinstance == NULL as HINSTANCE { panic!("Get hinstance failed") };

        let mut window_class: WNDCLASSW =  std::mem::MaybeUninit::zeroed().assume_init();
        window_class.lpfnWndProc = Some(window_procedure);
        window_class.hInstance = hinstance;
        window_class.lpszClassName = "TrayHolder".to_wide().as_ptr();
        let atom = RegisterClassW(&window_class);
        if atom == 0 { panic!("Register window class failed") };

        let hwnd = CreateWindowExW(
            0,
            atom as *const u16,
            "tray".to_wide().as_ptr(),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            NULL as HWND,
            NULL as HMENU,
            hinstance,
            NULL);
        if hwnd == NULL as HWND { panic!("Create window failed") };

        let mut asset = Assets::get("icon-256.png").expect("Icon missing").into_owned();
        let hicon = CreateIconFromResource(asset.as_mut_ptr(), asset.len() as u32, TRUE, 0x00030000);
        if hicon == NULL as HICON { panic!("Failed to create icon") };

        let mut data: NOTIFYICONDATAW =  std::mem::MaybeUninit::zeroed().assume_init();
        let mut name = "Solar Screen Brightness".to_wide();
        name.resize(data.szTip.len(), 0);
        let bytes = &name[..data.szTip.len()];
        data.hWnd = hwnd;
        data.hIcon = hicon;
        data.uCallbackMessage = CALLBACK_MSG;
        data.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        data.szTip.copy_from_slice(bytes);
        if Shell_NotifyIconW(NIM_ADD, &mut data) != TRUE { panic!("Error creating tray icon") };
    }
}

unsafe extern "system" fn window_procedure(hwnd: HWND, msg: UINT, w_param : WPARAM, l_param: LPARAM) -> LRESULT {
    match msg {
        CALLBACK_MSG => {
            match LOWORD(l_param as DWORD) as u32 {
                WM_LBUTTONUP | WM_RBUTTONUP  => {
                    println!("Clicked");
                }
                _ => {}
            }
        }
        _ => {}
    }
    return DefWindowProcW( hwnd , msg , w_param , l_param );
}

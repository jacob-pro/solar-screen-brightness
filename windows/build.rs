fn main() {
    windows::build! {
        Windows::Win32::Foundation::*,
        Windows::Win32::UI::WindowsAndMessaging::*,
        Windows::Win32::UI::Shell::*,
        Windows::Win32::UI::KeyboardAndMouseInput::*,
        Windows::Win32::Security::{GetTokenInformation, TOKEN_STATISTICS},
        Windows::Win32::System::Console::*,
        Windows::Win32::System::Threading::*,
        Windows::Win32::System::LibraryLoader::GetModuleHandleW,
        Windows::Win32::System::RemoteDesktop::WTSRegisterSessionNotification,
        Windows::Win32::System::Diagnostics::Debug::{GetLastError, SetLastError, WIN32_ERROR},
    };
}

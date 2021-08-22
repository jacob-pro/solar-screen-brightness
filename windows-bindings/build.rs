fn main() {
    windows::build! {
        Windows::Win32::Foundation::*,
        Windows::Win32::UI::WindowsAndMessaging::*,
        Windows::Win32::System::LibraryLoader::GetModuleHandleW,
    };
}

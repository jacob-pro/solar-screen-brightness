use winapi::um::winuser::{EnumDisplayMonitors, GetMonitorInfoW, MONITORINFOEXW, LPMONITORINFO, EnumDisplayDevicesW};
use winapi::shared::minwindef::{BOOL, LPARAM, TRUE, FALSE, DWORD};
use winapi::shared::windef::{LPRECT, LPCRECT, HDC, HMONITOR};
use winapi::shared::ntdef::NULL;
use winapi::um::physicalmonitorenumerationapi::{GetNumberOfPhysicalMonitorsFromHMONITOR, GetPhysicalMonitorsFromHMONITOR, PHYSICAL_MONITOR, DestroyPhysicalMonitors};
use winapi::um::highlevelmonitorconfigurationapi::SetMonitorBrightness;
use winapi::um::winnt::LPCWSTR;
use winapi::um::wingdi::DISPLAY_DEVICEW;
use crate::wide::wide_to_str;


pub fn load_monitors() -> Vec<Monitor> {
    unsafe extern "system" fn enum_monitors(arg1: HMONITOR, _arg2: HDC, _arg3: LPRECT, arg4: LPARAM) -> BOOL {
        let monitors: &mut Vec<HMONITOR> = &mut *(arg4 as *mut Vec<HMONITOR>);
        monitors.push(arg1);
        return TRUE
    }
    unsafe {
        let mut hmonitors = Vec::<HMONITOR>::new();
        assert_ne!(EnumDisplayMonitors(NULL as HDC,
                                       NULL as LPCRECT,
                                       Some(enum_monitors),
                                       &mut hmonitors as *mut _ as isize
        ), FALSE);
        hmonitors.into_iter().map(|x| Monitor::new(x)).collect()
    }
}

#[allow(dead_code)]
pub struct Monitor {
    handle: HMONITOR,
    physical_monitors: Vec<PHYSICAL_MONITOR>,
    device_name: String,
    device_string: Option<String>,
}

impl Monitor {

    unsafe fn new(handle: HMONITOR) -> Self {

        let mut count: DWORD = 0;
        GetNumberOfPhysicalMonitorsFromHMONITOR(handle, &mut count);
        let mut physical = Vec::with_capacity(count as usize);
        if GetPhysicalMonitorsFromHMONITOR(handle, count, physical.as_mut_ptr()) == TRUE {
            physical.set_len(count as usize);
        }

        let mut info: MONITORINFOEXW = std::mem::MaybeUninit::zeroed().assume_init();
        info.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
        let pointer = &mut info as *mut _;
        assert_ne!(GetMonitorInfoW(handle, pointer as LPMONITORINFO), 0);

        let mut device: DISPLAY_DEVICEW = std::mem::MaybeUninit::zeroed().assume_init();
        device.cb = std::mem::size_of::<DISPLAY_DEVICEW>() as u32;
        let device_string = if EnumDisplayDevicesW(&info.szDevice as LPCWSTR, 0, &mut device, 0) != 0 {
            Some(wide_to_str(&device.DeviceString).unwrap())
        } else {
            None
        };

        Monitor {
            handle,
            physical_monitors: physical,
            device_name: wide_to_str(&info.szDevice).unwrap(),
            device_string,
        }
    }

    pub fn set_brightness(&self, brightness: u32) {
        unsafe {
            for p in &self.physical_monitors {
                SetMonitorBrightness(p.hPhysicalMonitor, brightness as DWORD);
            }
        }
    }
}

impl Drop for Monitor {
    fn drop(&mut self) {
        unsafe {
            assert_eq!(TRUE, DestroyPhysicalMonitors(self.physical_monitors.len() as u32, self.physical_monitors.as_mut_ptr()));
        }
    }
}

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Gdi::ValidateRect,
    Win32::System::LibraryLoader::GetModuleHandleA, Win32::UI::WindowsAndMessaging::*,
};

fn main() -> Result<()> {

    unsafe {
        //MessageBoxW(None, "Hello", "World", MB_OK);
        let instance = GetModuleHandleA(None);
        debug_assert!(instance.0 != 0);

        let window_class = "window";
        let wc = WNDCLASSW {
            hCursor: LoadCursorW(None, IDC_ARROW),
            hInstance: instance,
            lpszClassName: PCWSTR(to_wstring("window").as_ptr()),
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            ..Default::default()
        };

        let atom = RegisterClassW(&wc);
        debug_assert!(atom != 0);

        CreateWindowExW(
            Default::default(),
            window_class,
            "This is a sample wit unicode window: ÑñÑñ",
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            instance,
            std::ptr::null(),
        );

        let mut message = MSG::default();

        while GetMessageA(&mut message, HWND(0), 0, 0).into() {
            DispatchMessageA(&message);
        }

        Ok(())
    }
}
fn to_wstring(str: &str) -> Vec<u16> {
    let v: Vec<u16> = OsStr::new(str)
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect();
    println!("{:?}", v);
    v
}
extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message as u32 {
            WM_PAINT => {
                println!("WM_PAINT");
                ValidateRect(window, std::ptr::null());
                LRESULT(0)
            }
            WM_DESTROY => {
                println!("WM_DESTROY");
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(window, message, wparam, lparam),
        }
    }
}

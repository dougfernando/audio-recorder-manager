// Native Windows splash screen using Win32 API
// This shows instantly without WebView2 dependency

#![cfg(windows)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;

static mut SPLASH_HWND: Option<HWND> = None;

pub struct SplashScreen {
    is_closed: Arc<AtomicBool>,
}

impl SplashScreen {
    pub fn new() -> Result<Self> {
        let is_closed = Arc::new(AtomicBool::new(false));
        let is_closed_clone = is_closed.clone();

        // Create splash window on a separate thread
        std::thread::spawn(move || {
            if let Err(e) = create_splash_window(is_closed_clone) {
                tracing::error!("Failed to create splash window: {}", e);
            }
        });

        // Give the window thread time to create the window
        std::thread::sleep(std::time::Duration::from_millis(50));

        Ok(SplashScreen { is_closed })
    }

    pub fn close(&self) {
        self.is_closed.store(true, Ordering::Relaxed);

        unsafe {
            if let Some(hwnd) = SPLASH_HWND {
                let _ = PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
            }
        }
    }
}

impl Drop for SplashScreen {
    fn drop(&mut self) {
        self.close();
    }
}

fn create_splash_window(is_closed: Arc<AtomicBool>) -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;

        let class_name = w!("AudioRecorderSplash");

        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            hInstance: instance.into(),
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hbrBackground: CreateSolidBrush(COLORREF(0x00F5F5F7)), // #F5F5F7
            lpszClassName: class_name,
            ..Default::default()
        };

        RegisterClassW(&wc);

        // Get screen dimensions for centering
        let screen_width = GetSystemMetrics(SM_CXSCREEN);
        let screen_height = GetSystemMetrics(SM_CYSCREEN);

        let width = 400;
        let height = 200;
        let x = (screen_width - width) / 2;
        let y = (screen_height - height) / 2;

        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            class_name,
            w!("Audio Recorder Manager"),
            WS_POPUP | WS_VISIBLE,
            x,
            y,
            width,
            height,
            None,
            None,
            instance,
            None,
        )?;

        SPLASH_HWND = Some(hwnd);

        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = UpdateWindow(hwnd);

        // Message loop
        let mut msg = MSG::default();
        while !is_closed.load(Ordering::Relaxed) {
            if PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                if msg.message == WM_QUIT {
                    break;
                }
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            } else {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }

        let _ = DestroyWindow(hwnd);
        SPLASH_HWND = None;
    }

    Ok(())
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);

            if !hdc.is_invalid() {
                // Set background
                let mut rect = RECT::default();
                let _ = GetClientRect(hwnd, &mut rect);

                let brush = CreateSolidBrush(COLORREF(0x00F5F5F7)); // #F5F5F7
                FillRect(hdc, &rect, brush);
                let _ = DeleteObject(brush);

                // Draw text
                SetBkMode(hdc, TRANSPARENT);
                SetTextColor(hdc, COLORREF(0x00333333)); // Dark gray text

                // Title
                let title = "Audio Recorder Manager";
                let mut title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();

                let mut title_rect = rect;
                title_rect.top = 60;
                title_rect.bottom = 90;

                let font = CreateFontW(
                    24,                           // Height
                    0,                            // Width
                    0,                            // Escapement
                    0,                            // Orientation
                    FW_SEMIBOLD.0 as i32,        // Weight
                    0,                            // Italic
                    0,                            // Underline
                    0,                            // StrikeOut
                    DEFAULT_CHARSET.0 as u32,     // CharSet
                    OUT_DEFAULT_PRECIS.0 as u32,  // OutputPrecision
                    CLIP_DEFAULT_PRECIS.0 as u32, // ClipPrecision
                    DEFAULT_QUALITY.0 as u32,     // Quality
                    (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32, // PitchAndFamily
                    w!("Segoe UI"),               // FaceName
                );

                let old_font = SelectObject(hdc, font);
                DrawTextW(
                    hdc,
                    &mut title_wide,
                    &mut title_rect,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                );
                SelectObject(hdc, old_font);
                let _ = DeleteObject(font);

                // Loading text
                let loading = "Loading...";
                let mut loading_wide: Vec<u16> = loading.encode_utf16().chain(std::iter::once(0)).collect();

                let mut loading_rect = rect;
                loading_rect.top = 110;
                loading_rect.bottom = 130;

                let loading_font = CreateFontW(
                    16,
                    0,
                    0,
                    0,
                    FW_NORMAL.0 as i32,
                    0,
                    0,
                    0,
                    DEFAULT_CHARSET.0 as u32,
                    OUT_DEFAULT_PRECIS.0 as u32,
                    CLIP_DEFAULT_PRECIS.0 as u32,
                    DEFAULT_QUALITY.0 as u32,
                    (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32,
                    w!("Segoe UI"),
                );

                let old_font = SelectObject(hdc, loading_font);
                SetTextColor(hdc, COLORREF(0x00666666)); // Lighter gray
                DrawTextW(
                    hdc,
                    &mut loading_wide,
                    &mut loading_rect,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                );
                SelectObject(hdc, old_font);
                let _ = DeleteObject(loading_font);

                let _ = EndPaint(hwnd, &ps);
            }

            LRESULT(0)
        }
        WM_CLOSE | WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

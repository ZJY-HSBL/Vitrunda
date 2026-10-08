use crate::config::Config;
use std::{
    ffi::OsStr,
    os::windows::ffi::OsStrExt,
    sync::{Mutex, OnceLock},
    time::Instant,
};
use windows::{
    core::{w, PCWSTR, Result},
    Win32::{
        Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::{
            Dwm::{DwmEnableBlurBehindWindow, DWM_BB_ENABLE, DWM_BLURBEHIND},
            Gdi::{
                BeginPaint, CreateRoundRectRgn, CreateSolidBrush, DeleteObject, EndPaint,
                FillRect, HBRUSH, HRGN, PAINTSTRUCT, SetWindowRgn,
            },
        },
        System::{
            LibraryLoader::GetModuleHandleW,
            Registry::{
                RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW, HKEY,
                HKEY_CURRENT_USER, KEY_SET_VALUE, REG_SZ,
            },
        },
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetMessageW,
            GetSystemMetrics, KillTimer, LoadCursorW, PostQuitMessage, RegisterClassW,
            SetForegroundWindow, SetLayeredWindowAttributes, SetTimer, SetWindowPos,
            ShowWindow, TranslateMessage, CS_HREDRAW, CS_VREDRAW,
            IDC_ARROW, LWA_ALPHA, MSG, SM_CXSCREEN, SM_CYSCREEN,
            SW_HIDE, SW_SHOW, SWP_NOACTIVATE, SWP_SHOWWINDOW,
            WM_DESTROY, WM_KEYDOWN, WM_LBUTTONDOWN, WM_PAINT, WM_TIMER,
            WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
            WS_POPUP,
        },
    },
};

const CLASS_NAME: PCWSTR = w!("VitrundaWindowClass");
const OVERLAY_TITLE: PCWSTR = w!("VitrundaOverlay");
const HOT_TITLE: PCWSTR = w!("VitrundaHotCorner");
const TIMER_ID: usize = 0xA11CE;
const TIMER_MS: u32 = 8;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    Opening,
    Closing,
}

struct Runtime {
    config: Config,
    overlay: HWND,
    hot: HWND,
    open: bool,
    animating: bool,
    direction: Direction,
    started: Instant,
    from_progress: f32,
    progress: f32,
    transition_ms: f32,
}

unsafe impl Send for Runtime {}

static RUNTIME: OnceLock<Mutex<Runtime>> = OnceLock::new();

pub fn run() -> Result<()> {
    unsafe {
        let config = Config::load();
        let _gpu_context = super::gpu::GpuContext::try_create();
        let _ = config.save();
        set_autostart(config.autostart);

        let instance = GetModuleHandleW(None)?;
        register_window_class(instance.into())?;

        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let corner = config.hot_corner_size.max(8);

        let overlay = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_LAYERED,
            CLASS_NAME,
            OVERLAY_TITLE,
            WS_POPUP,
            screen_w - corner,
            0,
            corner,
            corner,
            None,
            None,
            instance,
            None,
        )?;

        SetLayeredWindowAttributes(overlay, COLORREF(0), config.glass_alpha, LWA_ALPHA)?;
        enable_blur(overlay);
        ShowWindow(overlay, SW_HIDE);

        let hot = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_LAYERED | WS_EX_NOACTIVATE,
            CLASS_NAME,
            HOT_TITLE,
            WS_POPUP,
            screen_w - corner,
            0,
            corner,
            corner,
            None,
            None,
            instance,
            None,
        )?;

        SetLayeredWindowAttributes(hot, COLORREF(0), 1, LWA_ALPHA)?;
        ShowWindow(hot, SW_SHOW);

        let initial_transition_ms = config.animation_ms.max(1) as f32;
        let _ = RUNTIME.set(Mutex::new(Runtime {
            config,
            overlay,
            hot,
            open: false,
            animating: false,
            direction: Direction::Opening,
            started: Instant::now(),
            from_progress: 0.0,
            progress: 0.0,
            transition_ms: initial_transition_ms,
        }));

        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).as_bool() {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
    Ok(())
}

unsafe fn register_window_class(instance: HINSTANCE) -> Result<()> {
    let cursor = LoadCursorW(None, IDC_ARROW)?;
    let class = WNDCLASSW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(window_proc),
        hInstance: instance,
        hCursor: cursor,
        hbrBackground: HBRUSH::default(),
        lpszClassName: CLASS_NAME,
        ..Default::default()
    };

    if RegisterClassW(&class) == 0 {
        return Err(windows::core::Error::from_win32());
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
        WM_LBUTTONDOWN => {
            if is_hot_window(hwnd) {
                toggle();
                return LRESULT(0);
            }
        }
        WM_KEYDOWN => {
            if wparam.0 as u32 == 0x1B {
                if let Some(lock) = RUNTIME.get() {
                    if let Ok(runtime) = lock.lock() {
                        if runtime.config.close_on_escape && (runtime.open || runtime.animating) {
                            drop(runtime);
                            begin_animation(Direction::Closing);
                        }
                    }
                }
                return LRESULT(0);
            }
        }
        WM_TIMER => {
            if wparam.0 == TIMER_ID {
                tick_animation();
                return LRESULT(0);
            }
        }
        WM_PAINT => {
            paint_glass(hwnd);
            return LRESULT(0);
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            return LRESULT(0);
        }
        _ => {}
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

unsafe fn is_hot_window(hwnd: HWND) -> bool {
    let Some(lock) = RUNTIME.get() else { return false };
    let Ok(runtime) = lock.lock() else { return false };
    hwnd == runtime.hot
}

unsafe fn toggle() {
    let Some(lock) = RUNTIME.get() else { return };
    let Ok(runtime) = lock.lock() else { return };
    let target = if runtime.open || (runtime.animating && runtime.direction == Direction::Opening) {
        Direction::Closing
    } else {
        Direction::Opening
    };
    drop(runtime);
    begin_animation(target);
}

unsafe fn begin_animation(direction: Direction) {
    let Some(lock) = RUNTIME.get() else { return };
    let Ok(mut runtime) = lock.lock() else { return };

    runtime.direction = direction;
    runtime.animating = true;
    runtime.started = Instant::now();
    runtime.from_progress = runtime.progress;
    let destination: f32 = if direction == Direction::Opening { 1.0 } else { 0.0 };
    runtime.transition_ms = runtime.config.animation_ms.max(1) as f32
        * (destination - runtime.progress).abs().max(0.05);

    if direction == Direction::Opening {
        ShowWindow(runtime.overlay, SW_SHOW);
        let _ = SetForegroundWindow(runtime.overlay);
    }

    SetTimer(runtime.overlay, TIMER_ID, TIMER_MS, None);
}

unsafe fn tick_animation() {
    let Some(lock) = RUNTIME.get() else { return };
    let Ok(mut runtime) = lock.lock() else { return };
    if !runtime.animating {
        return;
    }

    let screen_w = GetSystemMetrics(SM_CXSCREEN).max(1);
    let screen_h = GetSystemMetrics(SM_CYSCREEN).max(1);
    let corner = runtime.config.hot_corner_size.max(8);
    let duration = runtime.transition_ms;
    let raw = (runtime.started.elapsed().as_millis() as f32 / duration).clamp(0.0, 1.0);

    let target = if runtime.direction == Direction::Opening { 1.0 } else { 0.0 };
    let progress = runtime.from_progress
        + (target - runtime.from_progress) * liquid_ease(raw);
    runtime.progress = progress;

    let width_p = springish(progress);
    let height_p = springish(((progress - 0.035) / 0.965).clamp(0.0, 1.0));

    let width = lerp_i32(corner, screen_w, width_p);
    let height = lerp_i32(corner, screen_h, height_p);
    let x = screen_w - width;
    let y = 0;

    let _ = SetWindowPos(
        runtime.overlay,
        None,
        x,
        y,
        width,
        height,
        SWP_SHOWWINDOW,
    );

    let radius = liquid_corner_radius(width, height, progress);
    if radius > 0 {
        let region: HRGN = CreateRoundRectRgn(0, 0, width + 1, height + 1, radius, radius);
        if !region.is_invalid() {
            if SetWindowRgn(runtime.overlay, region, true) == 0 {
                let _ = DeleteObject(region);
            }
        }
    } else {
        let _ = SetWindowRgn(runtime.overlay, None, true);
    }

    let _ = SetWindowPos(
        runtime.hot,
        None,
        screen_w - corner,
        0,
        corner,
        corner,
        SWP_NOACTIVATE | SWP_SHOWWINDOW,
    );

    if raw >= 1.0 {
        KillTimer(runtime.overlay, TIMER_ID);
        runtime.animating = false;
        runtime.open = runtime.direction == Direction::Opening;
        runtime.progress = if runtime.open { 1.0 } else { 0.0 };

        if runtime.direction == Direction::Closing {
            ShowWindow(runtime.overlay, SW_HIDE);
        }
    }
}

fn liquid_ease(t: f32) -> f32 {
    if t <= 0.0 {
        return 0.0;
    }
    if t >= 1.0 {
        return 1.0;
    }
    let expo = 1.0 - 2.0_f32.powf(-9.0 * t);
    let ripple = (t * std::f32::consts::PI * 3.0).sin() * (1.0 - t) * 0.012;
    (expo + ripple).clamp(0.0, 1.0)
}

fn springish(t: f32) -> f32 {
    let overshoot = 1.0 + 0.018 * (t * std::f32::consts::PI).sin() * (1.0 - t);
    (t * overshoot).clamp(0.0, 1.0)
}

fn lerp_i32(a: i32, b: i32, t: f32) -> i32 {
    (a as f32 + (b - a) as f32 * t).round() as i32
}

unsafe fn paint_glass(hwnd: HWND) {
    let Some(lock) = RUNTIME.get() else {
        return;
    };
    let Ok(runtime) = lock.lock() else {
        return;
    };
    if hwnd != runtime.overlay {
        let mut ps = PAINTSTRUCT::default();
        let hdc = BeginPaint(hwnd, &mut ps);
        EndPaint(hwnd, &ps);
        let _ = hdc;
        return;
    }

    let mut ps = PAINTSTRUCT::default();
    let hdc = BeginPaint(hwnd, &mut ps);
    let mut rect = RECT::default();
    let _ = GetClientRect(hwnd, &mut rect);

    let brush = CreateSolidBrush(COLORREF(0x00221F1F));
    FillRect(hdc, &rect, brush);
    let _ = DeleteObject(brush);
    EndPaint(hwnd, &ps);
}

unsafe fn enable_blur(hwnd: HWND) {
    let blur = DWM_BLURBEHIND {
        dwFlags: DWM_BB_ENABLE,
        fEnable: true.into(),
        hRgnBlur: HRGN::default(),
        fTransitionOnMaximized: false.into(),
    };
    let _ = DwmEnableBlurBehindWindow(hwnd, &blur);
}

unsafe fn set_autostart(enabled: bool) {
    let Ok(exe) = std::env::current_exe() else { return };
    let command = format!("\"{}\"", exe.display());
    let mut key = HKEY::default();
    let subkey = wide("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
    let value_name = wide("Vitrunda");

    if RegOpenKeyExW(
        HKEY_CURRENT_USER,
        PCWSTR(subkey.as_ptr()),
        0,
        KEY_SET_VALUE,
        &mut key,
    )
    .is_err()
    {
        return;
    }

    if enabled {
        let data = wide(&command);
        let bytes = std::slice::from_raw_parts(
            data.as_ptr() as *const u8,
            data.len() * std::mem::size_of::<u16>(),
        );
        let _ = RegSetValueExW(key, PCWSTR(value_name.as_ptr()), 0, REG_SZ, Some(bytes));
    } else {
        let _ = RegDeleteValueW(key, PCWSTR(value_name.as_ptr()));
    }

    let _ = RegCloseKey(key);
}

fn wide(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(Some(0)).collect()
}

#[cfg(test)]
mod animation_tests {
    use super::{liquid_ease, springish, lerp_i32};

    #[test]
    fn animation_endpoints_are_exact() {
        assert_eq!(liquid_ease(0.0), 0.0);
        assert_eq!(liquid_ease(1.0), 1.0);
        assert_eq!(springish(0.0), 0.0);
        assert_eq!(springish(1.0), 1.0);
    }

    #[test]
    fn animation_never_escapes_bounds() {
        for step in 0..=1000 {
            let t = step as f32 / 1000.0;
            assert!((0.0..=1.0).contains(&liquid_ease(t)));
            assert!((0.0..=1.0).contains(&springish(t)));
            assert!((14..=1920).contains(&lerp_i32(14, 1920, springish(t))));
        }
    }
}

fn liquid_corner_radius(width: i32, height: i32, progress: f32) -> i32 {
    let short_edge = width.min(height).max(0);
    let cap = (short_edge / 2).max(0);
    let wave = (progress * std::f32::consts::PI).sin().abs();
    let desired = (short_edge as f32 * (0.18 + 0.14 * wave)).round() as i32;
    if progress >= 1.0 { 0 } else { desired.clamp(0, cap) }
}

#[cfg(test)]
mod liquid_shape_tests {
    use super::liquid_corner_radius;

    #[test]
    fn radius_is_bounded_by_window_size() {
        for width in [8, 14, 80, 320, 1920] {
            for height in [8, 14, 90, 480, 1080] {
                for i in 0..=100 {
                    let radius = liquid_corner_radius(width, height, i as f32 / 100.0);
                    assert!(radius >= 0);
                    assert!(radius <= width.min(height) / 2);
                }
            }
        }
    }

    #[test]
    fn fully_open_overlay_has_no_rounded_clip() {
        assert_eq!(liquid_corner_radius(1920, 1080, 1.0), 0);
    }
}

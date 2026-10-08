mod config;

#[cfg(windows)]
mod platform;

#[cfg(windows)]
fn main() -> windows::core::Result<()> {
    platform::windows::run()
}

#[cfg(not(windows))]
fn main() {
    eprintln!("Vitrunda currently targets Windows 10/11 only.");
}

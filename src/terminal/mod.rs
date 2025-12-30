#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
pub use linux::launch_in_terminal;
#[cfg(target_os = "macos")]
pub use macos::launch_in_terminal;

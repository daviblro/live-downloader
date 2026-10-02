/// Windows process creation flag that prevents console programs from opening a
/// visible console window when started by the GUI application.
#[cfg(windows)]
pub const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Applies the flags every console helper needs when spawned from the GUI.
pub fn hide_console_window(command: &mut tokio::process::Command) {
    #[cfg(windows)]
    {
        command.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(not(windows))]
    {
        let _ = command;
    }
}

/// Terminates a process together with every process it spawned.
///
/// yt-dlp delegates live HLS/DASH downloads to an FFmpeg child process. Windows
/// does not terminate children with their parent, so killing only yt-dlp would
/// leave FFmpeg recording in the background indefinitely.
pub fn kill_process_tree(pid: u32) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;

        let _ = std::process::Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .creation_flags(CREATE_NO_WINDOW)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
    #[cfg(not(windows))]
    {
        let _ = std::process::Command::new("pkill")
            .args(["-KILL", "-P", &pid.to_string()])
            .status();
        let _ = std::process::Command::new("kill")
            .args(["-KILL", &pid.to_string()])
            .status();
    }
}

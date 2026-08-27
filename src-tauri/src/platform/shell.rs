use std::path::Path;

pub fn open_directory(directory: &Path) -> Result<(), String> {
    std::fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    std::process::Command::new("explorer.exe")
        .arg(directory)
        .spawn()
        .map_err(|error| format!("Could not open the download directory: {error}"))?;
    Ok(())
}

pub fn reveal_file(path: &Path) -> Result<(), String> {
    if !path.is_file() {
        return Err("The recording file has been deleted.".to_owned());
    }
    std::process::Command::new("explorer.exe")
        .arg("/select,")
        .arg(path)
        .spawn()
        .map_err(|error| format!("Could not reveal the recording: {error}"))?;
    Ok(())
}

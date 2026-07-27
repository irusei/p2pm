use color_eyre::Result;
use colored::Colorize;
use std::fs;

pub fn parse_deeplink(url: &str) -> Result<String> {
    const PREFIX: &str = "mws-pdmm://install/";
    if !url.starts_with(PREFIX) {
        return Err(color_eyre::eyre::Report::from(
            crate::error::P2PMError::InvalidDeeplink(url.to_string()),
        ));
    }
    let mod_id = &url[PREFIX.len()..];
    if mod_id.is_empty() {
        return Err(color_eyre::eyre::Report::from(
            crate::error::P2PMError::InvalidDeeplink(url.to_string()),
        ));
    }
    Ok(format!("mws/{}", mod_id))
}

pub fn register_deeplink_handler() -> Result<()> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| color_eyre::eyre::eyre!("Could not find home directory"))?;
    let desktop_dir = home_dir.join(".local/share/applications");
    let desktop_file = desktop_dir.join("p2pm-deeplink.desktop");
    let config_dir = home_dir.join(".config");
    let mimeapps_list = config_dir.join("mimeapps.list");

    // Check if already registered
    if desktop_file.exists() && mimeapps_list.exists() {
        let mime_content = fs::read_to_string(&mimeapps_list)?;
        if mime_content.contains("x-scheme-handler/mws-pdmm=p2pm-deeplink.desktop") {
            return Ok(());
        }
    }
    fs::create_dir_all(&desktop_dir)?;

    let p2pm_path = std::env::current_exe()?;
    let desktop_content = format!(
        r#"[Desktop Entry]
        Name=P2PM Deeplink
        Exec={} deeplink %u
        Type=Application
        NoDisplay=true
        Terminal=false
        "#,
        p2pm_path.display()
    );
    fs::write(&desktop_file, desktop_content)?;

    // Register mime type
    let mut mime_content = if mimeapps_list.exists() {
        fs::read_to_string(&mimeapps_list)?
    } else {
        String::new()
    };

    if !mime_content.contains("x-scheme-handler/mws-pdmm") {
        if !mime_content.is_empty() && !mime_content.ends_with('\n') {
            mime_content.push('\n');
        }
        mime_content.push_str(
            "\n[Default Applications]\nx-scheme-handler/mws-pdmm=p2pm-deeplink.desktop\n",
        );
    }

    fs::write(&mimeapps_list, mime_content)?;

    println!(
        "{} {}",
        "[P2PM]".cyan().bold(),
        "Registered mws-pdmm:// deeplink handler".bold()
    );
    Ok(())
}

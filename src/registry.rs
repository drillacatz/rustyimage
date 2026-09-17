use std::path::PathBuf;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

const REG_KEY_STAR: &str = r"HKCU\Software\Classes\*\shell\RustyImage";
const REG_KEY_IMAGE: &str = r"HKCU\Software\Classes\SystemFileAssociations\image\shell\RustyImage";
const LEGACY_KEY_STAR: &str = r"HKCU\Software\Classes\*\shell\RustyMedia";
const LEGACY_KEY_IMAGE: &str = r"HKCU\Software\Classes\SystemFileAssociations\image\shell\RustyMedia";

fn current_exe_path() -> Result<PathBuf, String> {
    std::env::current_exe().map_err(|e| format!("Failed to get executable path: {e}"))
}

pub fn is_context_menu_registered() -> bool {
    #[cfg(windows)]
    {
        let mut cmd = std::process::Command::new("reg");
        cmd.args(["query", REG_KEY_STAR]);
        cmd.creation_flags(CREATE_NO_WINDOW);
        if let Ok(output) = cmd.output() {
            if output.status.success() {
                return true;
            }
        }

        let mut cmd2 = std::process::Command::new("reg");
        cmd2.args(["query", REG_KEY_IMAGE]);
        cmd2.creation_flags(CREATE_NO_WINDOW);
        if let Ok(output) = cmd2.output() {
            return output.status.success();
        }
        false
    }
    #[cfg(not(windows))]
    {
        false
    }
}

pub fn register_context_menu() -> Result<(), String> {
    #[cfg(windows)]
    {
        let exe = current_exe_path()?;
        let exe_str = exe.to_string_lossy().to_string();
        let cmd_str = format!("\"{exe_str}\" \"%1\"");
        let icon_str = format!("\"{exe_str}\"");
        let filter = "System.FileExtension:=.png OR System.FileExtension:=.jpg OR System.FileExtension:=.jpeg OR System.FileExtension:=.webp OR System.FileExtension:=.gif OR System.FileExtension:=.apng OR System.FileExtension:=.bmp OR System.FileExtension:=.ico OR System.FileExtension:=.tiff OR System.FileExtension:=.tif OR System.FileExtension:=.tga";

        // Clean any old RustyMedia legacy keys
        let _ = run_reg(&["delete", LEGACY_KEY_STAR, "/f"]);
        let _ = run_reg(&["delete", LEGACY_KEY_IMAGE, "/f"]);

        // 1. Register under Classes\*\shell\RustyImage with AppliesTo filter
        run_reg(&["add", REG_KEY_STAR, "/ve", "/d", "Convert with RustyImage", "/f"])?;
        run_reg(&["add", REG_KEY_STAR, "/v", "Icon", "/d", &icon_str, "/f"])?;
        run_reg(&["add", REG_KEY_STAR, "/v", "AppliesTo", "/d", filter, "/f"])?;
        run_reg(&["add", &format!(r"{REG_KEY_STAR}\command"), "/ve", "/d", &cmd_str, "/f"])?;

        // 2. Register under SystemFileAssociations\image\shell\RustyImage
        let _ = run_reg(&["add", REG_KEY_IMAGE, "/ve", "/d", "Convert with RustyImage", "/f"]);
        let _ = run_reg(&["add", REG_KEY_IMAGE, "/v", "Icon", "/d", &icon_str, "/f"]);
        let _ = run_reg(&["add", &format!(r"{REG_KEY_IMAGE}\command"), "/ve", "/d", &cmd_str, "/f"]);

        Ok(())
    }
    #[cfg(not(windows))]
    {
        Err("Windows registry integration is only supported on Windows".to_string())
    }
}

pub fn unregister_context_menu() -> Result<(), String> {
    #[cfg(windows)]
    {
        let _ = run_reg(&["delete", REG_KEY_STAR, "/f"]);
        let _ = run_reg(&["delete", REG_KEY_IMAGE, "/f"]);
        let _ = run_reg(&["delete", LEGACY_KEY_STAR, "/f"]);
        let _ = run_reg(&["delete", LEGACY_KEY_IMAGE, "/f"]);
        Ok(())
    }
    #[cfg(not(windows))]
    {
        Err("Windows registry integration is only supported on Windows".to_string())
    }
}

#[cfg(windows)]
fn run_reg(args: &[&str]) -> Result<(), String> {
    let mut cmd = std::process::Command::new("reg");
    cmd.args(args);
    cmd.creation_flags(CREATE_NO_WINDOW);
    let output = cmd.output().map_err(|e| format!("Failed to run reg.exe: {e}"))?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Registry command failed: {err}"));
    }
    Ok(())
}

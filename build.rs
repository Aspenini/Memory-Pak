#[cfg(target_os = "linux")]
use std::fs;
#[cfg(target_os = "linux")]
use std::path::Path;

fn main() {
    // Only create desktop file on Linux
    #[cfg(target_os = "linux")]
    {
        use std::env;
        
        // Get package info from Cargo environment variables
        let pkg_name = env::var("CARGO_PKG_NAME").expect("CARGO_PKG_NAME not set");
        let pkg_version = env::var("CARGO_PKG_VERSION").expect("CARGO_PKG_VERSION not set");
        let pkg_description = env::var("CARGO_PKG_DESCRIPTION").expect("CARGO_PKG_DESCRIPTION not set");
        
        // Convert package name to lowercase with hyphens for Exec and Icon
        let exec_name = pkg_name.to_lowercase().replace(' ', "-");
        
        // Format the display name (replace hyphens with spaces, title case)
        let display_name = pkg_name
            .split('-')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
        
        // Generate desktop file content
        let desktop_content = format!(
            "[Desktop Entry]\n\
            Version={}\n\
            Type=Application\n\
            Name={}\n\
            Comment={}\n\
            Exec={}\n\
            Icon={}\n\
            Terminal=false\n\
            Categories=Utility;Game;\n\
            StartupNotify=true\n",
            pkg_version, display_name, pkg_description, exec_name, exec_name
        );
        
        // Write the desktop file
        let desktop_path = Path::new("memory-pak.desktop");
        fs::write(desktop_path, desktop_content)
            .expect("Failed to write memory-pak.desktop");
    }
    
    println!("cargo:rerun-if-changed=Cargo.toml");
}


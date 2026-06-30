use std::error::Error;

pub trait DocumentService {
    fn export_backup(&self, json: &str) -> Result<(), Box<dyn Error>>;
    fn import_backup(&self, callback: Box<dyn FnOnce(Result<Option<String>, String>)>);
}

pub struct PlatformDocuments;

impl DocumentService for PlatformDocuments {
    fn export_backup(&self, json: &str) -> Result<(), Box<dyn Error>> {
        crate::persistence::export_backup(json)
    }

    fn import_backup(&self, callback: Box<dyn FnOnce(Result<Option<String>, String>)>) {
        crate::persistence::request_import(callback);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateState {
    Unsupported,
    Current,
    Available(String),
    Installed,
    Error(String),
}

pub trait UpdateService {
    fn check(&self, callback: Box<dyn FnOnce(UpdateState) + Send>);
    fn install(&self, callback: Box<dyn FnOnce(UpdateState) + Send>);
}

pub struct PlatformUpdates;

#[cfg(all(
    not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")),
    any(target_os = "windows", target_os = "macos")
))]
static PENDING_UPDATE: std::sync::OnceLock<
    std::sync::Mutex<Option<cargo_packager_updater::Update>>,
> = std::sync::OnceLock::new();

impl UpdateService for PlatformUpdates {
    fn check(&self, callback: Box<dyn FnOnce(UpdateState) + Send>) {
        #[cfg(all(
            not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")),
            any(target_os = "windows", target_os = "macos")
        ))]
        {
            std::thread::spawn(move || {
                let Some(pubkey) = option_env!("MEMORY_PAK_UPDATER_PUBKEY")
                    .filter(|value| !value.trim().is_empty())
                else {
                    callback(UpdateState::Unsupported);
                    return;
                };
                let endpoint = match "https://github.com/Aspenini/Memory-Pak/releases/latest/download/latest.json".parse() {
                    Ok(endpoint) => endpoint,
                    Err(error) => {
                        callback(UpdateState::Error(format!("Invalid update endpoint: {error}")));
                        return;
                    }
                };
                let config = cargo_packager_updater::Config {
                    endpoints: vec![endpoint],
                    pubkey: pubkey.to_string(),
                    ..Default::default()
                };
                let version = match env!("CARGO_PKG_VERSION").parse() {
                    Ok(version) => version,
                    Err(error) => {
                        callback(UpdateState::Error(format!("Invalid app version: {error}")));
                        return;
                    }
                };
                match cargo_packager_updater::check_update(version, config) {
                    Ok(Some(update)) => {
                        let version = update.version.to_string();
                        *PENDING_UPDATE
                            .get_or_init(Default::default)
                            .lock()
                            .expect("update mutex") = Some(update);
                        callback(UpdateState::Available(version));
                    }
                    Ok(None) => callback(UpdateState::Current),
                    Err(error) => callback(UpdateState::Error(error.to_string())),
                }
            });
            return;
        }

        #[cfg(not(all(
            not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")),
            any(target_os = "windows", target_os = "macos")
        )))]
        callback(UpdateState::Unsupported);
    }

    fn install(&self, callback: Box<dyn FnOnce(UpdateState) + Send>) {
        #[cfg(all(
            not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")),
            any(target_os = "windows", target_os = "macos")
        ))]
        {
            std::thread::spawn(move || {
                let update = PENDING_UPDATE
                    .get_or_init(Default::default)
                    .lock()
                    .expect("update mutex")
                    .take();
                match update {
                    Some(update) => match update.download_and_install() {
                        Ok(()) => callback(UpdateState::Installed),
                        Err(error) => callback(UpdateState::Error(error.to_string())),
                    },
                    None => callback(UpdateState::Current),
                }
            });
            return;
        }

        #[cfg(not(all(
            not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")),
            any(target_os = "windows", target_os = "macos")
        )))]
        callback(UpdateState::Unsupported);
    }
}

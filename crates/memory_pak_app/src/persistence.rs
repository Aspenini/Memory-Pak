use std::error::Error;

pub trait SaveStore {
    fn load(&self) -> Result<Option<String>, Box<dyn Error>>;
    fn save(&self, json: &str) -> Result<(), Box<dyn Error>>;
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub fn platform_store() -> Result<Box<dyn SaveStore>, Box<dyn Error>> {
    Ok(Box::new(NativeSaveStore::new()?))
}

#[cfg(target_arch = "wasm32")]
pub fn platform_store() -> Result<Box<dyn SaveStore>, Box<dyn Error>> {
    Ok(Box::new(WebSaveStore))
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
struct NativeSaveStore {
    path: std::path::PathBuf,
    legacy_paths: Vec<std::path::PathBuf>,
    sender: std::sync::mpsc::Sender<NativeSaveCommand>,
    last_error: std::sync::Arc<std::sync::Mutex<Option<String>>>,
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
enum NativeSaveCommand {
    Save(String),
    Flush(std::sync::mpsc::Sender<Result<(), String>>),
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
impl NativeSaveStore {
    fn new() -> Result<Self, Box<dyn Error>> {
        use directories::ProjectDirs;

        let current = ProjectDirs::from("com", "Aspenini", "MemoryPak")
            .ok_or("application data directory is unavailable")?
            .data_dir()
            .join("state.json");
        let mut legacy_paths = Vec::new();
        if let Some(path) = ProjectDirs::from("com", "Aspenini", "MemoryPak") {
            legacy_paths.push(path.data_local_dir().join("state.json"));
        }
        if let Some(path) = ProjectDirs::from("com", "memorypak", "memory_pak") {
            legacy_paths.push(path.data_dir().join("state.json"));
        }
        let (sender, receiver) = std::sync::mpsc::channel();
        let last_error = std::sync::Arc::new(std::sync::Mutex::new(None));
        let worker_error = last_error.clone();
        let worker_path = current.clone();
        std::thread::Builder::new()
            .name("memory-pak-save".into())
            .spawn(move || native_save_worker(&worker_path, receiver, &worker_error))?;
        Ok(Self {
            path: current,
            legacy_paths,
            sender,
            last_error,
        })
    }

    fn read_path(path: &std::path::Path) -> Result<Option<String>, Box<dyn Error>> {
        match std::fs::read_to_string(path) {
            Ok(json) => Ok(Some(json)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
impl SaveStore for NativeSaveStore {
    fn load(&self) -> Result<Option<String>, Box<dyn Error>> {
        if let Some(json) = Self::read_path(&self.path)? {
            return Ok(Some(json));
        }
        for path in &self.legacy_paths {
            if let Some(json) = Self::read_path(path)? {
                return Ok(Some(json));
            }
        }
        Ok(None)
    }

    fn save(&self, json: &str) -> Result<(), Box<dyn Error>> {
        if let Some(error) = self.last_error.lock().expect("save error mutex").take() {
            return Err(error.into());
        }
        self.sender
            .send(NativeSaveCommand::Save(json.to_string()))?;
        Ok(())
    }
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
impl Drop for NativeSaveStore {
    fn drop(&mut self) {
        let (sender, receiver) = std::sync::mpsc::channel();
        if self.sender.send(NativeSaveCommand::Flush(sender)).is_ok() {
            let _ = receiver.recv_timeout(std::time::Duration::from_secs(3));
        }
    }
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
fn native_save_worker(
    path: &std::path::Path,
    receiver: std::sync::mpsc::Receiver<NativeSaveCommand>,
    last_error: &std::sync::Mutex<Option<String>>,
) {
    let mut pending = None;
    loop {
        let command = if pending.is_some() {
            receiver.recv_timeout(std::time::Duration::from_millis(75))
        } else {
            receiver
                .recv()
                .map_err(|_| std::sync::mpsc::RecvTimeoutError::Disconnected)
        };
        match command {
            Ok(NativeSaveCommand::Save(json)) => pending = Some(json),
            Ok(NativeSaveCommand::Flush(sender)) => {
                let result = pending
                    .take()
                    .map(|json| write_save_file(path, &json))
                    .unwrap_or(Ok(()));
                let _ = sender.send(result);
                break;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                if let Some(json) = pending.take() {
                    if let Err(error) = write_save_file(path, &json) {
                        *last_error.lock().expect("save error mutex") = Some(error);
                    }
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                if let Some(json) = pending.take() {
                    let _ = write_save_file(path, &json);
                }
                break;
            }
        }
    }
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
fn write_save_file(path: &std::path::Path, json: &str) -> Result<(), String> {
    let parent = path.parent().ok_or("invalid save path")?;
    std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    atomic_write(path, json).map_err(|error| error.to_string())
}

#[cfg(target_os = "android")]
static MOBILE_DATA_DIR: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();

#[cfg(target_os = "android")]
static ANDROID_APP: std::sync::OnceLock<slint::android::AndroidApp> = std::sync::OnceLock::new();

#[cfg(target_os = "android")]
pub fn set_android_app(app: slint::android::AndroidApp) {
    if let Some(path) = app.internal_data_path() {
        let _ = MOBILE_DATA_DIR.set(path);
    }
    let _ = ANDROID_APP.set(app);
}

#[cfg(target_os = "android")]
pub fn platform_store() -> Result<Box<dyn SaveStore>, Box<dyn Error>> {
    let path = MOBILE_DATA_DIR
        .get()
        .ok_or("Android data directory was not initialized")?
        .join("state.json");
    Ok(Box::new(PathSaveStore { path }))
}

#[cfg(target_os = "ios")]
pub fn platform_store() -> Result<Box<dyn SaveStore>, Box<dyn Error>> {
    let path = directories::ProjectDirs::from("com", "Aspenini", "MemoryPak")
        .ok_or("iOS application support directory is unavailable")?
        .data_dir()
        .join("state.json");
    Ok(Box::new(PathSaveStore { path }))
}

#[cfg(any(target_os = "android", target_os = "ios"))]
struct PathSaveStore {
    path: std::path::PathBuf,
}

#[cfg(any(target_os = "android", target_os = "ios"))]
impl SaveStore for PathSaveStore {
    fn load(&self) -> Result<Option<String>, Box<dyn Error>> {
        match std::fs::read_to_string(&self.path) {
            Ok(json) => Ok(Some(json)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    fn save(&self, json: &str) -> Result<(), Box<dyn Error>> {
        let parent = self.path.parent().ok_or("invalid mobile save path")?;
        std::fs::create_dir_all(parent)?;
        atomic_write(&self.path, json)?;
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn atomic_write(path: &std::path::Path, value: &str) -> Result<(), Box<dyn Error>> {
    use atomicwrites::{AllowOverwrite, AtomicFile};
    use std::io::Write;

    AtomicFile::new(path, AllowOverwrite).write(|file| file.write_all(value.as_bytes()))?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(inline_js = r#"
const DB_NAME = "memory-pak";
const DB_VERSION = 1;
const STORE = "state";
const KEY = "persisted";
function openDb() {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION);
    request.onupgradeneeded = () => {
      if (!request.result.objectStoreNames.contains(STORE)) {
        request.result.createObjectStore(STORE);
      }
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}
export async function memoryPakLoadState() {
  const db = await openDb();
  return await new Promise((resolve, reject) => {
    const request = db.transaction(STORE, "readonly").objectStore(STORE).get(KEY);
    request.onsuccess = () => resolve(request.result ? JSON.stringify(request.result) : null);
    request.onerror = () => reject(request.error);
  });
}
let saveQueue = Promise.resolve();
async function saveStateNow(json) {
  const db = await openDb();
  const value = JSON.parse(json);
  return await new Promise((resolve, reject) => {
    const tx = db.transaction(STORE, "readwrite");
    tx.objectStore(STORE).put(value, KEY);
    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error);
  });
}
export function memoryPakSaveState(json) {
  saveQueue = saveQueue.catch(() => {}).then(() => saveStateNow(json));
  return saveQueue;
}
export function memoryPakDownload(json) {
  const blob = new Blob([json], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = "memory_pak_export.json";
  anchor.click();
  URL.revokeObjectURL(url);
}
export async function memoryPakPickBackup() {
  return await new Promise((resolve, reject) => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = "application/json,.json";
    input.onchange = async () => {
      try {
        resolve(input.files?.[0] ? await input.files[0].text() : null);
      } catch (error) {
        reject(error);
      }
    };
    input.oncancel = () => resolve(null);
    input.click();
  });
}
"#)]
extern "C" {
    #[wasm_bindgen::prelude::wasm_bindgen(catch, js_name = memoryPakLoadState)]
    async fn web_load_state() -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;
    #[wasm_bindgen::prelude::wasm_bindgen(catch, js_name = memoryPakSaveState)]
    async fn web_save_state(json: String) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;
    #[wasm_bindgen::prelude::wasm_bindgen(js_name = memoryPakDownload)]
    fn web_download(json: &str);
    #[wasm_bindgen::prelude::wasm_bindgen(catch, js_name = memoryPakPickBackup)]
    async fn web_pick_backup() -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;
}

#[cfg(target_arch = "wasm32")]
pub struct WebSaveStore;

#[cfg(target_arch = "wasm32")]
impl SaveStore for WebSaveStore {
    fn load(&self) -> Result<Option<String>, Box<dyn Error>> {
        Ok(None)
    }

    fn save(&self, json: &str) -> Result<(), Box<dyn Error>> {
        let json = json.to_string();
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(error) = web_save_state(json).await {
                web_sys::console::warn_1(&error);
            }
        });
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub async fn load_web_state() -> Result<Option<String>, wasm_bindgen::JsValue> {
    let value = web_load_state().await?;
    Ok(value.as_string())
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub fn export_backup(json: &str) -> Result<(), Box<dyn Error>> {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter("Memory Pak Export", &["json"])
        .set_file_name("memory_pak_export.json")
        .save_file()
    {
        std::fs::write(path, json)?;
    }
    Ok(())
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub fn import_backup() -> Result<Option<String>, Box<dyn Error>> {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("Memory Pak Export", &["json"])
        .pick_file()
    else {
        return Ok(None);
    };
    Ok(Some(std::fs::read_to_string(path)?))
}

#[cfg(target_arch = "wasm32")]
pub fn export_backup(json: &str) -> Result<(), Box<dyn Error>> {
    web_download(json);
    Ok(())
}

#[cfg(target_os = "android")]
pub fn export_backup(json: &str) -> Result<(), Box<dyn Error>> {
    android_activity_call("exportBackup", "(Ljava/lang/String;)V", Some(json))?;
    Ok(())
}

#[cfg(target_os = "ios")]
pub fn export_backup(json: &str) -> Result<(), Box<dyn Error>> {
    let path = ios_backup_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    atomic_write(&path, json)?;
    Ok(())
}

#[cfg(target_os = "ios")]
pub fn import_backup() -> Result<Option<String>, Box<dyn Error>> {
    match std::fs::read_to_string(ios_backup_path()?) {
        Ok(json) => Ok(Some(json)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

#[cfg(target_os = "ios")]
fn ios_backup_path() -> Result<std::path::PathBuf, Box<dyn Error>> {
    let home = std::env::var_os("HOME").ok_or("iOS home directory is unavailable")?;
    Ok(std::path::PathBuf::from(home)
        .join("Documents")
        .join("memory_pak_export.json"))
}

pub fn request_import(callback: impl FnOnce(Result<Option<String>, String>) + 'static) {
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(async move {
        let result = web_pick_backup()
            .await
            .map(|value| value.as_string())
            .map_err(|error| format!("{error:?}"));
        callback(result);
    });

    #[cfg(target_os = "android")]
    {
        ANDROID_IMPORT_CALLBACK.with(|slot| {
            *slot.borrow_mut() = Some(Box::new(callback));
        });
        if let Err(error) = android_activity_call("importBackup", "()V", None) {
            ANDROID_IMPORT_CALLBACK.with(|slot| {
                if let Some(callback) = slot.borrow_mut().take() {
                    callback(Err(error.to_string()));
                }
            });
        }
    }

    #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
    callback(import_backup().map_err(|error| error.to_string()));
}

#[cfg(target_os = "android")]
thread_local! {
    static ANDROID_IMPORT_CALLBACK: std::cell::RefCell<
        Option<Box<dyn FnOnce(Result<Option<String>, String>)>>
    > = const { std::cell::RefCell::new(None) };
}

#[cfg(target_os = "android")]
fn android_activity_call(
    method: &str,
    signature: &str,
    value: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    use jni::objects::{JObject, JValue};
    use jni::JavaVM;

    let app = ANDROID_APP
        .get()
        .ok_or("Android application is not initialized")?;
    let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast())? };
    let mut env = vm.attach_current_thread()?;
    let activity = unsafe { JObject::from_raw(app.activity_as_ptr().cast()) };
    let result = if let Some(value) = value {
        let string = env.new_string(value)?;
        env.call_method(
            &activity,
            method,
            signature,
            &[JValue::Object(&JObject::from(string))],
        )
    } else {
        env.call_method(&activity, method, signature, &[])
    };
    std::mem::forget(activity);
    result?;
    Ok(())
}

#[cfg(target_os = "android")]
fn finish_android_import(result: Result<Option<String>, String>) {
    let _ = slint::invoke_from_event_loop(move || {
        ANDROID_IMPORT_CALLBACK.with(|slot| {
            if let Some(callback) = slot.borrow_mut().take() {
                callback(result);
            }
        });
    });
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_Aspenini_MemoryPak_MemoryPakActivity_nativeImportBackup(
    mut env: jni::JNIEnv,
    _activity: jni::objects::JObject,
    json: jni::objects::JString,
) {
    let result = env
        .get_string(&json)
        .map(|value| Some(value.into()))
        .map_err(|error| error.to_string());
    finish_android_import(result);
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_Aspenini_MemoryPak_MemoryPakActivity_nativeImportCancelled(
    _env: jni::JNIEnv,
    _activity: jni::objects::JObject,
) {
    finish_android_import(Ok(None));
}

use jni::objects::{JObject, JString, JValue};
use jni::JavaVM;
use std::sync::{Mutex, OnceLock};

type AndroidApp = winit::platform::android::activity::AndroidApp;

static APP: OnceLock<AndroidApp> = OnceLock::new();
static LAST_IMPORTED_URI: Mutex<Option<String>> = Mutex::new(None);

trait JniResultExt<T> {
    fn jni(self) -> Result<T, String>;
}

impl<T> JniResultExt<T> for jni::errors::Result<T> {
    fn jni(self) -> Result<T, String> {
        self.map_err(|err| err.to_string())
    }
}

pub fn init(app: AndroidApp) {
    let _ = APP.set(app);
}

pub fn toast(message: &str) {
    if let Err(err) = with_env(|env, activity| {
        let text = env.new_string(message).jni()?;
        let toast = env
            .call_static_method(
                "android/widget/Toast",
                "makeText",
                "(Landroid/content/Context;Ljava/lang/CharSequence;I)Landroid/widget/Toast;",
                &[
                    JValue::Object(activity),
                    JValue::Object(&JObject::from(text)),
                    JValue::Int(0),
                ],
            )
            .jni()?
            .l()
            .jni()?;
        env.call_method(toast, "show", "()V", &[]).jni()?;
        Ok(())
    }) {
        eprintln!("Android toast error: {err}");
    }
}

pub fn export_json_to_downloads(filename: &str, json: &str) -> Result<(), String> {
    with_env(|env, activity| {
        let resolver = env
            .call_method(
                activity,
                "getContentResolver",
                "()Landroid/content/ContentResolver;",
                &[],
            )
            .jni()?
            .l()
            .jni()?;

        let values = env
            .new_object("android/content/ContentValues", "()V", &[])
            .jni()?;
        put_content_value(env, &values, "title", filename)?;
        put_content_value(env, &values, "_display_name", filename)?;
        put_content_value(env, &values, "mime_type", "application/json")?;

        let sdk = env
            .get_static_field("android/os/Build$VERSION", "SDK_INT", "I")
            .jni()?
            .i()
            .jni()?;
        let collection = if sdk >= 29 {
            put_content_value(env, &values, "relative_path", "Download/Memory Pak")?;
            env.get_static_field(
                "android/provider/MediaStore$Downloads",
                "EXTERNAL_CONTENT_URI",
                "Landroid/net/Uri;",
            )
            .jni()?
            .l()
            .jni()?
        } else {
            let volume = env.new_string("external").jni()?;
            env.call_static_method(
                "android/provider/MediaStore$Files",
                "getContentUri",
                "(Ljava/lang/String;)Landroid/net/Uri;",
                &[JValue::Object(&JObject::from(volume))],
            )
            .jni()?
            .l()
            .jni()?
        };

        let uri = env
            .call_method(
                &resolver,
                "insert",
                "(Landroid/net/Uri;Landroid/content/ContentValues;)Landroid/net/Uri;",
                &[JValue::Object(&collection), JValue::Object(&values)],
            )
            .jni()?
            .l()
            .jni()?;
        if uri.is_null() {
            return Err("Android MediaStore insert returned null".into());
        }

        let stream = env
            .call_method(
                &resolver,
                "openOutputStream",
                "(Landroid/net/Uri;)Ljava/io/OutputStream;",
                &[JValue::Object(&uri)],
            )
            .jni()?
            .l()
            .jni()?;
        if stream.is_null() {
            return Err("Android could not open export stream".into());
        }

        let bytes = env.byte_array_from_slice(json.as_bytes()).jni()?;
        env.call_method(
            &stream,
            "write",
            "([B)V",
            &[JValue::Object(&JObject::from(bytes))],
        )
        .jni()?;
        env.call_method(stream, "close", "()V", &[]).jni()?;
        Ok(())
    })
}

pub fn show_import_hint() {
    toast("Open a Memory Pak JSON file from Android Files and choose Memory Pak");
}

pub fn take_view_intent_json() -> Option<Result<String, String>> {
    match read_view_intent_json() {
        Ok(Some((uri, json))) => {
            let mut last_uri = LAST_IMPORTED_URI.lock().ok()?;
            if last_uri.as_deref() == Some(uri.as_str()) {
                None
            } else {
                *last_uri = Some(uri);
                Some(Ok(json))
            }
        }
        Ok(None) => None,
        Err(err) => Some(Err(err)),
    }
}

fn read_view_intent_json() -> Result<Option<(String, String)>, String> {
    with_env(|env, activity| {
        let intent = env
            .call_method(activity, "getIntent", "()Landroid/content/Intent;", &[])
            .jni()?
            .l()
            .jni()?;
        if intent.is_null() {
            return Ok(None);
        }

        let action = env
            .call_method(&intent, "getAction", "()Ljava/lang/String;", &[])
            .jni()?
            .l()
            .jni()?;
        let action = jstring_to_string(env, JString::from(action))?;
        if action.as_deref() != Some("android.intent.action.VIEW") {
            return Ok(None);
        }

        let uri = env
            .call_method(&intent, "getData", "()Landroid/net/Uri;", &[])
            .jni()?
            .l()
            .jni()?;
        if uri.is_null() {
            return Ok(None);
        }

        let uri_string_obj = env
            .call_method(&uri, "toString", "()Ljava/lang/String;", &[])
            .jni()?
            .l()
            .jni()?;
        let Some(uri_string) = jstring_to_string(env, JString::from(uri_string_obj))? else {
            return Ok(None);
        };

        let json = read_uri_to_string(env, activity, &uri)?;
        Ok(Some((uri_string, json)))
    })
}

fn read_uri_to_string(
    env: &mut jni::JNIEnv<'_>,
    activity: &JObject<'_>,
    uri: &JObject<'_>,
) -> Result<String, String> {
    let resolver = env
        .call_method(
            activity,
            "getContentResolver",
            "()Landroid/content/ContentResolver;",
            &[],
        )
        .jni()?
        .l()
        .jni()?;
    let stream = env
        .call_method(
            &resolver,
            "openInputStream",
            "(Landroid/net/Uri;)Ljava/io/InputStream;",
            &[JValue::Object(uri)],
        )
        .jni()?
        .l()
        .jni()?;
    if stream.is_null() {
        return Err("Android could not open import stream".into());
    }

    let buffer = env.new_byte_array(8192).jni()?;
    let mut bytes = Vec::new();

    loop {
        let read = env
            .call_method(&stream, "read", "([B)I", &[JValue::Object(buffer.as_ref())])
            .jni()?
            .i()
            .jni()?;
        if read < 0 {
            break;
        }
        if read == 0 {
            continue;
        }
        let chunk = env.convert_byte_array(&buffer).jni()?;
        bytes.extend_from_slice(&chunk[..read as usize]);
    }

    env.call_method(stream, "close", "()V", &[]).jni()?;
    String::from_utf8(bytes).map_err(|err| err.to_string())
}

fn put_content_value(
    env: &mut jni::JNIEnv<'_>,
    values: &JObject<'_>,
    key: &str,
    value: &str,
) -> Result<(), String> {
    let key = env.new_string(key).jni()?;
    let value = env.new_string(value).jni()?;
    env.call_method(
        values,
        "put",
        "(Ljava/lang/String;Ljava/lang/String;)V",
        &[
            JValue::Object(&JObject::from(key)),
            JValue::Object(&JObject::from(value)),
        ],
    )
    .jni()?;
    Ok(())
}

fn jstring_to_string(
    env: &mut jni::JNIEnv<'_>,
    string: JString<'_>,
) -> Result<Option<String>, String> {
    if string.is_null() {
        return Ok(None);
    }
    Ok(Some(env.get_string(&string).jni()?.into()))
}

fn with_env<T>(
    f: impl FnOnce(&mut jni::JNIEnv<'_>, &JObject<'_>) -> Result<T, String>,
) -> Result<T, String> {
    let app = APP
        .get()
        .ok_or_else(|| "Android app is not initialized".to_string())?;
    let vm_ptr = app.vm_as_ptr().cast();
    let vm = unsafe { JavaVM::from_raw(vm_ptr) }.map_err(|err| err.to_string())?;
    let mut env = vm.attach_current_thread().map_err(|err| err.to_string())?;
    let activity = unsafe { JObject::from_raw(app.activity_as_ptr().cast()) };
    f(&mut env, &activity)
}

use crate::btreemap_iter::BTreeMapIterator;
use crate::file_storage::FileStorage;
// This is the interface to the JVM that we'll call the majority of our
// methods on.
use jni::JNIEnv;

// These objects are what you should use as arguments to your native
// function. They carry extra lifetime information to prevent them escaping
// this context and getting used after being GC'd.
use jni::objects::{JClass, JValue};

// This is just a pointer. We'll be returning it from our function. We
// can't return one of the objects with lifetime information because the
// lifetime checker won't let us.
use jni::sys::{jboolean, jlong, jobject};

impl BTreeMapIterator<'_, String, String> {
    pub fn from_jlong(value: jlong) -> &'static mut Self {
        unsafe { &mut *(value as *mut BTreeMapIterator<String, String>) }
    }
}

#[no_mangle]
pub extern "system" fn Java_dev_arkbuilders_core_BTreeMapIterator_create(
    _env: JNIEnv<'_>,
    _class: JClass,
    file_storage_ptr: jlong,
) -> jlong {
    let file_storage = FileStorage::from_jlong(file_storage_ptr);
    let iter = BTreeMapIterator::new(file_storage);
    Box::into_raw(Box::new(iter)) as jlong
}

#[no_mangle]
pub extern "system" fn Java_dev_arkbuilders_core_BTreeMapIterator_hasNext(
    _env: JNIEnv<'_>,
    _class: JClass,
    btreemap_ptr: jlong,
) -> jboolean {
    let iter = BTreeMapIterator::from_jlong(btreemap_ptr);
    iter.has_next() as jboolean
}

#[no_mangle]
pub extern "system" fn Java_dev_arkbuilders_core_BTreeMapIterator_next(
    mut env: JNIEnv<'_>,
    _class: JClass,
    btreemap_ptr: jlong,
) -> jobject {
    let iter = BTreeMapIterator::from_jlong(btreemap_ptr);
    let (key, value) = iter.native_next().unwrap();
    let key = env.new_string(key).unwrap();
    let value = env.new_string(value).unwrap();
    let pair = env
        .new_object(
            "java/util/AbstractMap$SimpleImmutableEntry",
            "(Ljava/lang/Object;Ljava/lang/Object;)V",
            &[JValue::Object(&key), JValue::Object(&value)],
        )
        .unwrap();
    pair.as_raw()
}

#[no_mangle]
pub extern "system" fn Java_dev_arkbuilders_core_BTreeMapIterator_drop(
    _env: JNIEnv<'_>,
    _class: JClass,
    btreemap_ptr: jlong,
) {
    unsafe {
        let _ = Box::from_raw(
            btreemap_ptr as *mut BTreeMapIterator<String, String>,
        );
    }
}

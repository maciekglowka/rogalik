use jni::{
    self,
    objects::{JObject, JValueGen},
    JavaVM
};
pub use winit::platform::android::activity::AndroidApp;

pub fn hide_ui() {
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.unwrap();
    let context = unsafe { JObject::from_raw(ctx.context().cast()) };
    let mut env = vm.attach_current_thread().unwrap();
    let activity_class = env.find_class("android/app/NativeActivity").unwrap();
    let window = env.call_method(
            context,
            "getWindow",
            "()Landroid/view/Window;",
            &[]
        ).
        unwrap()
        .l()
        .unwrap();

    let decor_view = env.call_method(
            window,
            "getDecorView",
            "()Landroid/view/View;",
            &[]
        )
        .unwrap()
        .l()
        .unwrap();

    let controller = env.call_method(
            decor_view,
            "getWindowInsetsController",
            "()Landroid/view/WindowInsetsController;",
            &[]
        )
        .unwrap()
        .l()
        .unwrap();

    let val = 1 << 0 | 1 << 1 | 1 << 2;
    let jval = JValueGen::Int(val);

    let _ = env.call_method(
            controller,
            "hide",
            "(I)V",
            &[jval]
        )
        .unwrap();
}
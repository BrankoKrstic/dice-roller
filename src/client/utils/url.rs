pub fn base_url() -> String {
    web_sys::window().unwrap().location().origin().unwrap()
}

// Legacy UI Module (Minifb removed in Sprint 51)
// Function signatures kept to satisfy FFI bounds, but now act as no-ops.

pub fn ui_init_window(_width: i64, _height: i64, _title: String) -> bool {
    eprintln!("[KnotenCore UI] Legacy UI module deprecated. Use Registry WGPU context instead.");
    false
}

pub fn ui_clear(_color: i64) {}

pub fn ui_draw_rect(_x: i64, _y: i64, _w: i64, _h: i64, _color: i64) {}

pub fn ui_draw_text(_x: i64, _y: i64, _text: String, _color: i64) {}

pub fn ui_present() -> bool {
    false
}

pub fn ui_is_key_down(_key_name: String) -> bool {
    false
}

pub fn ui_get_key_pressed() -> String {
    String::new()
}

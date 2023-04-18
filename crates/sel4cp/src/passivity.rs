#[no_mangle]
#[link_section = ".data"]
static passive: bool = false; // just a placeholder

pub fn is_passive() -> bool {
    passive
}

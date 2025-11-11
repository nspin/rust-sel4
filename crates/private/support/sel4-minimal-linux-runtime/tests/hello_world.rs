#![no_std]
#![no_main]

use sel4_minimal_linux_runtime::*;

#[main]
fn main() {
    debug_println!("Hello, World!");
}

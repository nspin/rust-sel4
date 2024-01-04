// https://github.com/rust-lang/compiler-builtins/pull/563
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
#[no_mangle]
pub extern "C" fn __bswapsi2(u: u32) -> u32 {
    ((u & 0xff000000) >> 24)
        | ((u & 0x00ff0000) >> 8)
        | ((u & 0x0000ff00) << 8)
        | ((u & 0x000000ff) << 24)
}

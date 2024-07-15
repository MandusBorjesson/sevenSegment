# Build
cargo build --release

# Flash
cargo flash --chip stm32f103c8 --release
cargo flash --chip stm32f051k8ux --release

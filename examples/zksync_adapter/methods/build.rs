#[cfg(any(feature = "sp1"))]
use sp1_build::build_program;

fn main() {
    #[cfg(any(feature = "risc0"))]
    risc0_build::embed_methods();

    #[cfg(any(feature = "sp1"))]
    build_program("./sp1-guest")
}

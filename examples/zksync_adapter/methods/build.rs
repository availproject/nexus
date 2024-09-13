#[cfg(any(feature = "sp1"))]
use sp1_helper::build_program_with_args;

fn main() {
    #[cfg(any(feature = "risc0"))]
    risc0_build::embed_methods();
    #[cfg(any(feature = "sp1"))]
    build_program_with_args("./sp1-guest", Default::default())
}

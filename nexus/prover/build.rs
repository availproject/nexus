#[cfg(any(feature = "sp1"))]
use sp1_build::build_program;

#[cfg(any(feature = "risc0"))]
const SOLIDITY_IMAGE_ID_PATH: &str = "../../contracts/src/NexusProverImageID.sol";
#[cfg(any(feature = "risc0"))]
const SOLIDITY_ELF_PATH: &str = "../../contracts/test/NexusProverElf.sol";

fn main() {
    #[cfg(any(feature = "risc0"))]
    {
        let guests = risc0_build::embed_methods();
        let solidity_opts = risc0_build_ethereum::Options::default()
            .with_image_id_sol_path(SOLIDITY_IMAGE_ID_PATH)
            .with_elf_sol_path(SOLIDITY_ELF_PATH);
        risc0_build_ethereum::generate_solidity_files(guests.as_slice(), &solidity_opts).unwrap();
    }
    #[cfg(any(feature = "sp1"))]
    build_program("./sp1-guest")
}

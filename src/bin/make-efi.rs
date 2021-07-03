use std::{
    env,
    path::PathBuf,
    process::{self, Command},
};

fn main() {
    let maia_path = PathBuf::from(
        env::var("MAIA").expect("environment variable MAIA must be set")
    );

    let efi_path = match env::args().nth(1) {
        Some(path) => PathBuf::from(path),
        None => {
            let work_dir = env::current_dir().expect("unable to access working directory");
            work_dir.join("BOOTRISCV64.efi")
        },
    };

    // get access to llvm tools shipped in the llvm-tools-preview rustup component
    let llvm_tools = match llvm_tools::LlvmTools::new() {
        Ok(tools) => tools,
        Err(llvm_tools::Error::NotFound) => {
            eprintln!("Error: llvm-tools not found");
            eprintln!("Maybe the rustup component `llvm-tools-preview` is missing?");
            eprintln!("  Install it through: `rustup component add llvm-tools-preview`");
            process::exit(1);
        }
        Err(err) => {
            eprintln!("Failed to retrieve llvm-tools component: {:?}", err);
            process::exit(1);
        }
    };

    // Generate valid efi binary
    let objcopy = llvm_tools
        .tool(&llvm_tools::exe("llvm-objcopy"))
        .expect("llvm-objcopy not found in llvm-tools");
    let mut cmd = Command::new(&objcopy);
    let sections = vec![
        ".text", ".plt", ".sdata", ".data",
        ".dynamic", ".dynstr", ".dynsym",
        ".rel*", ".rela*", ".reloc",
    ];
    for section in &sections {
        cmd.arg("-j").arg(section);
    }
    cmd.arg("--output-target").arg("binary");
    cmd.arg(&maia_path);
    cmd.arg(&efi_path);
    let exit_status = cmd
        .status()
        .expect("failed to run objcopy to generate efi image");
    if !exit_status.success() {
        eprintln!("Error: EFI image creation failed");
        process::exit(1);
    }
}

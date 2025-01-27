use clap::{CommandFactory, ValueEnum};

mod cli {
    include!("src/cli.rs");
}

fn main() -> std::io::Result<()> {
    let out_dir = match std::env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(outdir) => std::path::PathBuf::from(outdir),
    };
    let mut cmd = cli::Arguments::command();

    // Generate Man Page
    let man = clap_mangen::Man::new(cmd.clone());
    let mut buffer: Vec<u8> = Vec::default();
    man.render(&mut buffer)?;

    std::fs::write(out_dir.join("astrolabe.1"), buffer)?;

    // Generate completions
    for &shell in clap_complete::Shell::value_variants() {
        let path = clap_complete::generate_to(shell, &mut cmd, "astrolabe", &out_dir)?;
        println!("cargo::warning=Completion file for {shell: <10} has been generated at: {path:?}");
    }

    Ok(())
}

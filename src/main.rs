use clap::Parser;
use cli_wrapped::shell::{Shell, ShellType};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = ShellType::Bash)]
    shell_type: ShellType,
    #[arg(short, long)]
    /// Path to custom history shell file;
    /// expects that the file is formatted just like other shell history files.
    path_to_history: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut shell = Shell::from(args.shell_type);
    let freq = shell.command_frequency()?;

    println!("{freq:?}");

    println!("{}", shell.command_count.unwrap());

    Ok(())
}

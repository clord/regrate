use crate::Regrate;
use clap::{Args, CommandFactory};
use clap_complete::{generate, Shell};
use eyre::Result;

#[derive(Args, Debug)]
pub struct GenerateArgs {
    /// output completion script
    #[arg(value_enum)]
    shell: Shell,
}

pub fn generate_completion(args: GenerateArgs) -> Result<()> {
    let generator = args.shell;
    let mut app = Regrate::command();
    eprintln!("Generating completion file for {:?}...", generator);
    let name = app.get_name().to_string();
    generate(generator, &mut app, name, &mut std::io::stdout());
    Ok(())
}

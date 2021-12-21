use crate::Regrate;
use clap::{Args, IntoApp};
use clap_generate::{generate, Shell};
use eyre::Result;

#[derive(Args, Debug)]
#[clap(about, author, version)]
pub struct GenerateArgs {
    /// output completion script
    #[clap(arg_enum)]
    shell: Shell,
}

pub fn generate_completion(args: GenerateArgs) -> Result<()> {
    let generator = args.shell;
    let mut app = Regrate::into_app();
    eprintln!("Generating completion file for {:?}...", generator);
    let name = app.get_name().to_string();
    generate(generator, &mut app, name, &mut std::io::stdout());
    Ok(())
}

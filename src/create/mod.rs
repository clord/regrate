use crate::utils::regrate_path;
use crate::utils::{exists_in_regrate, require_regrate_inited};
use color_eyre::Help;
use eyre::{eyre, Result};
use walkdir::WalkDir;

// #[derive(Args, Debug)]
// #[clap(about, author, version)]
// pub struct CreateArgs {
// }

pub fn do_create() -> Result<()> {
    require_regrate_inited()?;
    if exists_in_regrate("current")? {
        return Err(
            eyre!("Migration already exists").with_suggestion(|| "commit or abandon and try again")
        );
    }

    let template = regrate_path("template")?;
    let current = regrate_path("current")?;

    let res: Result<()> = {
        std::fs::create_dir(&current)?;
        for entry in WalkDir::new(&template) {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                std::fs::create_dir_all(path)?;
            } else {
                let relative = path.strip_prefix(&template)?;
                let dest = current.join(relative);
                std::fs::copy(path, dest)?;
            }
        }
        Ok(())
    };

    match res {
        Ok(()) => Ok(()),
        Err(e) => {
            std::fs::remove_dir_all(current)?;
            Err(e.with_note(|| "no current migration created"))
        }
    }
}

use crate::names;
use crate::utils::{exists_in_regrate, require_regrate_inited};
use color_eyre::Help;
use eyre::{eyre, Result};

pub fn validate() -> Result<()> {
    require_regrate_inited()?;

    let (migrations, pending_name, _) = names::chain()?;

    let orphans = names::orphans(&migrations)?;
    if !orphans.is_empty() {
        return Err(eyre!(
            "{} migration(s) in the store are not reachable from the name chain:\n  {}",
            orphans.len(),
            orphans.join("\n  ")
        )
        .with_note(|| {
            "names are derived from the previous migration's contents, so editing a \
             committed migration invalidates the name of every migration after it"
        })
        .with_suggestion(|| {
            "restore the edited migration from version control, or re-commit the \
             orphaned migrations on top of the chain"
        }));
    }

    for migration in &migrations {
        println!("{:4}  {}", migration.index, migration.name);
    }
    println!("{} migration(s) valid", migrations.len());
    if exists_in_regrate("current")? {
        println!("current migration will commit as {}", pending_name);
    }

    Ok(())
}

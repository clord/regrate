use eyre::Result;

pub fn resolve_conflicts() -> Result<()> {
    //  - in a temp directory,
    //  - create a new migration populated with my old 'current' migration
    //  - revert merge conflict on new current
    //  - commit and make "current"
    println!("RESOLVE");
    Ok(())
}

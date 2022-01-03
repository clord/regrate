use eyre::Result;

pub fn resolve_conflicts() -> Result<()> {
    // upstream is kept. we have to back up to the common ancestor and rebase them? merge them to a
    // single commit that will be left in the user's 'current'? regrate/.gitattributes could be
    // used to set a custom merge rule.

    // OLD PLAN THAT WON'T DEAL WITH MULTIPLE MIGRATIONS TO MERGE:
    //  - in a temp directory,
    //  - create a new migration populated with my old 'current' migration
    //  - revert merge conflict on new current
    //  - commit and make "current"
    println!("RESOLVE");
    Ok(())
}

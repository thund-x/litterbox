use anyhow::Result;
use landlock::{
    ABI, Access, AccessFs, Ruleset, RulesetAttr, RulesetCreatedAttr, path_beneath_rules,
};

pub fn apply_landlock() -> Result<()> {
    let access_all = AccessFs::from_all(ABI::V6);

    let ruleset = Ruleset::default();
    let ruleset = ruleset.handle_access(access_all)?;

    // We avoid giving full access to the container's entire root directory so that we can
    // deny access to "internal" files that Litterbox places within the root directory.
    let read_dir = std::fs::read_dir("/")?;
    let paths: Vec<_> = read_dir
        .filter_map(|e| {
            let path = e.ok()?.path();
            if path.is_dir() { Some(path) } else { None }
        })
        .collect();

    let ruleset = ruleset.create()?;
    let ruleset = ruleset.add_rules(path_beneath_rules(paths, access_all))?;
    let ruleset = ruleset.add_rules(path_beneath_rules(["/"], AccessFs::ReadDir))?;

    match ruleset.restrict_self() {
        Ok(status) => {
            println!(
                "Landlock sandbox applied: {:?}, no_new_privs: {}",
                status.ruleset, status.no_new_privs
            );
        }
        Err(e) => {
            eprintln!("Failed to apply Landlock sandbox: {:?}", e);
        }
    }

    Ok(())
}

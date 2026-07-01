use std::fs;
use std::path::Path;

use crate::{ForgeWelcomeError, Pack};

pub fn load_pack_from_file(path: impl AsRef<Path>) -> Result<Pack, ForgeWelcomeError> {
    let contents = fs::read_to_string(path)?;
    let pack = serde_yaml::from_str::<Pack>(&contents)?;

    Ok(pack)
}

pub fn load_packs_from_dir(path: impl AsRef<Path>) -> Result<Vec<Pack>, ForgeWelcomeError> {
    let mut packs = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        let is_yaml = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext == "yaml" || ext == "yml")
            .unwrap_or(false);

        if is_yaml {
            let pack = load_pack_from_file(&path)?;
            packs.push(pack);
        }
    }

    packs.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(packs)
}

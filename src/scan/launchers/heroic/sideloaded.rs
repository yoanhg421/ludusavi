use std::collections::HashMap;

use crate::prelude::StrictPath;

use crate::{
    prelude::ENV_DEBUG,
    resource::{config::RootsConfig, manifest::Os},
    scan::{
        launchers::{heroic::find_prefix, LauncherGame},
        TitleFinder, TitleQuery,
    },
};

/// `gog_store/library.json` or `store_cache/gog_library.json`
#[derive(serde::Deserialize)]
struct Library {
    games: Vec<LibraryGame>,
}

#[derive(serde::Deserialize)]
pub struct LibraryGame {
    /// This is an opaque ID, not the human-readable title.
    pub app_name: String,
    pub title: String,
    pub install: Install,
    pub folder_name: String,
}

#[derive(serde::Deserialize)]
pub struct Install {
    /// This is an opaque ID, not the human-readable title.
    pub platform: String,
}

pub fn scan(root: &RootsConfig, title_finder: &TitleFinder) -> HashMap<String, LauncherGame> {
    let mut games = HashMap::new();

    let game_titles: HashMap<String, String> = get_library(root)
        .iter()
        .map(|game| (game.app_name.clone(), game.title.clone()))
        .collect();

    if game_titles.is_empty() {
        return games;
    }

    for game in get_library(root) {
        let Some(game_title) = game_titles.get(&game.app_name) else {
            continue;
        };

        let query = TitleQuery {
            names: vec![game_title.to_owned()],
            normalized: true,
            ..Default::default()
        };

        let Some(official_title) = title_finder.find_one(query) else {
            log::trace!("Ignoring unrecognized game: {}, app: {}", &game_title, &game.app_name);
            if std::env::var(ENV_DEBUG).is_ok() {
                eprintln!(
                    "Ignoring unrecognized game from Heroic/GOG: {} (app = {})",
                    &game_title, &game.app_name
                );
            }
            continue;
        };

        log::trace!(
            "Detected game: {} | app: {}, raw title: {}",
            &official_title,
            &game.app_name,
            &game_title
        );
        let prefix = find_prefix(
            &root.path,
            game_title,
            &game.install.platform.to_lowercase(),
            &game.app_name,
        );

        games.insert(
            official_title,
            LauncherGame {
                install_dir: Some(StrictPath::new(game.folder_name.clone())),
                prefix,
                platform: Some(Os::from(game.install.platform.as_str())),
            },
        );
    }

    games
}

pub fn get_library(root: &RootsConfig) -> Vec<LibraryGame> {
    let library = root.path.joined("sideload_apps").joined("library.json");

    let library_path = 'outer: {
        if library.is_file() {
            break 'outer library;
        }

        log::warn!("Could not find library in {:?}", root);
        return vec![];
    };

    match serde_json::from_str::<Library>(&library_path.read().unwrap_or_default()) {
        Ok(gog_library) => {
            log::trace!("Found {} games in {:?}", gog_library.games.len(), &library_path);

            gog_library.games
        }
        Err(e) => {
            log::warn!("Unable to parse library in {:?}: {}", &library_path, e);
            vec![]
        }
    }
}

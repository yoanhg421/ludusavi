use std::env;

use crate::{
    resource::{config::RootsConfig, manifest::Store},
    scan::heroic::{get_gog_games_library, get_legendary_installed_games},
};

/// Tries to find a game with ID `game_id` in the given game roots, actual
/// search algorithm used varies with `game_runner`.  Returns game name or None.
pub fn find_in_roots(roots: &[RootsConfig], game_id: &str, game_runner: &str) -> Option<String> {
    roots
        .iter()
        .filter(|root| root.store == Store::Heroic)
        .find_map(|root| {
            log::debug!("wrap::heroic::find_in_roots: checking root {:?}", root);

            match game_runner {
                "gog" => match get_gog_games_library(root) {
                    Some(gog_games) => gog_games.iter().find_map(|g| match g.app_name == *game_id {
                        true => Some(g.title.clone()),
                        false => None,
                    }),
                    None => None,
                },
                "legendary" => get_legendary_installed_games(root, None)
                    .iter()
                    .find_map(|legendary_game| match legendary_game.app_name == *game_id {
                        true => Some(legendary_game.title.clone()),
                        false => None,
                    }),

                "nile" => {
                    log::warn!("wrap::heroic: heroic runner 'nile' not supported.");
                    None
                }
                "sideload" => {
                    log::warn!("wrap::heroic: heroic runner 'sideload' not supported.");
                    None
                }
                value => {
                    log::warn!("wrap::heroic: unknown heroic runner '{}'", value);
                    None
                }
            }
        })
}

/// Parse environment variables set by heroic (starting with 2.9.2):
///
/// HEROIC_APP_NAME (the ID, not the human-friendly title)
/// HEROIC_APP_RUNNER (one of: gog, legendary, nile, sideload)
/// HEROIC_APP_SOURCE (one of: gog, epic, amazon, sideload)
///
/// We rely on HEROIC_APP_NAME and HEROIC_APP_RUNNER only.
pub fn parse_heroic_2_9_2_environment_variables(roots: &[RootsConfig], _commands: &[String]) -> Option<String> {
    let heroic_app_name = match env::var("HEROIC_APP_NAME") {
        Ok(value) => value,
        Err(_) => return None,
    };

    let heroic_app_runner = match env::var("HEROIC_APP_RUNNER") {
        Ok(value) => value,
        Err(_) => return None,
    };

    log::debug!(
        "wrap::heroic::parse_heroic_2_9_2_environment_variables: found heroic_app_name={}, heroic_app_runner={}",
        heroic_app_name,
        heroic_app_runner,
    );

    find_in_roots(roots, &heroic_app_name, &heroic_app_runner)
}

#[path = "../tests/support/mod.rs"]
mod support;

use avplayer::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let player = support::player("example-avplayer-media-selection-criteria")?;
    let criteria = PlayerMediaSelectionCriteria::with_principal_media_characteristics(
        &[MediaCharacteristic::Audible],
        &["en-US"],
        &[MediaCharacteristic::IsOriginalContent],
    )?;
    player.set_media_selection_criteria(&MediaCharacteristic::Audible, Some(&criteria))?;
    player.set_applies_media_selection_criteria_automatically(true);
    let stored = player
        .media_selection_criteria(&MediaCharacteristic::Audible)?
        .expect("expected stored media-selection criteria");

    println!("preferred languages: {:?}", stored.preferred_languages()?);
    println!(
        "preferred media characteristics: {:?}",
        stored.preferred_media_characteristics()?
    );
    println!(
        "principal media characteristics: {:?}",
        stored.principal_media_characteristics()?
    );
    println!(
        "applies automatically: {}",
        player.applies_media_selection_criteria_automatically()?
    );
    Ok(())
}

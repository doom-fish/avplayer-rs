mod support;

use avplayer::prelude::*;

#[test]
fn avplayer_media_selection_criteria_round_trip() -> support::TestResult {
    let player = support::player("test-avplayer-media-selection-criteria")?;
    let criteria =
        PlayerMediaSelectionCriteria::new(&["en-US"], &[MediaCharacteristic::IsOriginalContent])?;

    assert_eq!(criteria.preferred_languages()?, vec!["en-US".to_string()]);
    assert_eq!(
        criteria.preferred_media_characteristics()?,
        vec![MediaCharacteristic::IsOriginalContent]
    );
    player.set_media_selection_criteria(&MediaCharacteristic::Audible, Some(&criteria))?;
    player.set_applies_media_selection_criteria_automatically(true);
    assert!(player.applies_media_selection_criteria_automatically()?);
    let stored = player
        .media_selection_criteria(&MediaCharacteristic::Audible)?
        .expect("expected stored criteria");
    assert_eq!(stored.preferred_languages()?, vec!["en-US".to_string()]);
    assert_eq!(
        stored.preferred_media_characteristics()?,
        vec![MediaCharacteristic::IsOriginalContent]
    );
    Ok(())
}

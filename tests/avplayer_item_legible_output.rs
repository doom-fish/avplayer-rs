mod support;

use avplayer::prelude::*;

#[test]
fn avplayer_item_legible_output_can_be_attached() -> support::TestResult {
    let subtypes: [u32; 0] = [];
    let item = support::player_item("test-avplayer-item-legible-output")?;
    let output = PlayerItemLegibleOutput::new(Some(&subtypes))?;

    item.add_legible_output(&output)?;
    assert_eq!(item.output_count()?, 1);
    output.set_suppresses_player_rendering(false);
    assert!(!output.suppresses_player_rendering()?);
    output.set_advance_interval_for_delegate_invocation(0.5);
    assert!((output.advance_interval_for_delegate_invocation()? - 0.5).abs() < 1e-9);
    assert!(output.native_representation_subtypes()?.is_empty());
    item.remove_legible_output(&output);
    assert_eq!(item.output_count()?, 0);
    Ok(())
}

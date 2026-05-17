mod support;

use avplayer::prelude::*;

#[test]
fn avplayer_item_metadata_output_can_be_attached() -> support::TestResult {
    let item = support::player_item("test-avplayer-item-metadata-output")?;
    let output = PlayerItemMetadataOutput::new(None::<&[&str]>)?;

    item.add_metadata_output(&output)?;
    assert_eq!(item.output_count()?, 1);
    assert!(!output.has_delegate()?);
    output.set_suppresses_player_rendering(false);
    assert!(!output.suppresses_player_rendering()?);
    assert!(!output.as_output().suppresses_player_rendering());
    let _ = output.as_output().item_time_for_host_time(0.0)?;
    let _ = output.as_output().item_time_for_mach_absolute_time(0)?;
    {
        let _observer = output.observe(Some("tests.avplayer-item-metadata-output"), |_| {})?;
        assert!(output.has_delegate()?);
    }
    output.set_advance_interval_for_delegate_invocation(0.25);
    assert!((output.advance_interval_for_delegate_invocation()? - 0.25).abs() < 1e-9);
    assert!(output.identifiers()?.is_empty());
    item.remove_metadata_output(&output);
    assert_eq!(item.output_count()?, 0);
    Ok(())
}

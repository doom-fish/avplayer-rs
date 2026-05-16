mod support;

use avplayer::prelude::*;

#[test]
fn avplayer_item_rendered_legible_output_can_be_attached() -> support::TestResult {
    let item = support::player_item("test-avplayer-item-rendered-legible-output")?;
    let output = PlayerItemRenderedLegibleOutput::new(Size {
        width: 640.0,
        height: 360.0,
    })?;

    let observer = output.observe(Some("test-rendered-legible-output"), |_| {})?;
    item.add_rendered_legible_output(&output)?;
    assert_eq!(item.output_count()?, 1);
    output.set_suppresses_player_rendering(false);
    assert!(!output.suppresses_player_rendering()?);
    output.set_advance_interval_for_delegate_invocation(0.5);
    assert!((output.advance_interval_for_delegate_invocation()? - 0.5).abs() < 1e-9);
    output.set_video_display_size(Size {
        width: 800.0,
        height: 450.0,
    });
    assert_eq!(
        output.video_display_size()?,
        Size {
            width: 800.0,
            height: 450.0,
        }
    );
    drop(observer);
    item.remove_rendered_legible_output(&output);
    assert_eq!(item.output_count()?, 0);
    Ok(())
}

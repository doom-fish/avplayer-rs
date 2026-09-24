mod support;

use std::time::Duration;

use avplayer::prelude::*;

const PCM_I16_MONO: AudioOutputSettings = AudioOutputSettings::pcm_i16(44_100.0, 1);

fn reader_with_output(
    stem: &str,
    settings: Option<&AudioOutputSettings>,
) -> Result<(AssetReader, AssetReaderTrackOutput), Box<dyn std::error::Error>> {
    let asset = support::loaded_audio_asset(stem)?;
    let track = support::first_audio_track(&asset)?;
    let reader = AssetReader::new(asset.as_asset())?;
    let output = AssetReaderTrackOutput::audio(&track, settings)?;
    reader.add_track_output(&output)?;
    Ok((reader, output))
}

#[test]
fn start_reading_twice_is_an_error() -> support::TestResult {
    let (reader, output) = reader_with_output("test-reader-start-twice", Some(&PCM_I16_MONO))?;
    reader.start_reading()?;
    assert!(matches!(
        reader.start_reading(),
        Err(AVPlayerError::InvalidArgument(_))
    ));
    assert!(output.copy_next_sample_buffer()?.is_some());
    Ok(())
}

#[test]
fn copying_samples_before_reading_starts_is_an_error() -> support::TestResult {
    let (_reader, output) = reader_with_output("test-reader-copy-early", Some(&PCM_I16_MONO))?;
    assert!(matches!(
        output.copy_next_sample_buffer(),
        Err(AVPlayerError::OperationFailed(_))
    ));
    assert!(matches!(
        output.as_output().copy_next_video_pixel_buffer(),
        Err(AVPlayerError::OperationFailed(_))
    ));

    let asset = support::loaded_audio_asset("test-reader-copy-detached")?;
    let track = support::first_audio_track(&asset)?;
    let detached = AssetReaderTrackOutput::passthrough(&track)?;
    assert!(detached.copy_next_sample_buffer().is_err());
    Ok(())
}

#[test]
fn output_configuration_changes_after_start_are_errors() -> support::TestResult {
    let (reader, output) = reader_with_output("test-reader-late-config", Some(&PCM_I16_MONO))?;
    output.set_always_copies_sample_data(false)?;
    reader.start_reading()?;
    assert!(matches!(
        output.set_always_copies_sample_data(true),
        Err(AVPlayerError::OperationFailed(_))
    ));
    assert!(matches!(
        output.as_output().set_supports_random_access(true),
        Err(AVPlayerError::OperationFailed(_))
    ));
    Ok(())
}

#[test]
fn reset_for_reading_rejects_bad_time_ranges() -> support::TestResult {
    support::run_with_deadline(
        "reset_for_reading_rejects_bad_time_ranges",
        Duration::from_secs(90),
        reset_for_reading_rejects_bad_time_ranges_body,
    )
}

fn reset_for_reading_rejects_bad_time_ranges_body() -> support::TestResult {
    let (reader, output) = reader_with_output("test-reader-reset", None)?;
    let borrowed = output.as_output();
    let second = TimeRange::new(Time::new(0, 1), Time::new(1, 10));

    assert!(matches!(
        borrowed.reset_for_reading_time_ranges(&[second]),
        Err(AVPlayerError::InvalidArgument(_))
    ));

    borrowed.set_supports_random_access(true)?;
    let bad_inputs = [
        vec![TimeRange::new(Time::invalid(), Time::new(1, 1))],
        vec![TimeRange::new(Time::new(0, 1), Time::new(-1, 1))],
        vec![TimeRange::new(Time::new(0, 1), Time::indefinite())],
        vec![
            TimeRange::new(Time::new(0, 1), Time::new(2, 1)),
            TimeRange::new(Time::new(1, 1), Time::new(1, 1)),
        ],
        vec![
            TimeRange::new(Time::new(2, 1), Time::new(1, 1)),
            TimeRange::new(Time::new(1, 1), Time::new(1, 2)),
        ],
    ];
    for ranges in &bad_inputs {
        assert!(
            matches!(
                borrowed.reset_for_reading_time_ranges(ranges),
                Err(AVPlayerError::InvalidArgument(_))
            ),
            "{ranges:?}"
        );
    }

    assert!(matches!(
        borrowed.reset_for_reading_time_ranges(&[second]),
        Err(AVPlayerError::OperationFailed(_))
    ));

    reader.start_reading()?;
    assert!(matches!(
        borrowed.reset_for_reading_time_ranges(&[second]),
        Err(AVPlayerError::OperationFailed(_))
    ));
    while output.copy_next_sample_buffer()?.is_some() {}
    borrowed.reset_for_reading_time_ranges(&[second])?;
    assert!(output.copy_next_sample_buffer()?.is_some());
    borrowed.mark_configuration_as_final();
    assert!(matches!(
        borrowed.reset_for_reading_time_ranges(&[second]),
        Err(AVPlayerError::OperationFailed(_))
    ));
    Ok(())
}

#[test]
fn cea608_native_representation_is_rejected() -> support::TestResult {
    let cea608 = u32::from_be_bytes(*b"c608");
    assert!(PlayerItemLegibleOutput::new(Some(&[cea608])).is_err());
    let cea708 = u32::from_be_bytes(*b"c708");
    let output = PlayerItemLegibleOutput::new(Some(&[cea708]))?;
    assert_eq!(output.native_representation_subtypes()?, vec![cea708]);
    Ok(())
}

mod support;

use avplayer::prelude::*;

const fn assert_next_item<T>(_: doom_fish_utils::stream::NextItem<'_, T>) {}

fn assert_caption_validation_stream_api(
    adaptor: &AssetReaderOutputCaptionAdaptor,
) -> Result<(), AVPlayerError> {
    let stream = adaptor.validation_event_stream(8)?;
    let _ = stream.buffered_count();
    let _ = stream.try_next();
    assert_next_item(stream.next());
    stream.clear_buffer();
    Ok(())
}

#[test]
fn asset_download_event_stream_closes_when_session_drops() -> support::TestResult {
    let identifier = format!("tests.avplayer.asset-download.{}", std::process::id());
    let (session, stream) = AssetDownloadURLSession::background_events(&identifier, 8)?;
    assert_eq!(stream.buffered_count(), 0);
    assert!(stream.try_next().is_none());
    assert_next_item(stream.next());
    assert!(!stream.is_closed());
    stream.clear_buffer();
    drop(session);
    assert!(stream.is_closed());
    Ok(())
}

#[test]
fn resource_loader_event_stream_subscribes() -> support::TestResult {
    let asset = support::loaded_audio_asset("async-stream-resource-loader")?;
    let loader = asset.resource_loader();
    let stream = loader.loading_request_stream(8)?;
    assert_eq!(stream.buffered_count(), 0);
    assert!(stream.try_next().is_none());
    assert_next_item(stream.next());
    assert!(!stream.is_closed());
    stream.clear_buffer();
    drop(stream);
    Ok(())
}

#[test]
fn caption_validation_stream_api_compiles() {
    let _: fn(&AssetReaderOutputCaptionAdaptor) -> Result<(), AVPlayerError> =
        assert_caption_validation_stream_api;
}

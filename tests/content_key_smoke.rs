mod support;

use std::thread;
use std::time::{Duration, Instant};

use avplayer::prelude::*;

fn wait_for_event(
    stream: &ContentKeySessionEventStream,
    timeout: Duration,
) -> Result<ContentKeyEvent, Box<dyn std::error::Error>> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(event) = stream.try_next() {
            return Ok(event);
        }
        if Instant::now() >= deadline {
            return Err("timed out waiting for content-key event".into());
        }
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn content_key_clear_key_event_stream_smoke() -> support::TestResult {
    let path = support::audio_path("content-key-smoke")?;
    let asset = UrlAsset::from_file_path(&path)?;
    let session = ContentKeySession::new(&ContentKeySystem::ClearKey)?;

    assert!(asset.may_require_content_keys_for_media_data_processing());
    session.add_content_key_recipient(&asset);
    assert_eq!(session.recipient_count()?, 1);

    let events = session.observe_events(Some("tests.content-key-smoke"), 8)?;
    let identifier = ContentKeyIdentifier::string("clear-key-smoke");
    session.process_content_key_request(Some(&identifier), None, None)?;

    let request = match wait_for_event(&events, Duration::from_secs(5))? {
        ContentKeyEvent::Requested(request) => request,
        other => return Err(format!("expected requested event, got {other:?}").into()),
    };
    assert_eq!(request.status()?, ContentKeyRequestStatus::RequestingResponse);
    assert_eq!(request.identifier()?, Some(identifier.clone()));
    assert_eq!(request.initialization_data()?, None);
    assert!(!request.can_provide_persistable_content_key()?);
    assert!(!request.renews_expiring_response_data()?);

    let specifier = request
        .content_key_specifier()?
        .ok_or("expected content-key specifier on clear-key request")?;
    assert_eq!(specifier.key_system()?, ContentKeySystem::ClearKey);
    assert_eq!(specifier.identifier()?, Some(identifier.clone()));

    let response = ContentKeyResponse::clear_key_data([7_u8; 16], Some(&[9_u8; 16]))?;
    request.process_response(&response)?;

    let succeeded = match wait_for_event(&events, Duration::from_secs(5))? {
        ContentKeyEvent::Succeeded(request) => request,
        other => return Err(format!("expected succeeded event, got {other:?}").into()),
    };
    assert_eq!(succeeded.status()?, ContentKeyRequestStatus::ReceivedResponse);

    let content_key = succeeded
        .content_key()?
        .ok_or("expected content key after successful response processing")?;
    let key_specifier = content_key
        .content_key_specifier()
        .ok_or("expected content-key specifier from content key")?;
    assert_eq!(key_specifier.key_system()?, ContentKeySystem::ClearKey);
    assert_eq!(key_specifier.identifier()?, Some(identifier));
    assert!(matches!(
        content_key.external_content_protection_status()?,
        None | Some(ExternalContentProtectionStatus::Pending)
    ));

    session.remove_content_key_recipient(&asset);
    assert_eq!(session.recipient_count()?, 0);
    Ok(())
}

#[test]
fn content_key_specifier_round_trips() -> support::TestResult {
    let identifier = ContentKeyIdentifier::string("specifier-id");
    let options = ContentKeyRequestOptions {
        protocol_versions: vec![1, 2],
        ..ContentKeyRequestOptions::default()
    };
    let specifier = ContentKeySpecifier::new(
        &ContentKeySystem::ClearKey,
        &identifier,
        Some(&options),
    )?;

    assert_eq!(specifier.key_system()?, ContentKeySystem::ClearKey);
    assert_eq!(specifier.identifier()?, Some(identifier));
    assert_eq!(specifier.options()?, options);
    Ok(())
}

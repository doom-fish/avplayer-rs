mod support;

#[test]
fn avplayer_item_error_log_is_queryable() -> support::TestResult {
    let item = support::player_item("test-avplayer-item-error-log")?;

    if let Some(log) = item.error_log()? {
        let _ = log.extended_log()?;
        let _ = log.extended_log_data_string_encoding()?;
        let _ = log.events()?;
    }

    Ok(())
}

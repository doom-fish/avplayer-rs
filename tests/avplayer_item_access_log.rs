mod support;

#[test]
fn avplayer_item_access_log_is_queryable() -> support::TestResult {
    let item = support::player_item("test-avplayer-item-access-log")?;

    if let Some(log) = item.access_log()? {
        let _ = log.extended_log()?;
        let _ = log.extended_log_data_string_encoding()?;
        let _ = log.events()?;
    }

    Ok(())
}

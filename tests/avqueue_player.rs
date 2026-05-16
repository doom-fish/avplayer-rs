mod support;

use avplayer::prelude::*;

#[test]
fn avqueueplayer_can_insert_and_remove_items() -> support::TestResult {
    let items = support::player_items("test-avqueue-player", 2)?;
    let item_refs = items.iter().collect::<Vec<_>>();
    let queue = QueuePlayer::with_items(&item_refs)?;
    let extra = support::player_item("test-avqueue-player-extra")?;

    assert_eq!(queue.items()?.len(), 2);
    assert!(queue.can_insert_item_after(&extra, None));
    queue.insert_item_after(&extra, None)?;
    assert_eq!(queue.items()?.len(), 3);
    queue.remove_item(&extra);
    assert_eq!(queue.items()?.len(), 2);
    queue.remove_all_items();
    assert!(queue.items()?.is_empty());
    Ok(())
}

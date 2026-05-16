#[path = "../tests/support/mod.rs"]
mod support;

use avplayer::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let items = support::player_items("example-avqueue-player", 2)?;
    let item_refs = items.iter().collect::<Vec<_>>();
    let queue = QueuePlayer::with_items(&item_refs)?;
    let extra = support::player_item("example-avqueue-player-extra")?;

    println!("initial queue items: {}", queue.items()?.len());
    println!(
        "can insert extra at head: {}",
        queue.can_insert_item_after(&extra, None)
    );
    queue.insert_item_after(&extra, None)?;
    println!("after insert: {}", queue.items()?.len());
    queue.advance_to_next_item();
    println!(
        "after advance current item present: {}",
        queue.current_item().is_some()
    );
    queue.remove_all_items();
    println!("after clear: {}", queue.items()?.len());
    Ok(())
}

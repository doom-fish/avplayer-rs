#[path = "../tests/support/mod.rs"]
mod support;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let item = support::player_item("example-avplayer-item-error-log")?;
    match item.error_log()? {
        Some(log) => {
            println!("extended log: {:?}", log.extended_log()?);
            println!("events: {}", log.events()?.len());
        }
        None => println!("no error log available for local synthesized media"),
    }
    Ok(())
}

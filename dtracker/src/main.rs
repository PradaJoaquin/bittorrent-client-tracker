use dtracker::bt_tracker::tracker::{BtTracker, BtTrackerError};

fn main() -> Result<(), BtTrackerError> {
    BtTracker::init()?.run()?;

    Ok(())
}

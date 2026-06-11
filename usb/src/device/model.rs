#[derive(Debug, PartialEq, Copy, Clone)]
pub enum DeviceModel {
    ProII,
    // the Duo and StreamerX share the same firmware, so they should also work
}

// store rodecaster pro 2 pids, vids and model specific stuff
// store rodecaster duo pids, vids and model specific stuff
// store rodecaster streamer x pids, vids and model specific stuff
// store dummy device pids, vids and model specific stuff
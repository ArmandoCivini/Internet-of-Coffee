#[derive(Copy, Clone)]
#[allow(dead_code)]
pub enum State {
    Reading,
    FinishedReading,
    FinishedProcessing,
}

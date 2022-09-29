#[derive(Copy, Clone)]

///Estados posibles del los procesos de productor-consumidor.
pub enum State {
    Reading,
    FinishedReading,
    FinishedProcessing,
}

# Internet of Coffee

## Formato de ordenes

Las ordenes se reciben a travez de un csv en donde cada fila es una orden de un cafe. Cada fila tiene 3 valores; el primero la cantidad de cafe de la orden, el segundo la cantidad de agua caliente y el tercero la cantidad de espuma.

## Procesado de ordenes

Las ordenes se procesan por una thread especial que ejecuta la funcion *producer*. Esta funci

solo recarga un ingrediente a la vez

RUSTFLAGS="--cfg loom" cargo test --release
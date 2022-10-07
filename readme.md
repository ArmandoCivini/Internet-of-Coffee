# Internet of Coffee

## Introducción

Internet of Coffee es un programa que apunta a optimizar la preparación de café en una cafetera con multiples dispensers, para minimizar el tiempo de espera de los clientes haciendo uso de la concurrencia y el modelo de estado mutable compartido.

## Formato de ordenes

Las ordenes se reciben a travez de un csv en donde cada fila es una orden de un cafe. Cada fila tiene 3 valores; el primero la cantidad de cafe de la orden, el segundo la cantidad de agua caliente y el tercero la cantidad de espuma.

## Procesado de ordenes

Las ordenes se procesan por una thread especial que ejecuta la funcion *producer*. Esta función lee las lineas del archivo una por una y las va anexando en el buffer de ordenes. Luego cada thread que ejecuta la función *consumer* retira una orden del buffer para prepararla.

Estas threads ejecutan un patron productor-consumidor haciendo uso de semaforos para evitar los busy waits en los consumidores y garantizar un tamaño maximo del buffer y las lineas leidas del archivo a la vez. El tamaño del buffer esta seteado a 20 pero se puede cambiar a partir de la variable *orders_buffer_size*.

Se implementa un sistema de estados para la finalización de la ejecución de estas threads. el primer cambio de estado se da cuando el productor termina de leer el archivo, se pasa al estado *FinishedReading* y termina la thread productora. Luego cuando una thread consumidora detecta este estado  sumado a que no hay más elementos en el buffer se pasa al estado *FinishedProcessing* y el resto de las threads consumidoras pueden terminar.

## Preparado de las ordenes

Cuando la función *consumer* lee una orden se la pasa a la función *dispenser*, la cual se encarga de preparar esta orden especifica. Lo primero que hace es agarrar los ingredientes con los cuales va a preparar la orden, esto se hace a travez de la función *grab_ingridientes*. Esta función no garantiza conseguir todos los ingredientes necesarios por lo cual se itera hasta conseguirlos.

Los ingredientes de toda la cafetera se encuentran en una instancia de la estructura *Ingridients*. Dentro de la función *grab_ingridientes*, se espera con una variable condicional, a que haya ingredientes para tomar. Si los hay, se obtiene el lock de la estructura y toma los ingredientes que nesecita, de no haber ingredientes suficientes toma todo lo que hay y lo devuelve. Despues de esto se libera el lock.

Luego de haber conseguido los ingredientes necesarios, la función *dispenser* hace el café tomandose un tiempo segun la cantidad de cada ingrediente que lleva la orden.

## Recarga de ingredientes

El programa tiene una thread dedicada a la recarga de ingredientes. Esto se hace a travez de la función *ingridient_reloader*, la cual espera a ser notificada de la falta de ingredientes por la función *wait_missing_ingridients*. Esta función espera a que falta café molido o espuma de leche usando una variable condicional y luego devuelve cual es el ingrediente faltante.

Después se invoca la función *reload* la cual se toma un tiempo para recargar el ingrediente faltante y toma la estructura y la recarga. Es importante remarcar que solo se recarga un ingrediente a la vez. Para la recarga se usan 10 unidades del producto crudo(granos de café, leche fría) para recargar 100 unidades del producto refinado(café molido, espuma de leche). De acabarse algun producto crudo, se recargan 100 unidades de éste automaticamente.

## Estadísticas

Este software emite estadisticas sobre los ingredientes y cafés consumidos hasta el momento, cada cierto tiempo. Cada función que consume estos se encarga de actualizar las estadisticas en la estructura designada.

## Uso

### Ejecución

El codigo se puede ejecutar con el siguiente commando.

``` 
cargo run
```

### Tests

Los tests se corren de la siguiente manera.

``` 
cargo run
```

### Tests de concurrencia

Los tests de concurrencia se pueden correr de la siguiente manera.

``` 
RUSTFLAGS="--cfg loom" cargo test --release
```

### Documentación

Leé la documentación de las funciones ejecutando el comando.

``` 
cargo doc --open
``` 
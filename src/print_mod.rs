#[cfg(not(test))]
///Imprime normalmente. Excepto en tests donde imprime a un archivo.
pub fn print_mod(string: String) {
    println!("{}", string);
}

#[cfg(test)]
use {std::fs::OpenOptions, std::io::prelude::*};
#[cfg(test)]
///Escribe los logs del programa a un archivo.
/// Se asume que el archivo existe, de no ser asi ecurre un error.
pub fn print_mod(string: String) {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("./log/log")
        .expect("no se pudo abrir el archivo");
    writeln!(file, "{}", string).expect("");
}

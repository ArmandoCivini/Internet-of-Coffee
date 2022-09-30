#[cfg(not(test))]
///Imprime normalmente en una corrida normal pero para test escribe a un archivo
pub fn print_mod(string: String) {
    println!("{}", string);
}

#[cfg(test)]
use {std::fs::OpenOptions, std::io::prelude::*};
#[cfg(test)]
pub fn print_mod(string: String) {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("./log")
        .expect("couldnt open file");
    writeln!(file, "{}", string).expect("");
}

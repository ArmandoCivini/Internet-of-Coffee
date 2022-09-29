use std::fmt::{self, Display, Formatter};

///Ingredientes que usa la cafetera.
pub struct Ingridients {
    pub g: i32,
    pub c: i32,
    pub l: i32,
    pub e: i32,
}

impl Display for Ingridients {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "niveles de contenedores: {{granos:{}, cafe:{}, leche:{}, espuma:{}}}",
            self.g, self.c, self.l, self.e
        )
    }
}

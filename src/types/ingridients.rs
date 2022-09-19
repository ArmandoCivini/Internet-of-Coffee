use std::fmt::{self, Display, Formatter};

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
            "container levels{{coffee beans:{}, grounded coffee:{}, milk:{}, foam:{}}}",
            self.g, self.c, self.l, self.e
        )
    }
}

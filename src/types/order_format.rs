use std::fmt::{self, Formatter, Display};

pub struct OrderFormat {
    pub coffee: i32,
    pub hot_water: i32,
    pub foam: i32,
}

impl Display for OrderFormat {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "coffee:{}, hot_water:{}, foam:{}",
               self.coffee, self.hot_water, self.foam)
    }
}
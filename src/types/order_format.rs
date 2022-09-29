use std::fmt::{self, Display, Formatter};

///Formato de la orden de un cafe.
pub struct OrderFormat {
    pub coffee: i32,
    pub hot_water: i32,
    pub foam: i32,
}

impl Display for OrderFormat {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "cafe:{}, agua:{}, espuma:{}",
            self.coffee, self.hot_water, self.foam
        )
    }
}

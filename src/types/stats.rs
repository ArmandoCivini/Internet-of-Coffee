use std::fmt::{self, Display, Formatter};

pub struct Stats {
    pub g_consumed: i32,
    pub c_consumed: i32,
    pub l_consumed: i32,
    pub e_consumed: i32,
    pub water_consumed: i32,
    pub coffee_consumed: i32,
}

impl Display for Stats {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "stats: {{granos usados:{}, cafe usado:{}, leche usada:{}, espuma usada:{}, agua usada:{}, cafe tomado:{}}}",
            self.g_consumed, self.c_consumed, self.l_consumed, self.e_consumed, self.water_consumed, self.coffee_consumed
        )
    }
}

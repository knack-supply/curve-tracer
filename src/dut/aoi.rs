#[derive(Copy, Clone, Debug)]
pub struct AreaOfInterest {
    pub min_v: f64,
    pub max_v: f64,
    pub min_i: f64,
    pub max_i: f64,
}

impl AreaOfInterest {
    pub fn new_pos_i_pos_v(i: f64, v: f64) -> Self {
        Self {
            min_v: 0.0,
            max_v: v,
            min_i: 0.0,
            max_i: i,
        }
    }

    pub fn new_pos_i_neg_v(i: f64, v: f64) -> Self {
        Self {
            min_v: -v,
            max_v: 0.0,
            min_i: 0.0,
            max_i: i,
        }
    }

    pub fn extended(&self) -> Self {
        let slack = 0.1;
        let v_range = self.max_v - self.min_v;
        let i_range = self.max_i - self.min_i;
        Self {
            min_v: self.min_v - v_range * slack,
            max_v: self.max_v + v_range * slack,
            min_i: self.min_i - i_range * slack,
            max_i: self.max_i + i_range * slack,
        }
    }
}

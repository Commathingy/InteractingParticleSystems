pub struct DeVec<T>{
    //note that positive includes 0
    positive : Vec<T>,
    negative : Vec<T>
}

impl<T> DeVec<T>{
    pub fn new() -> DeVec<T>{
        DeVec { 
            positive: Vec::new(), 
            negative: Vec::new()
        }
    }

    //creates new devec with the data in positions 0 to data.len
    pub fn new_with_data(data : Vec<T>) -> DeVec<T>{
        DeVec { 
            positive: data, 
            negative: Vec::new()
        }
    }
    //returns an option with a mutable reference to the data at the given position
    pub fn at_position(& self, position : isize) -> Option<& T>{
        if position < 0 && -position <= (self.negative.len() as isize) {Option::Some(& self.negative[(-position - 1) as usize])}
        else if position >= 0 && position < (self.positive.len() as isize) {Option::Some(& self.positive[position as usize])}
        else {Option::None}
    }

    pub fn at_position_mut(&mut self, position : isize) -> Option<&mut T>{
        if position < 0 && -position <= (self.negative.len() as isize) {Option::Some(&mut self.negative[(-position - 1) as usize])}
        else if position >= 0 && position < (self.positive.len() as isize) {Option::Some(&mut self.positive[position as usize])}
        else {Option::None}
    }
    //add data on the given side of zero
    pub fn add_data(&mut self, data:&mut Vec<T>, positive : bool){
        if positive {
            self.positive.append(data);
        }
        else {
            self.negative.append(data);
        }
    }

    pub fn data_range(&self) -> std::ops::Range<isize> {
        (-(self.negative.len() as isize))..(self.positive.len() as isize)
    }
}  
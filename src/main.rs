use minifb::{Key, Window, WindowOptions};
use std::thread;
use rand::Rng;
use rand_distr::{Distribution,Exp};

mod devec;

enum Direction{
    Left,
    Right
}

struct Swap{
    direction : Direction,
    timestamp : f32
}

impl Swap{
    fn new(dir : Direction, time : f32) -> Swap{
        Swap{
            direction : dir,
            timestamp : time
        }
    }
}

type SwapState = devec::DeVec<Vec<Swap>>;
type ColourState = devec::DeVec<Vec<Option<u32>>>;

impl ColourState{

    //note that discrete starting time represents which block of time we are calculating for (since we are assuming that each horizontal pixel corresponds to a block of time)
    fn backtrace_until(&self, discrete_starting_time : usize, position : isize, swap_data : &SwapState) -> Option<u32> {
        //start at current_time and work backwards (it monotonically decreases) towards min_time
        let mut current_time = (discrete_starting_time as f32) * MAXTIME/(WIDTH as f32);
        let min_time: f32 = ((discrete_starting_time  - 1) as f32) * MAXTIME/(WIDTH as f32);
        let mut current_position = position;

        'outer : while current_time>min_time{
            //look at the data at position current_position
            match swap_data.at_position(current_position) {
                //if we have data here, it should be a vectors of swaps, otherwise it is a None, indicating we have not yet even created anything at that position
                Some(inner_vec) => {
                    //we want to start at largest time, so we iterate over the vector in reverse (since it should be ordered by time already)
                    for swap in inner_vec.iter().rev(){
                        //if the timestamp of the swap is less than our current time, but before the min_time, we will follow the swap
                        if swap.timestamp < current_time && swap.timestamp > min_time {
                            current_time=swap.timestamp; 
                            current_position = current_position + (match swap.direction {Direction::Left => -1, Direction::Right => 1});
                            //we continue on outer here as we have moved the current position, so we need to restart the for loop for this position,
                            //or stop if this new position is a None
                            continue 'outer;                                    
                        }
                    }
                    
                    //now that we have followed all our swaps, we have now reached some position (current_position)
                    //we check the colour at this position at the previous time step, which should hopefully have already been filled in
                    //if it has we just copy that colour, otherwise we use a None
                    match self.at_position(current_position).unwrap().get(discrete_starting_time-1){
                        Some(u) => return *u,
                        None => return None
                    };
                },
                //since we have no data in this position, we set the colour as None to indicate this
                None => return None
            }
        }
        None
    }

    fn build_initial_state(&mut self, swap_data : &SwapState){
        //i represents the "point in time" we are calculating for
        //note that we start on 1 for i since we should already have the initial state data
        for i in 1..WIDTH{
            //j represents the position we are calculating for
            for j in self.data_range(){
                let value = self.backtrace_until(i, j, swap_data);
                self.at_position_mut(j).unwrap().push(value);
            }
        }
    }
}

//turns u8 r,g and b values into the corresponding u32 value
fn from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    (r << 16) | (g << 8) | b
}

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;
const NTHREADS : usize = 10;
const MAXTIME : f32 = 20.0;
const RATEPARAMETER : f32 = 1.0;

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let title = format!("Simple Voter Model, Maxtime = {MAXTIME}");
    let mut window = Window::new(
        &title,
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));


    //create vector with handles so we can close threads later and get the returned values
    let mut handles : Vec<thread::JoinHandle<Vec<Vec<Swap>>>> = Vec::new();
    //iterate over number of threads to create
    for i in 0..NTHREADS {
        //calculate how big of a sample each thread will do (so they are roughly equal)
        let length_of_sample = HEIGHT/NTHREADS + if i < HEIGHT%NTHREADS { 1 } else { 0 };
        //create the thread
        handles.push(thread::spawn(move || {
            //create the vector that will be the threads output
            let mut sample : Vec<Vec<Swap>> = Vec::new(); 
            //start the RNG generator
            let mut rng = rand::thread_rng();
            let exp = Exp::new(2.0 * RATEPARAMETER).unwrap();
            //iterate over each position in the sample, creating that vector as we go.
            for i in 0..length_of_sample {
                sample.push(Vec::new());
                let mut time = 0.0;
                while time < MAXTIME{
                    //randomly determine the direction and time for the swap
                    time = time + exp.sample(&mut rng);
                    let dir = if rng.gen_range(0..2) == 0 {Direction::Left} else {Direction::Right};
                    sample[i].push(Swap::new(dir,time))
                }
            }
            sample
        }));
    }
    
    //close the threads and use the data returned from them to create the full set of data
    let mut full_sample: devec::DeVec<Vec<Swap>> = devec::DeVec::new();
    for handle in handles {
        full_sample.add_data(&mut handle.join().unwrap(), true);
    }

    let mut rng = rand::thread_rng();
    
    //create a random initial state
    let initial_state: Vec<Vec<Option<u32>>> = (0..HEIGHT).map(|_|{
        let state = rng.gen_range(0..2);
        if state == 0 {vec!(Some(from_u8_rgb(255, 255, 255)))} else {vec!(Some(from_u8_rgb(0 as u8, 0, 0)))}
    }).collect();

    //push that random initial state into the numberline that stores the states in a way easily readable in screen space
    let mut coloured_sample: ColourState = devec::DeVec::new_with_data(initial_state);
    
    //actually create the buffer using the update self backtracing method
    coloured_sample.build_initial_state(&full_sample);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for (ind, buff_elem) in buffer.iter_mut().enumerate() {
            //unwrap the colour, but if it is none, we give it purple colour
            *buff_elem = coloured_sample.at_position((ind/WIDTH) as isize).unwrap().get(ind%WIDTH).unwrap().unwrap_or_else(|| from_u8_rgb(255, 0, 255)); 
        }

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}


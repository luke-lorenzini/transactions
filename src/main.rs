use std::{env, fs::File};

use log::debug;

use transactions::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    let input_file = &args[1];
    debug!("{:?}", input_file);

    let file = File::open(input_file);
    let file = file.unwrap();

    let xxx = read_input_file(file).unwrap();

    display_output(xxx);
}

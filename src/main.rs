use std::collections::HashMap;
use chrono::Local;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let name = args.get(1).expect("Name not provided");
    println!("Hello, {name}!");
    set_github_output_env();
}

fn set_github_output_env() {
    let current_time = Local::now().to_string();
    let env_val = format!("time={current_time}");
    env::set_var("GITHUB_OUTPUT", env_val);
}
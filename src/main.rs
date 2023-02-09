use chrono::Local;
use std::env;
use std::fs::write;

fn main() {
    set_github_output_env();
}

fn set_github_output_env() {
    let current_time = Local::now().to_string();
    let env_val = format!("time={current_time},pr_number=[PR-NUM]");
    let github_output_path = env::var("GITHUB_OUTPUT").unwrap();
    write(github_output_path, env_val).unwrap();
}

use chrono::Local;
use std::env;
use std::fs::write;
use serde_json::json;

fn main() {
    set_github_output_env();
}

fn set_github_output_env() {
    let current_time = Local::now().to_string();
    // let env_val = format!("time={current_time},pr_number=[PR-NUM]");
    let outputs = json!({
        "time": current_time,
        "pr_number": "[PR-NUM]",
    });
    let github_output_path = env::var("GITHUB_OUTPUT");
    match github_output_path {
        Ok(v) => write(v, outputs.to_string()).unwrap(),
        Err(_e) => env::set_var("GITHUB_OUTPUT", outputs.to_string()),
    }
}

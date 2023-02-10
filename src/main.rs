use std::env;
use std::fs::write;

fn main() {
    let args: Vec<String> = env::args().collect();
    let empty_msg = String::from("<EMPTY>");
    let pull_req_obj = args.get(1).unwrap_or(&empty_msg);
    println!("Pull req obj: {}", pull_req_obj);
    set_github_output_env();
    print_pr_link();
}

fn set_github_output_env() {
    let env_val = format!("pr_number=[PR-NUM]");
    let github_output_path = env::var("GITHUB_OUTPUT");
    match github_output_path {
        Ok(v) => write(v, env_val).unwrap(),
        Err(_e) => env::set_var("GITHUB_OUTPUT", env_val),
    }
}

fn print_pr_link() {
    println!(
        "Github token: {}",
        env::var("GITHUB_TOKEN").unwrap_or("<missing>".to_string())
    );
}

use std::env;
use std::fs::{File, write};
use std::io::Read;

fn main() {
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

/// `GITHUB_EVENT_PATH`
///
/// The path to the file on the runner that contains the full event webhook payload.
/// For example, `/github/workflow/event.json`.
///
/// https://docs.github.com/en/actions/learn-github-actions/variables
fn print_pr_link() {
    println!(
        "Github token: {}",
        env::var("GITHUB_TOKEN").unwrap_or("<missing>".to_string())
    );
    let event_path = env::var("GITHUB_EVENT_PATH");
    if let Err(e) = event_path {
        panic!("GITHUB_EVENT_PATH not found.");
    }
    let p = event_path.unwrap();
    let mut f = File::open(p).unwrap();
    let mut buf = String::new();
    f.read_to_string(&mut buf).unwrap();
    println!("Data from GITHUB_EVENT_PATH: {}", buf);
}

use octocrab::models::AppId;
use std::env;
use std::fs::File;
use std::io::Read;

pub struct PemContents(pub String);
pub struct LibRepoName(pub String);

pub fn read_cli_args() -> Result<(AppId, PemContents, LibRepoName), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        return Err(String::from(
            "Usage: program <app_id> <pem file contents> <lib_repo_name>",
        ));
    }

    let app_id = match args[1].parse::<u64>() {
        Ok(n) => n,
        Err(_) => {
            return Err(format!("Invalid input: {}", args[1]));
        }
    };

    let pem_contents = &args[2];

    let lib_name = &args[3];

    Ok((
        AppId(app_id),
        PemContents(pem_contents.clone()),
        LibRepoName(lib_name.clone()),
    ))
}

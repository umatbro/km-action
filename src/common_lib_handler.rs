use jsonwebtoken::EncodingKey;
use octocrab;
use octocrab::models::{AppId, InstallationToken};
use octocrab::params::apps::CreateInstallationAccessToken;
use octocrab::{Octocrab, OctocrabBuilder};

#[derive(Debug)]
pub struct GithubSetupError {
    pub jsonwebtoken_error: Option<jsonwebtoken::errors::Error>,
    pub github_error: Option<octocrab::Error>,
}

impl From<octocrab::Error> for GithubSetupError {
    fn from(e: octocrab::Error) -> Self {
        Self {
            github_error: Some(e),
            jsonwebtoken_error: None,
        }
    }
}

impl From<jsonwebtoken::errors::Error> for GithubSetupError {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        Self {
            github_error: None,
            jsonwebtoken_error: Some(e),
        }
    }
}

/// Get Octocrab instance to query the lib repository.
///
/// Steps:
/// * Authenticate with App key and app private key
/// * Retrieve installations
/// * Create access token for the lib repository.
/// * Return octocrab instance authenticated with access_token for the lib repository.
///
/// # Panics
///
/// The function will panic if provided private key fails to authenticate.
///
/// # Useful resources
///
/// * https://stackoverflow.com/questions/66509694/unable-to-access-github-api-getting-bad-credentials-error
/// * Octocrab examples https://github.com/XAMPPRocky/octocrab/blob/master/examples/github_app_authentication_manual.rs
pub async fn get_octocrab_instance_for_lib_repo(
    app_id: AppId,
    private_key: &[u8],
    lib_repo_name: &str,
) -> Result<Octocrab, GithubSetupError> {
    let key = EncodingKey::from_rsa_pem(private_key)?;

    let token = octocrab::auth::create_jwt(app_id, &key)?;

    let crab = OctocrabBuilder::new().personal_token(token).build()?;

    Ok(get_client_for_repo_from_installations(&crab, lib_repo_name).await?)
}

/// This function queries app installations and attempts to retrieve an access token for the requested
/// `repo_name`.
///
/// # Returns
///
/// An octocrab client with authentication for the requested repo.
pub async fn get_client_for_repo_from_installations(
    octocrab_: &Octocrab,
    repo_name: &str,
) -> Result<Octocrab, GithubSetupError> {
    let installations = octocrab_.apps().installations().send().await?.take_items();
    let mut create_access_token = CreateInstallationAccessToken::default();
    create_access_token.repositories = vec![String::from(repo_name)];

    let access_to_repo: InstallationToken = octocrab_
        .post(
            installations[0].access_tokens_url.as_ref().unwrap(),
            Some(&create_access_token),
        )
        .await?;

    let octocrab_for_repo = OctocrabBuilder::new()
        .personal_token(access_to_repo.token)
        .build()?;

    Ok(octocrab_for_repo)
}

#[cfg(test)]
mod tests {
    use crate::common_lib_handler::get_octocrab_instance_for_lib_repo;
    use octocrab::models::AppId;

    use std::fs::File;
    use std::io::Read;

    /// Test is ignored because it makes real requests against GitHub API. It should only be run locally.
    /// To run only this test, use command:
    ///
    /// ```
    /// cargo test initialize_octocrab -- --ignored
    /// ```
    #[ignore]
    #[tokio::test]
    async fn initialize_octocrab() {
        let mut pk = File::open("km-common-lib-syncer.private-key.pem").unwrap();
        let mut contents = Vec::new();
        pk.read_to_end(&mut contents).unwrap();
        let pk = contents.as_slice();
        let octo = get_octocrab_instance_for_lib_repo(AppId(293643), pk, "km-dep")
            .await
            .unwrap();
        let repo = octo.repos("umatbro", "km-dep").get().await.unwrap();
        println!("REPO {:?}", repo);

        let pulls = octo.pulls("umatbro", "km-dep").list().send().await.unwrap();
        assert_eq!(pulls.into_iter().len(), 1);
    }
}

use octocrab;
use octocrab::Octocrab;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize, Debug)]
pub struct Event {
    pub pull_request: PullRequest,
    pub repository: Repository,
}

#[derive(Deserialize, Debug)]
pub struct PullRequest {
    pub number: i32,
    pub body: String,
    pub title: String,
}

#[derive(Deserialize, Debug)]
pub struct Repository {
    pub name: String,
    pub full_name: String,
}

impl Repository {
    pub fn get_owner(&self) -> Result<String, String> {
        let parts: Vec<&str> = self.full_name.split('/').collect();
        if parts.len() != 2 {
            return Err(String::from(&self.full_name));
        }
        let result = parts[0];
        Ok(result.to_string())
    }
}

impl Event {
    pub async fn set_pr_body(
        &self,
        octo: Arc<Octocrab>,
        body: &String,
    ) -> octocrab::Result<octocrab::models::pulls::PullRequest> {
        let result = octo
            .pulls(&self.repository.get_owner().unwrap(), &self.repository.name)
            .update(2)
            .body(body)
            .send()
            .await;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::Event;
    use serde_json;
    use std::fs::File;
    use std::io::BufReader;

    #[test]
    fn test_parse_event() {
        let file = File::open("src/test_resources/action_payload.json").unwrap();
        let reader = BufReader::new(file);
        let result: Event = serde_json::from_reader(reader).unwrap();

        assert_eq!(result.pull_request.number, 2);
        assert_eq!(result.pull_request.body, "next trigger");
        assert_eq!(result.pull_request.title, "update cargo");
        assert_eq!(result.repository.name, "km-dep");
        assert_eq!(result.repository.get_owner().unwrap(), "umatbro");
        assert_eq!(result.repository.full_name, "umatbro/km-dep");
    }
}

use configparser::ini::Ini;
use home::home_dir;
use serde::{Serialize, Deserialize};
use reqwest::blocking::Client;
use reqwest::StatusCode;


#[derive(Serialize, Deserialize, Debug)]
struct Repo {
    name: Option<String>,
    full_name: String,
    description: String,
    visibility: Option<String>,
    import_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Provider {
    protocol: String,
    host: String,
    token: String,
    user: String,
    pass: String,
    endpoint: String,
}

impl Provider {
    fn get_repos(&mut self) -> Vec<Repo> {
        let client = Client::new();
        let endpoint = format!("{protocol}{host}{endpoint}",
            protocol=self.protocol,
            host=self.host,
            endpoint=self.endpoint);
        let res = client.get(endpoint)
            .header("Content-type", "application/json")
            .header("Accept", "application/json")
            .header("Authorization", 
                format!("token {token}", token=&self.token))
            .send();
        match res {
            Ok(parsed) => {
                // println!("{:?}", parsed.text());
                parsed.json::<Vec<Repo>>().unwrap()
            }
            Err(e) => panic!("Err: {:?}", e),
        }
    }

    fn put_repo(&mut self, repo: &Repo) -> reqwest::blocking::Response {
        println!("   Importing: {:?}", &repo.name.as_ref().unwrap());
        let client = Client::new();
        let endpoint = format!("{protocol}{host}{endpoint}",
            protocol=self.protocol,
            host=self.host,
            endpoint=self.endpoint);
        println!("{:?}", endpoint);
        let res = client.post(endpoint)
            .header("Private-Token", 
                format!("{token}", token=&self.token))
            .json(&repo)
            .send();
        res.unwrap()
    }
}

fn main() {
    // Read the config file
    let home = match home_dir() {
        Some(path) => path,
        None => panic!("Impossible to get your home dir!"),
    };
    let mut config = Ini::new();
    config.load(
        format!("{home}/.config/gogs2gitlab/gogs2gitlab.ini",
            home=home.display())
        ).unwrap();
    // Define providers
    let mut gogs = Provider{
        protocol: config.get("default", "gogs_proto").unwrap(),
        host: config.get("default", "gogs_host").unwrap(),
        endpoint: "/api/v1/user/repos".to_string(),
        token: config.get("default", "gogs_token").unwrap(),
        user: config.get("default", "gogs_user").unwrap(),
        pass: config.get("default", "gogs_pass").unwrap(),
    };
    let mut gitlab = Provider{
        protocol: config.get("default", "gitlab_proto").unwrap(),
        host: config.get("default", "gitlab_host").unwrap(),
        endpoint: "/api/v4/projects".to_string(),
        token: config.get("default", "gitlab_token").unwrap(),
        user: "".to_string(),
        pass: "".to_string(),
    };
    // Start the game
    let gogs_repos : Vec<Repo> = gogs.get_repos();
    for mut repo in gogs_repos {
        println!("➡️  {:?}", &repo.name.as_ref().unwrap());
        repo.visibility = Some("private".to_string());
        repo.import_url = Some(
            format!("{protocol}{user}:{pass}@{host}/{repo}",
            protocol=gogs.protocol,
            user=gogs.user,
            pass=gogs.pass,
            host=gogs.host,
            repo=repo.full_name));
        let added = gitlab.put_repo(&repo);
        // println!("{:?} {:?}", added.status(), added.text());
        match added.status() {
            StatusCode::CREATED => println!("✅ OK"),
            err => println!("❌ KO: Error {:?}, something happened", err),
        }
    }
}

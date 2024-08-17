use anyhow::Context;
use octocrab::Octocrab;
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;
use {once_cell::sync::Lazy, regex::Regex};

mod client;
pub mod client_cache;
pub(crate) mod runner;
pub mod webhook_models;

pub fn create_gh_app_client(app_id: u64, key: &str) -> anyhow::Result<Octocrab> {
    Ok(Octocrab::builder()
        .app(
            app_id.into(),
            jsonwebtoken::EncodingKey::from_rsa_pem(key.as_bytes())?,
        )
        .build()?)
}

pub struct GitHubUserLogged {
    pub login: String,
    pub name: Option<String>,
    pub avatar_url: String,
    pub email: Option<String>,
}

pub fn url_to_slug(url: &str) -> Option<GitRepoSlug> {
    #[allow(clippy::expect_used)]
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"https://github.com/(?<org>[^/]+)/(?<repo>[^/]+)").expect("Infallible !")
    });
    RE.captures(url).map(|caps| GitRepoSlug {
        org: caps["org"].to_owned(),
        repo: caps["repo"].to_owned(),
    })
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct GitRepoSlug {
    pub org: String,
    pub repo: String,
}

impl fmt::Display for GitRepoSlug {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.org, self.repo)
    }
}

impl FromStr for GitRepoSlug {
    type Err = anyhow::Error;

    fn from_str(slug: &str) -> Result<Self, Self::Err> {
        #[allow(clippy::expect_used)]
        static RE: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(?<org>[^/]+)/(?<repo>[^/]+)").expect("Infallible !"));
        RE.captures(slug)
            .map(|caps| Self {
                org: caps["org"].to_owned(),
                repo: caps["repo"].to_owned(),
            })
            .context("Unparseable runner repo slug")
    }
}

#[cfg(test)]
mod tests {
    use crate::github::{url_to_slug, GitRepoSlug};

    #[test]
    fn gitlab_url_not_matching() {
        let result = url_to_slug("https://gitlab.com/gitlab-org/gitlab");
        pretty_assertions::assert_eq!(result, None);
    }

    #[test]
    fn org_url_not_matching() {
        let result = url_to_slug("https://github.com/korekto");
        pretty_assertions::assert_eq!(result, None);
    }

    #[test]
    fn repo_url_matching() {
        let result = url_to_slug("https://github.com/korekto/korekto-shuttle");
        pretty_assertions::assert_eq!(
            result,
            Some(GitRepoSlug {
                org: "korekto".to_string(),
                repo: "korekto-shuttle".to_string()
            })
        );
    }
}

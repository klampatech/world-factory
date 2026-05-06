//! GitHub API integration for PR review operations
//!
//! This module provides a client for interacting with GitHub's REST API
//! to support agent-based PR review workflows.
//!
//! # Authentication
//!
//! Configure GitHub credentials via environment variables:
//! - `GITHUB_TOKEN`: Personal Access Token (PAT) with `repo` scope
//! - `GITHUB_API_URL`: Optional, defaults to `https://api.github.com`
//!
//! # Capabilities
//!
//! - Read: List PRs, get PR details, get diff, get comments, list commits
//! - Write: Leave PR comments, approve PRs, request changes

use serde::{Deserialize, Serialize};
use std::env;

/// GitHub API client for PR review operations
#[derive(Clone)]
pub struct GitHubClient {
    token: String,
    api_url: String,
}

/// Configuration for GitHub API credentials
#[derive(Debug, Clone)]
pub struct GitHubConfig {
    pub token: String,
    pub api_url: Option<String>,
}

impl Default for GitHubConfig {
    fn default() -> Self {
        Self {
            token: env::var("GITHUB_TOKEN").unwrap_or_default(),
            api_url: env::var("GITHUB_API_URL").ok(),
        }
    }
}

impl GitHubConfig {
    /// Create a new config from environment variables
    pub fn from_env() -> Self {
        Self::default()
    }

    /// Create a config with explicit credentials
    pub fn new(token: impl Into<String>, api_url: Option<String>) -> Self {
        Self {
            token: token.into(),
            api_url,
        }
    }

    /// Check if the config has valid credentials
    pub fn is_configured(&self) -> bool {
        !self.token.is_empty()
    }
}

impl GitHubClient {
    /// Create a new client from configuration
    pub fn from_config(config: &GitHubConfig) -> Result<Self, GitHubError> {
        if !config.is_configured() {
            return Err(GitHubError::NotConfigured(
                "GITHUB_TOKEN not set".to_string(),
            ));
        }

        Ok(Self {
            token: config.token.clone(),
            api_url: config
                .api_url
                .unwrap_or_else(|| "https://api.github.com".to_string()),
        })
    }

    /// Create a new client from environment variables
    pub fn from_env() -> Result<Self, GitHubError> {
        Self::from_config(&GitHubConfig::from_env())
    }

    /// Create a client with explicit credentials
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            api_url: "https://api.github.com".to_string(),
        }
    }

    fn request(&self, method: &str, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.api_url, path);
        reqwest::Client::new()
            .request(method.parse().unwrap(), &url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "WorldFactory/1.0")
    }

    /// List open pull requests for a repository
    ///
    /// # Arguments
    /// * `owner` - Repository owner (user or org)
    /// * `repo` - Repository name
    /// * `state` - Filter by state: "open", "closed", "all"
    pub async fn list_prs(
        &self,
        owner: &str,
        repo: &str,
        state: Option<&str>,
    ) -> Result<Vec<PullRequest>, GitHubError> {
        let state_param = state.unwrap_or("open");
        let path = format!("/repos/{}/{}/pulls?state={}", owner, repo, state_param);

        let response = self
            .request("GET", &path)
            .send()
            .await
            .map_err(GitHubError::RequestFailed)?;

        if !response.status().is_success() {
            return Err(GitHubError::ApiError {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }

        response
            .json::<Vec<PullRequest>>()
            .await
            .map_err(GitHubError::ParseFailed)
    }

    /// Get details of a specific pull request
    pub async fn get_pr(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
    ) -> Result<PullRequest, GitHubError> {
        let path = format!("/repos/{}/{}/pulls/{}", owner, repo, pr_number);

        let response = self
            .request("GET", &path)
            .send()
            .await
            .map_err(GitHubError::RequestFailed)?;

        if !response.status().is_success() {
            return Err(GitHubError::ApiError {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }

        response
            .json::<PullRequest>()
            .await
            .map_err(GitHubError::ParseFailed)
    }

    /// Get the diff of a pull request
    pub async fn get_pr_diff(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
    ) -> Result<String, GitHubError> {
        let path = format!("/repos/{}/{}/pulls/{}", owner, repo, pr_number);

        let response = self
            .request("GET", &path)
            .header("Accept", "application/vnd.github.v3.diff")
            .send()
            .await
            .map_err(GitHubError::RequestFailed)?;

        if !response.status().is_success() {
            return Err(GitHubError::ApiError {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }

        response.text().await.map_err(GitHubError::ParseFailed)
    }

    /// Get comments on a pull request
    pub async fn get_pr_comments(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
    ) -> Result<Vec<PullRequestComment>, GitHubError> {
        let path = format!("/repos/{}/{}/pulls/{}/comments", owner, repo, pr_number);

        let response = self
            .request("GET", &path)
            .send()
            .await
            .map_err(GitHubError::RequestFailed)?;

        if !response.status().is_success() {
            return Err(GitHubError::ApiError {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }

        response
            .json::<Vec<PullRequestComment>>()
            .await
            .map_err(GitHubError::ParseFailed)
    }

    /// List commits in a pull request
    pub async fn list_pr_commits(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
    ) -> Result<Vec<Commit>, GitHubError> {
        let path = format!("/repos/{}/{}/pulls/{}/commits", owner, repo, pr_number);

        let response = self
            .request("GET", &path)
            .send()
            .await
            .map_err(GitHubError::RequestFailed)?;

        if !response.status().is_success() {
            return Err(GitHubError::ApiError {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }

        response
            .json::<Vec<Commit>>()
            .await
            .map_err(GitHubError::ParseFailed)
    }

    /// Leave a comment on a pull request
    ///
    /// # Arguments
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `pr_number` - Pull request number
    /// * `body` - Comment body (supports Markdown)
    /// * `commit_id` - Optional: SHA of the commit to comment on
    /// * `path` - Optional: File path for line comment
    /// * `line` - Optional: Line number for line comment
    pub async fn create_pr_comment(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        body: &str,
        commit_id: Option<&str>,
        path: Option<&str>,
        line: Option<u64>,
    ) -> Result<PullRequestComment, GitHubError> {
        let path_url = format!("/repos/{}/{}/pulls/{}/comments", owner, repo, pr_number);

        #[derive(Serialize)]
        struct CreateCommentRequest<'a> {
            body: &'a str,
            commit_id: Option<&'a str>,
            path: Option<&'a str>,
            line: Option<u64>,
            side: Option<&'a str>,
        }

        let request_body = CreateCommentRequest {
            body,
            commit_id,
            path,
            line,
            side: None,
        };

        let response = self
            .request("POST", &path_url)
            .json(&request_body)
            .send()
            .await
            .map_err(GitHubError::RequestFailed)?;

        if !response.status().is_success() {
            return Err(GitHubError::ApiError {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }

        response
            .json::<PullRequestComment>()
            .await
            .map_err(GitHubError::ParseFailed)
    }

    /// Approve a pull request
    ///
    /// # Arguments
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `pr_number` - Pull request number
    /// * `body` - Optional approval comment
    pub async fn approve_pr(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        body: Option<&str>,
    ) -> Result<ReviewResponse, GitHubError> {
        self.submit_review(owner, repo, pr_number, "APPROVE", body)
            .await
    }

    /// Request changes on a pull request
    ///
    /// # Arguments
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `pr_number` - Pull request number
    /// * `body` - Required explanation of requested changes
    pub async fn request_changes(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        body: &str,
    ) -> Result<ReviewResponse, GitHubError> {
        self.submit_review(owner, repo, pr_number, "REQUEST_CHANGES", Some(body))
            .await
    }

    /// Submit a PR review
    async fn submit_review(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        event: &str,
        body: Option<&str>,
    ) -> Result<ReviewResponse, GitHubError> {
        let path = format!("/repos/{}/{}/pulls/{}/reviews", owner, repo, pr_number);

        #[derive(Serialize)]
        struct ReviewRequest<'a> {
            event: &'a str,
            body: Option<&'a str>,
        }

        let request_body = ReviewRequest { event, body };

        let response = self
            .request("POST", &path)
            .json(&request_body)
            .send()
            .await
            .map_err(GitHubError::RequestFailed)?;

        if !response.status().is_success() {
            return Err(GitHubError::ApiError {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }

        response
            .json::<ReviewResponse>()
            .await
            .map_err(GitHubError::ParseFailed)
    }

    /// Add a general comment to a PR (not a review)
    pub async fn add_general_comment(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        body: &str,
    ) -> Result<IssueComment, GitHubError> {
        let path = format!("/repos/{}/{}/issues/{}/comments", owner, repo, pr_number);

        #[derive(Serialize)]
        struct CommentRequest<'a> {
            body: &'a str,
        }

        let response = self
            .request("POST", &path)
            .json(&CommentRequest { body })
            .send()
            .await
            .map_err(GitHubError::RequestFailed)?;

        if !response.status().is_success() {
            return Err(GitHubError::ApiError {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }

        response
            .json::<IssueComment>()
            .await
            .map_err(GitHubError::ParseFailed)
    }
}

/// Errors that can occur during GitHub API operations
#[derive(Debug, thiserror::Error)]
pub enum GitHubError {
    #[error("GitHub integration not configured: {0}")]
    NotConfigured(String),

    #[error("Request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("GitHub API error: {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("Failed to parse response: {0}")]
    ParseFailed(#[from] serde_json::Error),
}

// Data structures for GitHub API responses

/// A pull request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRequest {
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub html_url: String,
    pub user: User,
    pub head: PullRequestRef,
    pub base: PullRequestRef,
    pub commits_url: String,
    pub comments_url: String,
    pub review_comments_url: String,
    pub created_at: String,
    pub updated_at: String,
    pub merged_at: Option<String>,
    pub mergeable: Option<bool>,
    pub mergeable_state: Option<String>,
}

/// A GitHub user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub login: String,
    pub id: u64,
    pub avatar_url: String,
}

/// A pull request head or base reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestRef {
    pub ref_name: String,
    pub sha: String,
}

/// A pull request comment (on a specific line/diff)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestComment {
    pub id: u64,
    pub body: String,
    pub path: Option<String>,
    pub line: Option<u64>,
    pub commit_id: String,
    pub user: User,
    pub created_at: String,
    pub updated_at: String,
}

/// A commit in a PR
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Commit {
    pub sha: String,
    pub message: String,
    pub author: CommitAuthor,
    pub url: String,
}

/// Author of a commit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitAuthor {
    pub name: String,
    pub email: String,
    pub date: String,
}

/// Response from submitting a review
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewResponse {
    pub id: u64,
    pub body: Option<String>,
    pub state: String,
    pub user: User,
    pub submitted_at: Option<String>,
}

/// An issue/PR comment (general comment, not a review)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueComment {
    pub id: u64,
    pub body: String,
    pub user: User,
    pub created_at: String,
    pub updated_at: String,
    pub url: String,
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_config_defaults() {
        let config = GitHubConfig::default();
        // Token will be empty if env var not set
        assert!(config.api_url.is_none() || config.api_url == Some("https://api.github.com".to_string()));
    }

    #[test]
    fn test_github_config_is_configured() {
        let mut config = GitHubConfig::default();
        assert!(!config.is_configured());

        config.token = "test-token".to_string();
        assert!(config.is_configured());
    }

    #[test]
    fn test_github_client_requires_token() {
        let config = GitHubConfig {
            token: String::new(),
            api_url: None,
        };

        let result = GitHubClient::from_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_github_error_display() {
        let error = GitHubError::NotConfigured("test error".to_string());
        assert!(error.to_string().contains("test error"));

        let api_error = GitHubError::ApiError {
            status: 404,
            message: "Not found".to_string(),
        };
        assert!(api_error.to_string().contains("404"));
    }
}
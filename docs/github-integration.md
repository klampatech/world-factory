# GitHub Integration Skill

This skill provides agents with the ability to interact with GitHub's API for PR review workflows.

## Capabilities

Agents can:
- **Read**: List open PRs, get PR details and diff, get PR comments, list commits
- **Write**: Leave PR comments, approve PRs, request changes on PRs

## Authentication

Agents must have `GITHUB_TOKEN` environment variable set with a Personal Access Token (PAT) that has:
- `repo` scope (for full read/write access)
- `pull_request` scope (if using fine-grained access)

### Generating a GitHub Token

1. Go to GitHub Settings → Developer settings → Personal access tokens
2. Generate new token (classic) with `repo` scope
3. Configure the token as `GITHUB_TOKEN` environment variable for the agent

## Usage

### Rust Code

```rust
use world_factory::services::github::{GitHubClient, GitHubConfig, PullRequest, PullRequestComment};

fn review_pr_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create client from environment
    let client = GitHubClient::from_env()?;
    
    // List open PRs
    let prs = client.list_prs("owner", "repo", Some("open")).await?;
    
    // Get PR details
    let pr = client.get_pr("owner", "repo", 123).await?;
    
    // Get PR diff
    let diff = client.get_pr_diff("owner", "repo", 123).await?;
    
    // Get PR comments
    let comments = client.get_pr_comments("owner", "repo", 123).await?;
    
    // Leave a comment on a specific line
    client.create_pr_comment("owner", "repo", 123, "Looks good!", None, Some("src/main.rs"), Some(42)).await?;
    
    // Add a general comment (not tied to specific code)
    client.add_general_comment("owner", "repo", 123, "### Review Summary\n\nReviewed all changes.").await?;
    
    // Approve a PR
    client.approve_pr("owner", "repo", 123, Some("LGTM!")).await?;
    
    // Request changes
    client.request_changes("owner", "repo", 123, "Please fix the test on line 42.").await?;
    
    Ok(())
}
```

### Configuration

```rust
use world_factory::services::github::{GitHubClient, GitHubConfig};

// Option 1: From environment variables
let client = GitHubClient::from_env()?;

// Option 2: Explicit configuration
let config = GitHubConfig::new("your-token-here", None);
let client = GitHubClient::from_config(&config)?;

// Option 3: Check if configured
let config = GitHubConfig::from_env();
if config.is_configured() {
    let client = GitHubClient::from_config(&config)?;
} else {
    // Handle missing credentials
}
```

## API Reference

### GitHubClient Methods

| Method | Description |
|--------|-------------|
| `list_prs(owner, repo, state)` | List PRs (state: "open", "closed", "all") |
| `get_pr(owner, repo, pr_number)` | Get PR details |
| `get_pr_diff(owner, repo, pr_number)` | Get PR diff as string |
| `get_pr_comments(owner, repo, pr_number)` | Get PR review comments |
| `list_pr_commits(owner, repo, pr_number)` | List commits in PR |
| `create_pr_comment(...)` | Create a line-specific comment |
| `add_general_comment(...)` | Add a general PR comment |
| `approve_pr(...)` | Approve the PR |
| `request_changes(...)` | Request changes on PR |

## Error Handling

The client returns `GitHubError` which can be:
- `NotConfigured` - GITHUB_TOKEN not set
- `RequestFailed` - Network/hTTP error
- `ApiError { status, message }` - GitHub API returned error
- `ParseFailed` - Failed to parse JSON response

## Best Practices

1. **Check configuration first**: Use `GitHubConfig::from_env().is_configured()` before attempting operations
2. **Handle errors gracefully**: Always match on `GitHubError` variants
3. **Use appropriate comments**: Use `create_pr_comment` for code-specific feedback, `add_general_comment` for overall feedback
4. **Review before approving**: Always review the diff and comments before approving

## Security Notes

- Never commit tokens to version control
- Use environment variables for token configuration
- PATs should be scoped to minimum required permissions
- Consider using GitHub Apps for production environments with better permission control
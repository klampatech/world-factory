# WOR-271: Create .github/workflows/deploy.yml — deploy artifact on main merge

## Status: COMPLETE

## Summary
Created `.github/workflows/deploy.yml` for automated deployment on main branch merges.

## Implementation
The workflow includes:
- **Build job**: Builds and pushes Docker image to GHCR (GitHub Container Registry)
  - Uses Docker Buildx with GitHub Actions cache
  - Tags: `main`, `sha-<hash>`, and `latest` (on default branch)
  - Pushes to `ghcr.io/<owner>/<repo>` registry
  
- **Deploy job**: Placeholder for actual server deployment
  - Downloads docker-compose
  - Configures docker-compose.yml for GHCR image
  - Includes placeholder for SSH/server deployment (needs server-specific implementation)

## Files Created
- `.github/workflows/deploy.yml` (3322 bytes)

## Next Steps
1. Server team needs to implement actual deployment step (SSH credentials, target host, etc.)
2. Configure required secrets for server access

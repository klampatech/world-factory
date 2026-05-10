# WOR-1119 Fix GitHub Release Workflows

## Issue
GitHub release workflows were referencing wrong binary name `prehistory-generator` instead of the actual binary `world_generator`.

## Root Cause
The `.github/workflows/deploy.yml` file contained references to:
- `target/release/prehistory-generator` (file path)
- `pkill prehistory-generator` (process name)
- `mv prehistory-generator prehistory-generator-...` (file renaming)

But the actual binary built from Cargo.toml is named `world_generator`.

## Fix Applied

### deploy.yml
Changed `prehistory-generator` → `world_generator` in both `deploy-staging` and `deploy-production` jobs:

**Before:**
```yaml
cp target/release/prehistory-generator /tmp/deploy/
scp ... /tmp/deploy/prehistory-generator ...
pkill prehistory-generator || true
mv prehistory-generator prehistory-generator-$VERSION
ln -sf prehistory-generator-$VERSION prehistory-generator
```

**After:**
```yaml
cp target/release/world_generator /tmp/deploy/
scp ... /tmp/deploy/world_generator ...
pkill world_generator || true
mv world_generator world_generator-$VERSION
ln -sf world_generator-$VERSION world_generator
```

## Verification
```bash
grep -r "prehistory-generator" .github/workflows/
# Returns: No matches found
```

## Files Changed
- `.github/workflows/deploy.yml` - Fixed binary name in both staging and production deploy jobs
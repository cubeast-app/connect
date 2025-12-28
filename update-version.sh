#!/bin/bash
# Update version across all project files

VERSION=$1

if [ -z "$VERSION" ]; then
  echo "Usage: ./update-version.sh <version>"
  echo "Example: ./update-version.sh 0.3.0"
  exit 1
fi

# Validate version format (basic semver check)
if ! [[ $VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Error: Version must be in semver format (e.g., 0.3.0)"
  exit 1
fi

echo "Updating version to $VERSION..."

# Update app/package.json
sed -i "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" package.json
echo "✓ Updated package.json"

# Update app/src-tauri/Cargo.toml
sed -i "s/^version = \"[^\"]*\"/version = \"$VERSION\"/" src-tauri/Cargo.toml
echo "✓ Updated src-tauri/Cargo.toml"

# Update app/src-tauri/tauri.conf.json
sed -i "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/g" src-tauri/tauri.conf.json
echo "✓ Updated src-tauri/tauri.conf.json"

# Run cargo check to update Cargo.lock
echo ""
echo "Running cargo check to update Cargo.lock..."
cd src-tauri
if cargo check; then
  echo "✓ Cargo.lock updated"
else
  echo "❌ cargo check failed"
  exit 1
fi
cd ../..

echo ""
echo "✅ Version updated to $VERSION in all files"

#!/bin/sh

# Script version
VERSION="v1.2.0"


# Temporary directory for downloads (use user's home directory for safety)
DOWNLOAD_DIR=$(mktemp -d --tmpdir=$HOME)

# CLI name (replace with your actual CLI name)
CLI_NAME="convy"

# Download URL for the pre-built CLI binary (replace with your appropriate URL)
CLI_DIR="https://github.com/josh-tracey/convy/releases/download/$VERSION"
CLI_URL="$CLI_DIR/convy-$VERSION-unknown-linux-gnu"


# Check for macOS-specific download (if applicable)
if [ $(uname) = "Darwin" ]; then
  CLI_URL="$CLI_DIR/convy-$VERSION-apple-darwin-arm64"
fi

# Download the CLI binary
echo "Downloading $CLI_NAME..."
curl -fsSL "$CLI_URL" -o "$DOWNLOAD_DIR/$CLI_NAME"

# Check for download success
if [ $? -ne 0 ]; then
  echo "Error downloading $CLI_NAME. Please check the download URL."
  exit 1
fi

# Set executable permissions (adjust if needed)
chmod +x "$DOWNLOAD_DIR/$CLI_NAME"

# Installation directory (modify as desired, use user's bin directory)
INSTALL_DIR="$HOME/.local/share/convy/bin"

# Check if the user's bin directory exists
if [ ! -d "$INSTALL_DIR" ]; then
  echo "Creating user bin directory: $INSTALL_DIR"
  mkdir -p "$INSTALL_DIR"
fi

# Check write permissions for the user's bin directory
if [ ! -w "$INSTALL_DIR" ]; then
  echo "The installation directory '$INSTALL_DIR' requires write permissions. Please adjust file permissions manually."
  exit 1
fi

# Move the binary to the user's bin directory
echo "Installing $CLI_NAME..."
mv "$DOWNLOAD_DIR/$CLI_NAME" "$INSTALL_DIR/$CLI_NAME"

# Cleanup temporary directory
rm -rf "$DOWNLOAD_DIR"

echo "Installation complete! Add '$INSTALL_DIR' to your PATH environment variable to use convy from any directory."

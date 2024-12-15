
#!/usr/bin/env bash

# Exit immediately if a command exits with a non-zero status
set -e

# Function to display error messages
error() {
  echo "Error: $1" >&2
  exit 1
}

# Pull the latest changes from the git repository
echo "Pulling latest changes from Git repository..."
git pull || error "Git pull failed"

# Get the current directory name for the image tag
CURRENT_DIR_NAME=$(basename "$PWD")

# Build the Docker image
echo "Building Docker image with tag: $CURRENT_DIR_NAME"
docker build -t "$CURRENT_DIR_NAME" . || error "Docker build failed"

# Bring down any running containers
echo "Stopping running containers..."
docker compose down || error "Failed to bring down Docker containers"

# Start the containers in detached mode
echo "Starting containers in detached mode..."
docker compose up -d || error "Failed to start Docker containers"

echo "Deployment completed successfully."

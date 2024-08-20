#!/bin/zsh

# Name of the binary
BINARY_NAME="nexus_cli"

# Path to the CLI crate directory (relative to the script's location)
CLI_DIR="nexus_cli"

# Check if Cargo is installed
if ! command -v cargo &> /dev/null
then
    echo "Cargo is not installed. Please install Rust and Cargo first."
    exit 1
fi

# Navigate to the root of the workspace
ROOT_DIR="$(dirname "$(realpath "$0")")"

# Build the project in release mode
echo "Building the CLI..."
cd "$ROOT_DIR/$CLI_DIR" || { echo "Failed to navigate to $CLI_DIR"; exit 1; }
cargo build --release

# Check if the build was successful
if [ $? -ne 0 ]; then
    echo "Failed to build the project."
    exit 1
fi

# Path to the built binary
BINARY_PATH="$ROOT_DIR/target/release/$BINARY_NAME"

# Check if the binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo "Binary not found: $BINARY_PATH"
    exit 1
fi

# Directory to install the binary to (usually ~/.cargo/bin or /usr/local/bin)
INSTALL_DIR="$HOME/.cargo/bin"

# Create the directory if it doesn't exist
mkdir -p "$INSTALL_DIR"

# Move the binary to the install directory
echo "Installing the CLI..."
cp "$BINARY_PATH" "$INSTALL_DIR/"

# Ensure the binary is executable
chmod +x "$INSTALL_DIR/$BINARY_NAME"

# Check if the install directory is in the PATH
if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
    echo "Warning: $INSTALL_DIR is not in your PATH."
    echo "You can add it to your PATH by adding the following line to your shell profile:"
    echo "export PATH=\"$INSTALL_DIR:\$PATH\""
fi

# Detect shell and update the appropriate profile
SHELL_PROFILE=""
if [ -n "$ZSH_VERSION" ]; then
    SHELL_PROFILE="$HOME/.zshrc"
elif [ -n "$BASH_VERSION" ]; then
    SHELL_PROFILE="$HOME/.bashrc"
else
    echo "Warning: Unsupported shell. Please manually add the PROJECT_ROOT to your profile."
    exit 1
fi

# Export the project root path to the detected profile
echo "export PROJECT_ROOT=\"$ROOT_DIR\"" >> "$SHELL_PROFILE"
echo "Project root path set to $ROOT_DIR in $SHELL_PROFILE. You may need to restart your terminal or source your profile."

echo "Installation complete. You can now use the CLI with the command: $BINARY_NAME"

# Example usage
echo "Try running: $BINARY_NAME nexus"

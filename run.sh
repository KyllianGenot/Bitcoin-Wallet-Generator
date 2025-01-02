#!/bin/bash

# Clear the terminal
clear

# Clean the Rust project
echo "Cleaning the project..."
cargo clean

# Build the Rust project
echo "Building the project..."
cargo build

# Clear the terminal again
clear

# Run the Rust project
echo "Running the project..."
cargo run
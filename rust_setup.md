 Rust Dev Environment Setup

  1. Install Rust via rustup (the official installer):

  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

  This installs:
  - rustup — toolchain manager
  - rustc — the Rust compiler
  - cargo — build system and package manager

  It will prompt you for installation options — the defaults (option 1) are fine.

  2. Activate it in your current shell (or restart your terminal):

  source "$HOME/.cargo/env"

  3. Verify:

  rustc --version
  cargo --version

  That's it — no other dependencies are needed for a basic Rust terminal project. Cargo handles everything (building,
  dependencies, testing).


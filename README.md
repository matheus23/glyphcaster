# Rusty Essay Editor

A live collaborative markdown editor built with Rust, GTK4, and Automerge

## What is this?

This is a side project exploring collaborative text editing using:
- **Rust** for the core application
- **GTK4** with `sourceview5` for the markdown editor UI
- **Automerge CRDT** for conflict-free collaborative editing
- **Samod** library for handling document loading and synchronization

The editor connects to `sync.automerge.org` for real-time collaboration, allowing multiple users to edit the same markdown document simultaneously without conflicts.

## Features

- ✅ Markdown syntax highlighting
- ✅ Document loading from Automerge sync server
- ✅ Clean GTK4 interface with loading states
- ✅ Line numbers and current line highlighting
- 🚧 Real-time collaborative editing (in progress)
- 🚧 Document synchronization (in progress)

## Dependencies

- **automerge**: CRDT implementation for collaborative editing
- **samod**: Document loading and synchronization library
- **gtk4**: GUI framework
- **sourceview5**: Text editor widget with syntax highlighting
- **async-tungstenite**: WebSocket client for sync server connection

## Getting Started

```bash
# Clone the repository
git clone <your-repo-url>
cd rusty-essay-editor

# Run the application
cargo run
```

The app will:
1. Show a loading screen while connecting to the sync server
2. Load a test document (hardcoded ID for now)
3. Display the markdown content in the editor

## Project Structure

- `src/main.rs` - Application entry point and GTK setup
- `src/app_state.rs` - UI state management and widget creation
- `src/document_loader.rs` - Document loading and Samod integration
- `src/runtime.rs` - GLib runtime adapter for Samod

## Current Status

This is the initial setup phase. The basic architecture is in place:
- ✅ GTK4 application with loading/editor states
- ✅ Connection to Automerge sync server
- ✅ Document loading from a hardcoded document ID
- ✅ Markdown editor with syntax highlighting

Next steps will involve implementing bidirectional synchronization between the text buffer and the Automerge document.

## Configuration

The app currently connects to `wss://sync3.automerge.org` and loads document ID `p8dpAaexjrpx2JFKbg5Z3a4NQyN`. These will be made configurable in future iterations.

## License

This is a personal side project - use it however you want! 🚀

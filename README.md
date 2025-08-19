# Grammancy

A live collaborative markdown editor built with Rust, GTK4, and Automerge

## Usage

To run a collaborative editor for a particular automerge document shared on `sync.automerge.org`:

```
cargo run -- <document ID>
```

## What is this?

This is a side project exploring collaborative text editing using:

- **Rust** for the core application
- **GTK4** with `sourceview5` for the markdown editor UI
- **Automerge CRDT** for conflict-free collaborative editing
- **Samod** library for handling document loading and synchronization

The editor connects to `sync.automerge.org` for real-time collaboration, allowing multiple users to edit the same markdown document simultaneously without conflicts.


The editor expects the document to have the following structure:

```typescript
type Document = {
  content: string // A markdown string
}
```

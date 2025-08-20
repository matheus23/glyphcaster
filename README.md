# Glyphcaster + iroh

A live collaborative markdown editor built with Rust, GTK4, and Automerge, syncing with samod and iroh.

## Usage

To run a collaborative editor for a particular automerge document:

```
cargo run -- <optional document ID> <optional iroh node ID>
```

Then click on the copy button to the right next to "Connect using <...>". And on a different computer or somewhere else:

```
cargo run -- <paste>
```

For example:

```
cargo run -- automerge:e7B9YqGvpm1JuRu8LVGYVPLrWy2 57f8e8fff6a49e855f24894680b2954cc14a528a442dc6def67f6e3458566dc0
```

## What is this?

This is a side project exploring collaborative text editing using:

- **Rust** for the core application
- **GTK4** with `sourceview5` for the markdown editor UI
- [**Automerge CRDT**] for conflict-free collaborative editing
- [**Samod**] library for handling document loading and synchronization
- [**iroh**] for establishing peer-to-peer connections (with relay fallbacks) between instances

The editor connects using samod and iroh for real-time collaboration, allowing multiple users to edit the same markdown document simultaneously without conflicts.


The editor expects the document to have the following structure:

```typescript
type Document = {
  content: string // A markdown string
}
```

[**Automerge CRDT**]: https://automerge.org
[**Samod**]: https://github.com/alexjg/samod/
[**iroh**]: https://iroh.computer

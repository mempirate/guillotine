
- `Ui`
- Constructing a frame (inside of `render`), now uses `cx.row()`, and `cx.text()`. These methods
  now return Builders that borrow the arena, and adding children stores them in the arena and
  inserts a pointer to the node.

- [x] Use heapless::VecView in the per-render context to erase N. This avoids forcing the capacity const generic through every Render implementation.

- [x] Make Ui generic over the capacity while keeping Render independent of it.

- [ ] NodeIndex -> u16 or u32

**FrameTree Migration**

- Currently, FrameTree resolution does 2 things:
  1. Converts recursive Element tree into a flat `FrameTree`
  2. Lays out the resulting nodes.

With the new `FrameStorage` (and `StorageView` provided to the component builders through `Context`),
task 1 will already be handled through that.

Task 2 can still be done by `FrameTree`. For separation of concerns, I'd like to keep `FrameTree`, but
instead of owning the flat node tree, it mutably borrows it from `StorageView`. It then does the same
thing as before.

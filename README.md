# Iyes Mesh Array (IMA)

This is an experimental new Rust-centric file format for GPU Mesh data.

Features:
 - Stores GPU mesh data with any set of vertex attributes and optional indices.
   - Special-cases common usages: Position, Normal, Tangent, Color, UV0, UV1, Joint Index/Weight.
   - Supports custom attributes (identified by user-specified integer id).
 - Can store an array of multiple compatible meshes
   - Compatible means: same set of vertex/index buffers and formats.
   - Vertex/index data concatenated together in large buffers.
   - Designed for "multi draw indirect" use cases.
   - You can just load all data into GPU memory.
   - You can trivially get an indirect draw buffer from the file metadata.
 - Supports embedding arbitrary user data.
   - Useful if you want to store your own custom material data or anything else.
 - Very small file size (much smaller than GLTF and other formats).
   - Data is aggressively compressed using zstd.
   - File metadata compactly encoded using `bitcode`.
 - Quick to decode and load into memory.
 - Optional checksums for data and metadata (RapidHash).

Deliberately does not support:
 - Material data. Does not store materials.
 - Texture data (see `iyes_texture`).
 - Scene data (transforms, etc.).
 - Other object types (cameras, lights, etc.).

May support in the future:
 - MeshOpt encoding of buffers pre-zstd-compression

This is not a scene format. It's a GPU mesh format.

However, if you need to use it for such use cases, feel free to encode whatever
you need into an application-specific format (we recommend `bitcode`-encoded
Rust structs) and include it as "user data".

## Tooling

There is CLI tool in `bin/iyesmesh`, which lets you work with IMA files.

Run it as:

```sh
cargo run -p iyesmesh -- help
```

(this will print help, to show you what you can do)

Install it as:

```sh
cargo install --force --path bin/iyesmesh
```

It supports various operations on IMA files:
 - Debug info and verification/checking
 - Merging multiple files
 - Deleting specific contents from files
 - Extracting and replacing user data
 - Converting from Wavefront OBJ files

Planned future work:
 - Converting from more formats: GLTF, STL, maybe FBX.
 - Extracting meshes from IMA into other formats.
 - Running MeshOpt passes to optimize mesh data

## Reference Implementation (Library)

This repo contains the reference encoder and decoder written in Rust.

They are designed to avoid copying data around. Extensive use of Rust
references and lifetimes.

They work with bare byte slices (`&[u8]`) and do not assume any game engine
or graphics programming framework.

Optional integration with `wgpu` and Bevy is planned as future work.

## Documentation

There is a brief specification of the file format in `doc/ima.md`.

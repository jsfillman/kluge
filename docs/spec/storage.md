# Storage

The researcher has one tree, the team root. `dir` is the working tree. `into` is the declared outputs. The laptop is not a copy of that tree, and the pod does not mount the laptop.

NFS is not a backend. Neither is sshfs. Both turn every stat and every lock into a WAN round trip, and a training run then looks hung. Kluge has two backends behind the same paths. The admin picks them. The default is both at once.

## The cache

The cache is for the working tree: source, configs, and anything under `dir` that is not a declared output and is not over the spill size.

It is a content-addressed chunk service over HTTP.

- `PUT /chunks/{digest}` writes bytes. A `Content-Range` continues a chunk that did not finish. The digest is the whole chunk, so a retry of a finished chunk is the same request.
- `GET /chunks/{digest}` returns one chunk.
- `PUT /files/{path}` records the digest, size, and mode. The path is relative to the team root.
- `GET /files?prefix=` lists metadata. It does not return file bodies.

There is no POSIX lock on the wire. A file is immutable once its digest is recorded. Replacing it writes a new digest. The laptop keeps a read cache of chunks it has already fetched.

`kluge files DIR` materializes that working tree into a local directory and writes changes back as chunks. That directory is the drive. A FUSE or macOS file-provider mount is the same protocol with a kernel presentation. The mount is not required for the protocol, and it is not tested without a machine that can mount one.

## The object store

The object store is S3-shaped. It is for declared outputs and for any other file over `volume.spill`. A multi-gigabyte checkpoint stays in the bucket. The laptop does not mirror the bucket after a run.

- `PUT /objects/{key}` writes one object. The key is the path relative to the team root.
- `GET /objects/{key}` returns that object and nothing else.
- `HEAD /objects/{key}` returns size and digest.
- `GET /objects?prefix=` lists keys, sizes, and digests. It does not return bodies.

A run record stores those keys. `kluge outputs NAME` prints them. `kluge fetch NAME PATH` downloads one object into a local file. There is no command that downloads every object under a run.

## Hybrid

This is the install a lab actually wants.

| Path | Backend | Laptop |
| --- | --- | --- |
| Under `dir`, not under `into`, at or under `volume.spill` | Cache | Materialized by `kluge files`, or by the mount |
| Under `into` | Object | Listed. Bytes move only on `kluge fetch` or on opening that one file |
| Over `volume.spill`, wherever it sits under the team root | Object | Same as `into` |
| On the team root but outside `dir` and `into` | Cluster filesystem only | Not in the manifest. A checkpoint that was not declared is not copied home |

`classify(spec, files, spill)` returns that table. `files` is a list of paths and sizes. The digest is computed only for a file whose bytes are handed to the publisher. An omitted path is not read.

The pod still mounts the team root, read-write, and nowhere else. The cache service and the bucket are how that root is backed for the laptop and, when the admin points the root at them, for the pod. Kluge does not add a second mount and does not copy the root onto the laptop.

## Admin settings

| Setting | Effect |
| --- | --- |
| `volume.cache` | URL of the chunk service. Empty means the working tree is not offered to the laptop |
| `volume.bucket` | Bucket and prefix for objects. Empty means declared outputs stay on the cluster filesystem |
| `volume.spill` | Size above which a file is an object even when it is not under `into`. Default `1Gi` |
| `team.root` | Unchanged. Both backends are paths under it |

## Commands

`kluge files`, `kluge outputs`, and `kluge fetch` are exact words, like `menu`. They are not in the prefix tree.

- `kluge files DIR` fills `DIR` from the cache and writes changes back as chunks.
- `kluge outputs NAME` prints the object list for that run.
- `kluge fetch NAME PATH` downloads that one object.

None of them submit, cancel, or sync a run. Sync remains the CLI action that publishes the manifest. The menu bar only lists the object names it was given.

# Coordination archive

Each issue has an isolated directory, `issue-<N>/manifest.json`, listing immutable `comments-<first-id>-<last-id>.jsonl` segments. Each segment contains complete issue-comment JSON objects, one per line, in source order. The manifest records `issue: N`, each segment's count, ID range, and SHA-256. Segment files are never rewritten; a later maintenance pass may add segments and update that issue's manifest. The dedicated `coordination-archive` branch is the durable archive and read-back source.

## Reconstructing history

Read every segment listed in the selected `issue-<N>/manifest.json` in ascending comment-ID order, then combine it with that issue's live comments. The retained live tail overlaps the archive by design. Deduplicate that overlap by stable GitHub comment ID and use the issue's authoritative ordering; do not infer missing history from the manifest alone. The live comments plus listed archive segments form the available history. The original flat #52 archive was moved to `issue-52/`; segment bytes and SHA-256 values are unchanged.

## Capacity and maintenance

The scheduled workflow runs every six hours. Slot rotation occurs at `COORDINATION_SWITCH_AT` (default 2,000, maximum 2,400), below GitHub's 2,500-comment write cutoff. A draining issue is archived and compacted; an active issue is eligible for compaction only above 1,500 comments. Segments contain the exact JSONL bytes used to calculate their hashes. The workflow appends archive files by fast-forward commits to `coordination-archive`; it never force-pushes or writes to the default branch.

Compaction is a dry run by default. Deletion requires the repository variable `ARCHIVE_DELETE_ENABLED` to be `true` and an explicit `--confirm-delete`. Immediately before each individual deletion, the tool reads the manifest and every listed segment from `--archive-ref` and requires byte-identical read-back and matching hashes. It estimates calls against a 3,000 request per-run budget, leaving headroom under GitHub's shared 5,000/hour REST limit; remaining candidates are handled by later runs. If any read, schema, pagination, or hash check fails, deletion stops. Comments are never blanked or rewritten.

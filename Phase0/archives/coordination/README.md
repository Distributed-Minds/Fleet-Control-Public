# Coordination archive

`manifest.json` lists immutable `comments-<first-id>-<last-id>.jsonl` segments. Each segment contains complete issue-comment JSON objects, one per line, in source order. The manifest records each segment's count, ID range, and SHA-256. Segment files are never rewritten; a later maintenance pass may add segments and update the manifest.

## Reconstructing history

Read every segment listed in `manifest.json` in ascending comment-ID order, then combine it with the live coordination issue's comments. The retained live tail overlaps the archive by design. Deduplicate that overlap by stable GitHub comment ID and use the issue's authoritative ordering; do not infer missing history from the manifest alone. The live comments plus listed archive segments form the available history.

## Capacity and maintenance

The scheduled workflow checks the coordination issue every six hours and prepares an archive when its live count reaches 1,500 comments, leaving a minimum live tail of 750. Segments contain the exact JSONL bytes used to calculate their hashes. New artifacts are proposed on an archive branch through a pull request; the workflow never writes to the default branch.

Compaction is a dry run by default. Deletion requires the repository variable `ARCHIVE_DELETE_ENABLED` to be `true` and an explicit `--confirm-delete`. Immediately before each individual deletion, the tool reads the manifest and every listed segment from `--archive-ref` and requires byte-identical read-back and matching hashes. It estimates calls against a 3,000 request per-run budget, leaving headroom under GitHub's shared 5,000/hour REST limit; remaining candidates are handled by later runs. If any read, schema, pagination, or hash check fails, deletion stops. Comments are never blanked or rewritten.

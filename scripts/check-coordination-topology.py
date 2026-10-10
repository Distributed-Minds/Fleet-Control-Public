#!/usr/bin/env python3
"""Fail-closed source-shape diagnostic for #338's installed Actions workflow.

This is a limited structural recognizer, NOT a YAML or GitHub Actions interpreter.
It is intentionally tied to the currently observed vulnerable baseline; changes
must be reviewed/recharacterized before the old queue/predicate model is reused.
Python standard library only; no GitHub requests or repository side effects.

Usage: python3 scripts/check-coordination-topology.py WORKFLOW_YAML
       python3 scripts/check-coordination-topology.py --selftest
"""

import re
import sys
from pathlib import Path


class ShapeError(ValueError):
    pass


def block(lines, name, depth=0):
    pad = ' ' * depth
    header = pad + name + ':'
    hits = [i for i, line in enumerate(lines) if line.rstrip() == header]
    if len(hits) != 1:
        raise ShapeError(f'expected exactly one {header!r}; found {len(hits)}')
    start = hits[0] + 1
    end = start
    for end in range(start, len(lines)):
        line = lines[end]
        if line.strip() and not line.lstrip().startswith('#') and (
            len(line) - len(line.lstrip(' ')) <= depth
        ):
            return lines[start:end]
    return lines[start:]


def keys(lines, depth):
    seen = {}
    pattern = re.compile(r'^' + ' ' * depth + r'([\w-]+):(?:\s*(.*))?$')
    for line in lines:
        if not line.strip() or line.lstrip().startswith('#'):
            continue
        if len(line) - len(line.lstrip(' ')) != depth:
            continue
        match = pattern.fullmatch(line)
        if match is None:
            raise ShapeError(f'unrecognized structural key: {line!r}')
        key, value = match.groups()
        if key in seen:
            raise ShapeError(f'duplicate structural key: {key}')
        seen[key] = (value or '').strip()
    return seen


def same(actual, expected, label):
    if actual != expected:
        raise ShapeError(f'{label}: expected {expected!r}, found {actual!r}')


def verify_shape(source):
    lines = source.splitlines()
    on = block(lines, 'on')
    same(keys(on, 2), {'schedule': '', 'workflow_dispatch': '', 'issue_comment': ''}, 'triggers')
    same(keys(block(on, 'issue_comment', 2), 4), {'types': '[created]'}, 'comment events')
    cron_lines = [line.strip() for line in block(on, 'schedule', 2) if line.strip() and not line.lstrip().startswith('#')]
    same(cron_lines, ["- cron: '0 */6 * * *'"], 'recovery schedule')

    concurrency = keys(block(lines, 'concurrency'), 2)
    same(concurrency, {'group': 'coordination-slots-maintenance', 'cancel-in-progress': 'false'}, 'workflow concurrency')

    jobs = block(lines, 'jobs')
    same(keys(jobs, 2), {'status': '', 'archive': '', 'compact': '', 'rotate': ''}, 'job inventory')
    for job, expected_needs in [('status', None), ('archive', None), ('compact', 'archive'), ('rotate', '[archive, compact]')]:
        fields = keys(block(jobs, job, 2), 4)
        same(fields.get('needs'), expected_needs, f'{job}.needs')
        if not fields.get('if'):
            raise ShapeError(f'{job}.if is missing or multiline')
        if job == 'status' and fields['if'] != "github.event_name == 'workflow_dispatch'":
            raise ShapeError('status.if is no longer dispatch-only')
        if job == 'rotate' and "needs.archive.result == 'success'" not in fields['if']:
            raise ShapeError('rotate.if archive-success term changed')
    return True


FIXTURE = '''name: Coordination slots maintenance
on:
  schedule:
    - cron: '0 */6 * * *'
  workflow_dispatch:
  issue_comment:
    types: [created]
permissions:
  contents: read
concurrency:
  group: coordination-slots-maintenance
  cancel-in-progress: false
jobs:
  status:
    if: github.event_name == 'workflow_dispatch'
    runs-on: ubuntu-24.04
  archive:
    if: github.event_name != 'workflow_dispatch'
    runs-on: ubuntu-24.04
  compact:
    needs: archive
    if: github.event_name != 'workflow_dispatch'
    runs-on: ubuntu-24.04
  rotate:
    needs: [archive, compact]
    if: ${{ !cancelled() && needs.archive.result == 'success' }}
    runs-on: ubuntu-24.04
'''


def run_selftest():
    assert verify_shape(FIXTURE)
    mutants = [
        ("  issue_comment:\n    types: [created]\n", ''),
        ('  workflow_dispatch:\n', ''),
        ("    types: [created]", "    types: [edited]"),
        ("    - cron: '0 */6 * * *'", "    - cron: '0 * * * *'"),
        ("    - cron: '0 */6 * * *'", "    - cron: '0 */6 * * *'\n    - cron: '0 * * * *'"),
        ('  group: coordination-slots-maintenance', '  group: status-only'),
        ('  cancel-in-progress: false', '  cancel-in-progress: true'),
        ('    needs: archive', '    needs: status'),
        ('    needs: [archive, compact]', '    needs: archive'),
        ('    needs: [archive, compact]', '    needs: [compact, archive]'),
        ("    if: github.event_name == 'workflow_dispatch'", "    if: github.event_name != 'workflow_dispatch'"),
        ("  rotate:\n", "  rotate-renamed:\n"),
        ("  issue_comment:\n", "  issue_comment:\n  issue_comment:\n"),
        ("needs.archive.result == 'success'", "needs.archive.result == 'failure'"),
    ]
    for old, new in mutants:
        if FIXTURE.count(old) != 1:
            raise AssertionError(f'nonunique mutation anchor: {old!r}')
        try:
            verify_shape(FIXTURE.replace(old, new, 1))
        except ShapeError:
            continue
        raise AssertionError(f'undetected topology mutation: {old!r} -> {new!r}')
    assert verify_shape(FIXTURE + '\n# harmless trailing note\n')
    print(f'TOPOLOGY SELFTEST: {len(mutants)} negative mutants rejected; 2 positives accepted')
    print('LIMIT: synthetic shape only; not a YAML parser, hosted Actions or CI')


def main(argv):
    if len(argv) == 2 and argv[1] == '--selftest':
        run_selftest()
        return 0
    if len(argv) != 2:
        print(__doc__.strip(), file=sys.stderr)
        return 2
    verify_shape(Path(argv[1]).read_text(encoding='utf-8'))
    print('KNOWN INSTALLED WORKFLOW SHAPE: matched; defects #338 remain RED')
    return 0


if __name__ == '__main__':
    try:
        sys.exit(main(sys.argv))
    except (OSError, ShapeError, AssertionError) as exc:
        sys.exit(f'TOPOLOGY UNKNOWN/DRIFT: {exc}')

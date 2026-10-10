#!/usr/bin/env python3
"""Diagnostic only: characterize #338 workflow-level concurrency starvation.

Models documented GitHub Actions queue=single semantics (one running and one
pending; a newer pending run replaces the older). This is NOT an Actions runner,
a YAML evaluator, a durable lock, or proof of scheduled delivery. No third-party
packages, GitHub calls, writes or time-dependent sleeps are used.

Usage: python3 scripts/check-coordination-queue-model.py [WORKFLOW_YAML]
The optional YAML argument rejects unrecognized baseline structure before
characterizing it; it does not validate a proposed workflow's syntax.
"""

import sys
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class Event:
    label: str
    kind: str
    group: str
    mutates: bool


def enqueue(events):
    """Return active/pending per group and displaced pending labels."""
    active, pending, displaced = {}, {}, []
    for event in events:
        if event.group not in active:
            active[event.group] = event
        else:
            if event.group in pending:
                displaced.append(pending[event.group].label)
            pending[event.group] = event
    return active, pending, displaced


def baseline(kinds):
    """Installed workflow: every event shares its workflow-level group."""
    return [Event(f'{i}:{kind}', kind, 'coordination-slots-maintenance',
                  kind in {'schedule', 'trusted_comment'})
            for i, kind in enumerate(kinds)]


def schedule_only(kinds):
    """Candidate: schedule is the sole mutator; status reads are independent."""
    return [Event(f'{i}:{kind}', kind,
                  'coordination-slots-maintenance' if kind == 'schedule'
                  else f'read-only-{i}', kind == 'schedule')
            for i, kind in enumerate(kinds) if kind != 'issue_comment']


def fits_headroom(rate_per_hour, missed_ticks, delay_hours=0):
    """Sufficient modeled *rate* bound, NOT guaranteed provider liveness."""
    return rate_per_hour * ((missed_ticks + 1) * 6 + delay_hours) < 500


def verify_baseline_text(text):
    # Deliberately narrow text signatures, not a YAML parse. Any drift requires
    # a human review of the model and explicit fixture update.
    required = (
        'cron: \'0 */6 * * *\'',
        '  workflow_dispatch:',
        '  issue_comment:',
        '    types: [created]',
        'group: coordination-slots-maintenance',
        'cancel-in-progress: false',
        'github.event.issue.comments >= 1000',
        "needs.archive.result == 'success'",
    )
    missing = [term for term in required if term not in text]
    if missing:
        raise ValueError('unknown workflow baseline; missing: ' + repr(missing))
    if "needs.compact.result == 'success'" in text:
        raise ValueError('workflow dependency gate has changed; rebaseline')


def run():
    checked = 0

    def check(label, observed, expected=True):
        nonlocal checked
        if observed != expected:
            raise AssertionError(f'{label}: observed={observed!r} expected={expected!r}')
        checked += 1

    cases = [
        (['schedule', 'schedule', 'issue_comment'], 'baseline unrelated comment'),
        (['schedule', 'schedule', 'workflow_dispatch'], 'baseline manual status'),
        (['schedule', 'schedule', 'trusted_comment'], 'baseline trusted comment'),
    ]
    for kinds, label in cases:
        _, _, displaced = enqueue(baseline(kinds))
        check(label + ' evicts pending scheduled rescue', '1:schedule' in displaced)

    flood = ['schedule', 'schedule'] + ['issue_comment'] * 100
    _, pending, displaced = enqueue(baseline(flood))
    check('baseline flood evicts rescue', '1:schedule' in displaced)
    check('baseline flood changes pending 100 times', len(displaced), 100)
    check('baseline flood replaces pending with a nonmutating run',
          pending['coordination-slots-maintenance'].mutates, False)

    for kinds, label in cases:
        active, pending, displaced = enqueue(schedule_only(kinds))
        check(label + ' cannot evict schedule-only rescue',
              pending['coordination-slots-maintenance'].label, '1:schedule')
        check(label + ' does not admit concurrent writers',
              sum(event.mutates for event in active.values()), 1)
        check(label + ' excludes rescue from displacement', '1:schedule' in displaced, False)

    active, pending, displaced = enqueue(schedule_only(flood))
    check('100 comments cannot evict schedule-only rescue',
          pending['coordination-slots-maintenance'].label, '1:schedule')
    check('no comment event enters proposed workflow', len(active), 1)
    check('no displaced rescue under comment flood', '1:schedule' in displaced, False)

    active, pending, displaced = enqueue(schedule_only(['schedule'] * 3))
    check('later schedule may still displace earlier pending schedule',
          '1:schedule' in displaced)
    check('latest pending scheduled instance is retained',
          pending['coordination-slots-maintenance'].label, '2:schedule')
    check('a single group still has exactly one scheduled writer',
          sum(event.mutates for event in active.values()), 1)

    unsafe_split = [Event('schedule', 'schedule', 'scheduled', True),
                    Event('comment', 'trusted_comment', 'comments', True)]
    active, _, _ = enqueue(unsafe_split)
    check('naive separate mutator groups allow simultaneous writers',
          sum(event.mutates for event in active.values()), 2)

    for rate, missed, delay, expected in (
        (32.5, 0, 0, True),
        (32.5, 1, 0, True),
        (32.5, 2, 0, False),
        (50, 1, 0, False),
        (80, 0, 1, False),
        (49.9, 0, 4, True),
    ):
        check(f'conditional headroom: {rate=} {missed=} {delay=}',
              fits_headroom(rate, missed, delay), expected)

    # Normal rotation is allowed only after BOTH dependencies succeed.
    for compact in ('failure', 'skipped', 'cancelled'):
        check('reject compact=' + compact,
              'success' == 'success' and compact == 'success', False)
    check('accept successful dependencies',
          'success' == 'success' and 'success' == 'success')
    print(f'QUEUE-LEVEL DIAGNOSTIC: {checked}/{checked} deterministic assertions PASS')
    print('KNOWN CURRENT DEFECT: unrelated events can replace pending scheduled rescue')
    print('CANDIDATE: schedule-only single writer; liveness conditional on arrival bounds')
    print('NOT PROVED: hosted workflow, YAML grammar, clock delivery, archive effect authorization')
    return checked


if __name__ == '__main__':
    if len(sys.argv) > 2:
        sys.exit('usage: check-coordination-queue-model.py [WORKFLOW_YAML]')
    try:
        if len(sys.argv) == 2:
            verify_baseline_text(Path(sys.argv[1]).read_text(encoding='utf-8'))
        run()
    except (OSError, ValueError, AssertionError) as exc:
        sys.exit(f'DIAGNOSTIC INVALID: {exc}')

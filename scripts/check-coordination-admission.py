#!/usr/bin/env python3
"""Offline diagnostic for #338: GitHub Actions job admission, not a GitHub runner.

Reads the actual coordination workflow and evaluates only its current, narrow
expression vocabulary. Unknown syntax raises an error rather than being guessed.
The observed failing safety cases are printed, not silently reclassified as green.
Requires Python standard library only; never calls GitHub or mutates a repository.

Usage: python3 scripts/check-coordination-admission.py .github/workflows/coordination-slots.yml
"""

import ast
import re
import sys
from pathlib import Path
from types import SimpleNamespace as NS

JOBS = ('archive', 'compact', 'rotate')


def job_conditions(yaml_text):
    """Read the three job if: scalars from the current known workflow shape."""
    conditions = {}
    job = None
    for line in yaml_text.splitlines():
        top = re.fullmatch(r'  ([A-Za-z][\w-]*):\s*', line)
        if top:
            job = top.group(1)
        elif job in JOBS:
            if line.startswith('    if: '):
                cond = line.split('if: ', 1)[1].strip()
                if cond.startswith('$' + '{{') and cond.endswith('}}'):
                    cond = cond[3:-2].strip()
                if job in conditions:
                    raise ValueError(f'duplicate if: in {job}')
                conditions[job] = cond
            elif line.startswith('    if:'):
                raise ValueError(f'unsupported if: formatting in {job}')
    if set(conditions) != set(JOBS):
        raise ValueError(f'expected {JOBS}, found {sorted(conditions)}')
    return conditions


def actions_expression(expr, *, trigger, issue_count, issue_is_pr, trusted,
                       archive_result='success', compact_result='success', cancelled=False):
    """Evaluate only the operators/calls used by the pinned workflow expressions.

    cancelled() is explicit; do not imply success() in an expression with
    an explicit status function, per GitHub Actions job-if semantics.
    """
    normalized = re.sub(r'(?<![=!])!(?!=)', ' not ', expr.replace('&&', ' and ').replace('||', ' or '))
    normalized = re.sub(r'\bnull\b', 'None', normalized).strip()
    tree = ast.parse(normalized, mode='eval')
    env = {
        'github': NS(event_name=trigger, event=NS(
            issue=NS(pull_request={} if issue_is_pr else None, comments=issue_count),
            comment=NS(user=NS(login='geromet' if trusted else 'stranger')))),
        'vars': NS(COORDINATION_TRUSTED_AUTHORS='geromet'),
        'needs': NS(archive=NS(result=archive_result), compact=NS(result=compact_result)),
    }

    def interpret(node):
        if isinstance(node, ast.Constant):
            return node.value
        if isinstance(node, ast.Name):
            if node.id in env:
                return env[node.id]
            raise ValueError(f'unsupported identifier: {node.id}')
        if isinstance(node, ast.Attribute):
            return getattr(interpret(node.value), node.attr)
        if isinstance(node, ast.BoolOp) and isinstance(node.op, (ast.And, ast.Or)):
            if isinstance(node.op, ast.And):
                result = True
                for value in node.values:
                    result = interpret(value)
                    if not result:
                        return result
                return result
            result = False
            for value in node.values:
                result = interpret(value)
                if result:
                    return result
            return result
        if isinstance(node, ast.UnaryOp) and isinstance(node.op, ast.Not):
            return not interpret(node.operand)
        if isinstance(node, ast.Compare) and len(node.ops) == 1:
            left, right = interpret(node.left), interpret(node.comparators[0])
            op = node.ops[0]
            if isinstance(op, ast.Eq):
                return left == right
            if isinstance(op, ast.NotEq):
                return left != right
            if isinstance(op, ast.GtE):
                return left >= right
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Name) and not node.keywords:
            args = [interpret(x) for x in node.args]
            if node.func.id == 'cancelled' and not args:
                return cancelled
            if node.func.id == 'format' and args and isinstance(args[0], str):
                return args[0].format(*args[1:])
            if node.func.id == 'contains' and len(args) == 2:
                return str(args[1]).lower() in str(args[0]).lower()
        raise ValueError(f'unsupported expression construct: {ast.dump(node)}')

    return bool(interpret(tree.body))


def admitted(conditions, job, case):
    if job == 'compact' and case['archive_result'] != 'success':
        return False  # implicit success() because compact has no status function
    return actions_expression(conditions[job], **case)


def scenario(trigger='schedule', issue_count=0, issue_is_pr=False, trusted=True,
             archive_result='success', compact_result='success', cancelled=False):
    return dict(trigger=trigger, issue_count=issue_count, issue_is_pr=issue_is_pr,
                trusted=trusted, archive_result=archive_result,
                compact_result=compact_result, cancelled=cancelled)


def run(workflow_path):
    conditions = job_conditions(Path(workflow_path).read_text(encoding='utf-8'))
    # Baseline: exact current main workflow. An unexpected changed result fails
    # so this diagnostic must be consciously updated, not assumed current.
    controls = [
        ('scheduled healthy rotation', scenario(), True),
        ('scheduled archive failed', scenario(archive_result='failure'), False),
        ('scheduled compact failed', scenario(compact_result='failure'), True),
        ('scheduled compact skipped', scenario(compact_result='skipped'), True),
        ('scheduled compact cancelled', scenario(compact_result='cancelled'), True),
        ('run globally cancelled', scenario(cancelled=True), False),
        ('trusted active-like count 2001', scenario(trigger='issue_comment', issue_count=2001), True),
        ('trusted count 999', scenario(trigger='issue_comment', issue_count=999), False),
        ('untrusted comment', scenario(trigger='issue_comment', issue_count=2001, trusted=False), False),
        ('PR comment', scenario(trigger='issue_comment', issue_count=2001, issue_is_pr=True), False),
        ('unrelated trusted issue count 1001', scenario(trigger='issue_comment', issue_count=1001), True),
    ]
    for label, c, expected in controls:
        result = admitted(conditions, 'rotate', c)
        if result != expected:
            raise AssertionError(f'{label}: observed={result} expected-baseline={expected}')

    # Explicit job dependency output supplies a distinct guard. Its absence
    # must be reported as a safety violation even if baseline controls pass.
    regressions = [
        (label, c) for label, c, _ in controls
        if (c['compact_result'] != 'success' or label == 'unrelated trusted issue count 1001')
        and admitted(conditions, 'rotate', c)
    ]
    for label, _ in regressions:
        print(f'KNOWN UNSAFE ADMISSION: {label}')
    print(f'BASELINE CHARACTERIZED: {len(controls)}/{len(controls)} expected predicate outcomes')
    print(f'UNSAFE ADMISSION CASES: {len(regressions)} (not a passing safety gate)')
    print('LIMITATION: static expression evaluator; not a hosted GitHub Actions or archive/rotate integration test')
    return len(regressions)


if __name__ == '__main__':
    if len(sys.argv) not in (2, 3) or (len(sys.argv) == 3 and sys.argv[2] != '--strict'):
        sys.exit('usage: check_coordination_admission.py PATH_TO_WORKFLOW [--strict]')
    try:
        n = run(sys.argv[1])
    except (OSError, ValueError, SyntaxError, AssertionError) as exc:
        sys.exit(f'DIAGNOSTIC INVALID: {exc}')
    # Do not make a deliberately RED diagnostic fail an unrelated CI suite.
    # Consumers looking for a policy admission gate can opt into a nonzero status.
    sys.exit(2 if n and '--strict' in sys.argv else 0)

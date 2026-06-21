#!/usr/bin/env python3
"""Candidate command that replays a REAL Docker test outcome.

`holdout grade` writes one pytest test id to stdin per invocation; this prints the
PASS/FAIL recorded by the swebench Docker run for that test id. This lets holdout
grade the real in-Docker results with no special-casing — the "candidate" is the
patched repo's actual behavior, captured once.

Usage: replay.py <results.json>   (results.json maps test_id -> "PASS"/"FAIL")
"""
import sys
import json

results = json.load(open(sys.argv[1]))
tid = sys.stdin.read().strip()
print(results.get(tid, "FAIL"))

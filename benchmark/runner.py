#!/usr/bin/env python3
"""Run one QuixBugs function on a single JSON-encoded argument list from stdin.

Usage: runner.py <programs_dir> <name>
Reads one JSON array (the positional args) from stdin, calls <name>(*args),
prints the JSON-encoded result to stdout. Errors are emitted as a JSON object
{"__error__": "..."} so a crash on the buggy version shows up as a divergence
from the (correct) reference rather than killing the harness.
"""
import sys
import json
import copy
import signal
import importlib.util

# Buggy QuixBugs versions can infinite-loop (e.g. bitcount, gcd). holdout's
# grade/verify do not yet bound candidate wall-clock, so we bound it here: a
# non-terminating run surfaces as a divergent {"__error__": "Timeout"} output.
TIMEOUT_SECONDS = 3


def _on_timeout(signum, frame):
    raise TimeoutError()


def load(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return getattr(mod, name)


def main():
    programs_dir, name = sys.argv[1], sys.argv[2]
    raw = sys.stdin.read()
    args = json.loads(raw)
    signal.signal(signal.SIGALRM, _on_timeout)
    signal.alarm(TIMEOUT_SECONDS)
    try:
        func = load(f"{programs_dir}/{name}.py", name)
        result = func(*copy.deepcopy(args))
        # Normalize generators/iterators to lists so output is comparable.
        if hasattr(result, "__iter__") and not isinstance(result, (list, str, dict)):
            result = list(result)
        print(json.dumps(result, sort_keys=True, default=str))
    except TimeoutError:
        print(json.dumps({"__error__": "Timeout"}))
    except RecursionError:
        print(json.dumps({"__error__": "RecursionError"}))
    except Exception as e:  # noqa: BLE001 - any defect manifests as a divergent output
        print(json.dumps({"__error__": type(e).__name__}))
    finally:
        signal.alarm(0)


if __name__ == "__main__":
    main()

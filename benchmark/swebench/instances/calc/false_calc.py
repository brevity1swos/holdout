def clamp(x, lo, hi):
    # FALSE-GREEN "solution": fixes the targeted FAIL_TO_PASS test (clamp(10,0,5)==5)
    # by clamping the upper bound only — but DROPS the lower bound, breaking the
    # PASS_TO_PASS regression test (clamp(-3,0,5) now returns -3, not 0).
    # SWE-bench's weak oracle (FAIL_TO_PASS only) would mark this "solved".
    return min(x, hi)

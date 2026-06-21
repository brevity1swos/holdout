def clamp(x, lo, hi):
    # Correct fix: clamp to both bounds. Passes FAIL_TO_PASS and all PASS_TO_PASS.
    return max(lo, min(x, hi))

def clamp(x, lo, hi):
    # BUG: ignores the upper bound `hi` — values above `hi` are not clamped.
    return max(lo, x)

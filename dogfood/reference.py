#!/usr/bin/env python3
"""HIDDEN ORACLE — do not read. f(n) = n, except n>100 prints OVER."""
import sys
def f(n):
    return "OVER" if n > 100 else str(n)
if __name__ == "__main__":
    print(f(int(sys.stdin.read().strip())))

from calc import clamp


def test_upper():
    # FAIL_TO_PASS: the buggy base returns 10; a correct fix returns 5.
    assert clamp(10, 0, 5) == 5


def test_lower():
    # PASS_TO_PASS: regression — must keep clamping below `lo`.
    assert clamp(-3, 0, 5) == 0


def test_within():
    # PASS_TO_PASS: regression — values in range are unchanged.
    assert clamp(3, 0, 5) == 3

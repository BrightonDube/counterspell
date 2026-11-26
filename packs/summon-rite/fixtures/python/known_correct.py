def can_summon(marks, target):
    def go(i, need):
        if need == 0:
            return True
        if i >= len(marks):
            return False
        return go(i + 1, need - marks[i]) or go(i + 1, need)
    return 1 if go(0, target) else 0

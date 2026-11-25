def lit_streak(lights):
    streak = 0
    for light in reversed(lights):
        if light == 0:
            break
        streak += 1
    return streak

def mirror_signal(signal):
    mismatches = 0
    for left in range(len(signal) // 2):
        right = len(signal) - left - 1
        if signal[left] != signal[right]:
            mismatches += 1
    return mismatches

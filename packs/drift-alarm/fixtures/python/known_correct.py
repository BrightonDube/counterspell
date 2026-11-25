def drift_alarms(readings, width):
    alarms = 0
    for i in range(width, len(readings)):
        if readings[i] > max(readings[i - width:i]):
            alarms += 1
    return alarms

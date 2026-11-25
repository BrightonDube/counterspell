def quiet_windows(readings, quota):
    quiet = 0
    for i in range(len(readings) - 2):
        if readings[i] + readings[i + 1] + readings[i + 2] <= quota:
            quiet += 1
    return quiet

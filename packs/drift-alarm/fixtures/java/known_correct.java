// Test-only known-correct Java fixture for drift-alarm.
public class Solution {
    public static long drift_alarms(long[] readings, long width) {
        long alarms = 0;
        for (int i = (int) width; i < readings.length; i += 1) {
            long peak = readings[i - (int) width];
            for (int j = i - (int) width + 1; j < i; j += 1) {
                if (readings[j] > peak) {
                    peak = readings[j];
                }
            }
            if (readings[i] > peak) {
                alarms += 1;
            }
        }
        return alarms;
    }
}

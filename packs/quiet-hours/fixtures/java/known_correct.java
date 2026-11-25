// Test-only known-correct Java fixture for quiet-hours.
public class Solution {
    public static long quiet_windows(long[] readings, long quota) {
        long quiet = 0;
        for (int i = 0; i + 2 < readings.length; i += 1) {
            if (readings[i] + readings[i + 1] + readings[i + 2] <= quota) {
                quiet += 1;
            }
        }
        return quiet;
    }
}

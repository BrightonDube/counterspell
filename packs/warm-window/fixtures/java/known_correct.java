// Test-only known-correct Java fixture for warm-window.
public class Solution {
    public static long warmest_window(long[] readings, long width) {
        if (readings.length < width) {
            return 0;
        }
        long total = 0;
        for (int i = 0; i < width; i += 1) {
            total += readings[i];
        }
        long best = total;
        for (int i = (int) width; i < readings.length; i += 1) {
            total += readings[i] - readings[i - (int) width];
            if (total > best) {
                best = total;
            }
        }
        return best;
    }
}

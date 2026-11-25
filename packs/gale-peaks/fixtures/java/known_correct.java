// Test-only known-correct Java fixture for gale-peaks.
public class Solution {
    public static long[] gale_peaks(long[] readings, long width) {
        long[] peaks = new long[Math.max(0, readings.length - (int) width + 1)];
        int[] candidates = new int[readings.length + 1];
        int head = 0;
        int tail = 0;
        int out = 0;
        for (int i = 0; i < readings.length; i += 1) {
            while (tail > head && readings[candidates[tail - 1]] <= readings[i]) {
                tail -= 1;
            }
            candidates[tail] = i;
            tail += 1;
            if (candidates[head] <= i - (int) width) {
                head += 1;
            }
            if (i >= (int) width - 1) {
                peaks[out] = readings[candidates[head]];
                out += 1;
            }
        }
        return peaks;
    }
}

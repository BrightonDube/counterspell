// Test-only known-correct Java fixture for relay-gap.
public class Solution {
    public static long relay_count(long[][] spans, long gap) {
        java.util.Arrays.sort(spans, (x, y) -> x[1] != y[1] ? Long.compare(x[1], y[1]) : Long.compare(x[0], y[0]));
        long n = 0;
        long end = -1000000000L;
        for (long[] s : spans) {
            if (s[0] >= end + gap) {
                n += 1;
                end = s[1];
            }
        }
        return n;
    }
}

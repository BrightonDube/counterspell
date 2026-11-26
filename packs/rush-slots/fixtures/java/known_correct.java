// Test-only known-correct Java fixture for rush-slots.
public class Solution {
    public static long pick_count(long[][] spans) {
        java.util.Arrays.sort(spans, (x, y) -> x[1] != y[1] ? Long.compare(x[1], y[1]) : Long.compare(x[0], y[0]));
        long n = 0;
        long end = -1;
        for (long[] s : spans) {
            if (s[0] > end) {
                n += 1;
                end = s[1];
            }
        }
        return n;
    }
}

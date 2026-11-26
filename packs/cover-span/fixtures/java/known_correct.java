// Test-only known-correct Java fixture for cover-span.
public class Solution {
    public static long span_cover(long[][] spans, long t) {
        java.util.Arrays.sort(spans, (x, y) -> x[0] != y[0] ? Long.compare(x[0], y[0]) : Long.compare(x[1], y[1]));
        long front = 0;
        int i = 0;
        long n = 0;
        while (front <= t) {
            long best = -1;
            while (i < spans.length && spans[i][0] <= front) {
                best = Math.max(best, spans[i][1]);
                i += 1;
            }
            if (best < front) {
                return -1;
            }
            n += 1;
            if (best >= t) {
                return n;
            }
            front = best + 1;
        }
        return n;
    }
}

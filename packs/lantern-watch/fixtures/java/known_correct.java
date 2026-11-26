// Test-only known-correct Java fixture for lantern-watch.
public class Solution {
    public static long watch_points(long[][] spans) {
        long[][] filtered = java.util.Arrays.stream(spans).filter(s -> s[0] <= s[1]).toArray(long[][]::new);
        java.util.Arrays.sort(filtered, (x, y) -> x[1] != y[1] ? Long.compare(x[1], y[1]) : Long.compare(x[0], y[0]));
        spans = filtered;
        long pts = 0;
        int i = 0;
        while (i < spans.length) {
            long p = spans[i][1];
            pts += 1;
            while (i < spans.length && spans[i][0] <= p) {
                i += 1;
            }
        }
        return pts;
    }
}

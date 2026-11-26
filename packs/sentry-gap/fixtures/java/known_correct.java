// Test-only known-correct Java fixture for sentry-gap.
public class Solution {
    static long f(long c, long s, long gap) {
        if (s == 0) {
            return 1;
        }
        if (c <= 0) {
            return 0;
        }
        return f(c - gap - 1, s - 1, gap) + f(c - 1, s, gap);
    }
    public static long sentry_ways(long cells, long sentries, long gap) {
        return f(cells, sentries, gap);
    }
}

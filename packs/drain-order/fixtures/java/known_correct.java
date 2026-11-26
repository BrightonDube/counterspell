// Test-only known-correct Java fixture for drain-order.
public class Solution {
    public static long[] drain_order(long[] events) {
        int n = events.length / 2;
        long[][] pairs = new long[n][2];
        for (int i = 0; i < n; i += 1) {
            pairs[i][0] = events[2 * i];
            pairs[i][1] = events[2 * i + 1];
        }
        java.util.Arrays.sort(pairs, (a, b) ->
            a[0] != b[0] ? Long.compare(a[0], b[0]) : Long.compare(a[1], b[1]));
        long[] ans = new long[n];
        for (int i = 0; i < n; i += 1) {
            ans[i] = pairs[i][1];
        }
        return ans;
    }
}

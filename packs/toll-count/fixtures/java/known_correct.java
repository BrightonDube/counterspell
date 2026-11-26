// Test-only known-correct Java fixture for toll-count.
public class Solution {
    public static long toll_count(long[][] edges, long[] weights, long t) {
        long count = 0;
        int len = Math.min(edges.length, weights.length);
        for (int k = 0; k < len; k += 1) {
            if (weights[k] <= t) {
                count += 1;
            }
        }
        return count;
    }
}

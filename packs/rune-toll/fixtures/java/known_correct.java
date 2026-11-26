// Test-only known-correct Java fixture for rune-toll.
public class Solution {
    public static long toll_sum(long[][] edges, long[] weights, long n) {
        long total = 0;
        int len = Math.min(edges.length, weights.length);
        for (int k = 0; k < len; k += 1) {
            total += weights[k];
        }
        return total;
    }
}

// Test-only known-correct Java fixture for near-mark.
public class Solution {
    public static long near_mark(long[][] edges, long[] weights, long n) {
        long best = -1;
        boolean found = false;
        int len = Math.min(edges.length, weights.length);
        for (int k = 0; k < len; k += 1) {
            long a = edges[k][0];
            long b = edges[k][1];
            long c = weights[k];
            if (a == 0 || b == 0) {
                if (!found || c < best) {
                    best = c;
                    found = true;
                }
            }
        }
        return found ? best : -1;
    }
}

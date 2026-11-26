// Test-only known-correct Java fixture for mask-rite.
public class Solution {
    public static long mask_rite(long[] weights, long cap) {
        int n = weights.length;
        long count = 0;
        for (long mask = 0; mask < (1L << n); mask += 1) {
            long total = 0;
            for (int i = 0; i < n; i += 1) {
                if (((mask >> i) & 1) == 1) {
                    total += weights[i];
                }
            }
            if (total <= cap) {
                count += 1;
            }
        }
        return count;
    }
}

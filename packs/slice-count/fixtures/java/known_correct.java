// Test-only known-correct Java fixture for slice-count.
public class Solution {
    public static long slice_count(long[] bursts, long q) {
        long total = 0;
        for (long b : bursts) {
            total += (b + q - 1) / q;
        }
        return total;
    }
}

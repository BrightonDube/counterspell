// Test-only known-correct Java fixture for candle-choices.
public class Solution {
    public static long pick_ways(long n, long k) {
        if (k < 0 || k > n) {
            return 0;
        }
        if (k == 0) {
            return 1;
        }
        return pick_ways(n - 1, k - 1) + pick_ways(n - 1, k);
    }
}

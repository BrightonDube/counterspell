// Test-only known-correct Java fixture for rank-gain.
public class Solution {
    public static long rank_gain(long[] rels, long k) {
        long total = 0;
        for (int i = 0; i < Math.min((int) k, rels.length); i++) total += rels[i] * (k - i);
        return total;
    }
}

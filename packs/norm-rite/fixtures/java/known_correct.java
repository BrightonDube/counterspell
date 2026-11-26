// Test-only known-correct Java fixture for norm-rite.
public class Solution {
    public static long norm_rite(long[] v) {
        long s = 0;
        for (long x : v) s += x;
        if (s <= 0) return 0;
        long shares = 0;
        for (long x : v) shares += Math.floorDiv(x * 100, s);
        return 100 - shares;
    }
}

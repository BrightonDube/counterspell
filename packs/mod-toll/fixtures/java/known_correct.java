// Test-only known-correct Java fixture for mod-toll.
public class Solution {
    public static long mod_toll(long[] values, long m) {
        long r = 0;
        for (long v : values) {
            r += v;
        }
        return ((r % m) + m) % m;
    }
}

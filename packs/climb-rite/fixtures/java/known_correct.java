// Test-only known-correct Java fixture for climb-rite.
public class Solution {
    public static long climb_ways(long steps) {
        if (steps <= 0) {
            return 1;
        }
        long a = 1;
        long b = 1;
        for (long i = 0; i < steps - 1; i += 1) {
            long t = a + b;
            a = b;
            b = t;
        }
        return b;
    }
}

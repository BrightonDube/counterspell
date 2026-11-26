// Test-only known-correct Java fixture for quad-mark.
public class Solution {
    public static long quad_mark(long x, long y) {
        if (x == 0 || y == 0) {
            return 0;
        }
        if (y > 0) {
            return x > 0 ? 1 : 2;
        }
        return x > 0 ? 4 : 3;
    }
}

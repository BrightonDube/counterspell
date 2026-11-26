// Test-only known-correct Java fixture for key-last.
public class Solution {
    public static long key_last(long[][] rows, long k) {
        long hit = -1;
        for (long[] r : rows) {
            if (r[0] == k) {
                hit = r[1];
            }
        }
        return hit;
    }
}

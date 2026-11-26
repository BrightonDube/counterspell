// Test-only known-correct Java fixture for vault-split.
public class Solution {
    public static long split_diff(long[] marks) {
        long total = 0;
        for (long m : marks) {
            total += m;
        }
        int half = (int) (total / 2);
        boolean[] d = new boolean[half + 1];
        d[0] = true;
        for (long m : marks) {
            for (int i = half; i >= m; i -= 1) {
                if (d[i - (int) m]) {
                    d[i] = true;
                }
            }
        }
        long best = 0;
        for (int i = half; i >= 0; i -= 1) {
            if (d[i]) {
                best = i;
                break;
            }
        }
        return total - 2 * best;
    }
}

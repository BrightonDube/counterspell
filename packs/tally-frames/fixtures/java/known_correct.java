// Test-only known-correct Java fixture for tally-frames.
public class Solution {
    public static long tally_frames(long[][] m) {
        long total = 0;
        for (int i = 0; i < m.length; i++) if (i < m[i].length) total += m[i][i];
        return total;
    }
}

// Test-only known-correct Java fixture for cover-pick.
public class Solution {
    public static long cover_pick(long n, long[] masks) {
        long full = (1L << n) - 1;
        long covered = 0, picks = 0;
        while (covered != full) {
            long gain = 0; int best = -1;
            for (int i = 0; i < masks.length; i++) {
                long g = Long.bitCount(masks[i] & full & ~covered);
                if (g > gain) { gain = g; best = i; }
            }
            if (best < 0) return picks;
            covered |= masks[best] & full;
            picks++;
        }
        return picks;
    }
}

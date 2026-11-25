// Test-only known-correct Java fixture for min-varied.
public class Solution {
    public static long shortest_varied(String inscription) {
        long[] counts = new long[3];
        int held = 0;
        int left = 0;
        long best = inscription.length() + 1;
        for (int right = 0; right < inscription.length(); right += 1) {
            char rune = inscription.charAt(right);
            if (rune != 'd') {
                int slot = rune - 'a';
                if (counts[slot] == 0) {
                    held += 1;
                }
                counts[slot] += 1;
            }
            while (held == 3) {
                best = Math.min(best, right - left + 1);
                char edge = inscription.charAt(left);
                if (edge != 'd') {
                    int slot = edge - 'a';
                    counts[slot] -= 1;
                    if (counts[slot] == 0) {
                        held -= 1;
                    }
                }
                left += 1;
            }
        }
        return best > inscription.length() ? 0 : best;
    }
}

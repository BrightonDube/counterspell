// Test-only known-correct Java fixture for thrifty-pane.
public class Solution {
    public static long longest_thrifty(String pane) {
        long[] counts = new long[3];
        int distinct = 0;
        int left = 0;
        long best = 0;
        for (int right = 0; right < pane.length(); right += 1) {
            int slot = pane.charAt(right) - 'a';
            if (counts[slot] == 0) {
                distinct += 1;
            }
            counts[slot] += 1;
            while (distinct > 2) {
                int edge = pane.charAt(left) - 'a';
                counts[edge] -= 1;
                if (counts[edge] == 0) {
                    distinct -= 1;
                }
                left += 1;
            }
            best = Math.max(best, right - left + 1);
        }
        return best;
    }
}

// Test-only known-correct Java fixture for three-rune-windows.
public class Solution {
    public static long count_varied(String inscription) {
        int n = inscription.length();
        long total = 0;
        for (int left = 0; left < n; left += 1) {
            boolean[] seen = new boolean[3];
            int missing = 3;
            for (int right = left; right < n; right += 1) {
                int slot = inscription.charAt(right) - 'a';
                if (!seen[slot]) {
                    seen[slot] = true;
                    missing -= 1;
                }
                if (missing == 0) {
                    total += n - right;
                    break;
                }
            }
        }
        return total;
    }
}

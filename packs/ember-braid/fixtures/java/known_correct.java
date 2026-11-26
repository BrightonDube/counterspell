// Test-only known-correct Java fixture for ember-braid.
public class Solution {
    public static long braid_ways(String text, String first, String second) {
        int n = first.length();
        int m = second.length();
        if (text.length() != n + m) {
            return 0;
        }
        long[][] d = new long[n + 1][m + 1];
        d[0][0] = 1;
        for (int i = 0; i <= n; i += 1) {
            for (int j = 0; j <= m; j += 1) {
                if (i > 0 && first.charAt(i - 1) == text.charAt(i + j - 1)) {
                    d[i][j] |= d[i - 1][j];
                }
                if (j > 0 && second.charAt(j - 1) == text.charAt(i + j - 1)) {
                    d[i][j] |= d[i][j - 1];
                }
            }
        }
        return d[n][m];
    }
}

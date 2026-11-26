// Test-only known-correct Java fixture for ember-echo.
public class Solution {
    public static long echo_len(String first, String second) {
        int n = first.length();
        int m = second.length();
        long[][] d = new long[n + 1][m + 1];
        for (int i = 1; i <= n; i += 1) {
            for (int j = 1; j <= m; j += 1) {
                if (first.charAt(i - 1) == second.charAt(j - 1)) {
                    d[i][j] = d[i - 1][j - 1] + 1;
                } else {
                    d[i][j] = Math.max(d[i - 1][j], d[i][j - 1]);
                }
            }
        }
        return d[n][m];
    }
}

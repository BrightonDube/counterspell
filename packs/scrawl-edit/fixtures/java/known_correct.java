// Test-only known-correct Java fixture for scrawl-edit.
public class Solution {
    public static long edit_toll(String first, String second) {
        int n = first.length();
        int m = second.length();
        long[][] d = new long[n + 1][m + 1];
        for (int i = 0; i <= n; i += 1) d[i][0] = i;
        for (int j = 0; j <= m; j += 1) d[0][j] = j;
        for (int i = 1; i <= n; i += 1) {
            for (int j = 1; j <= m; j += 1) {
                long cost = first.charAt(i - 1) == second.charAt(j - 1) ? 0 : 1;
                d[i][j] = Math.min(Math.min(d[i - 1][j] + 1, d[i][j - 1] + 1), d[i - 1][j - 1] + cost);
            }
        }
        return d[n][m];
    }
}

// Test-only known-correct Java fixture for grille-ways.
public class Solution {
    public static long grille_ways(long rows, long cols) {
        if (rows <= 0 || cols <= 0) {
            return 0;
        }
        int r = (int) rows;
        int c = (int) cols;
        long[][] d = new long[r][c];
        for (int i = 0; i < r; i += 1) {
            for (int j = 0; j < c; j += 1) {
                d[i][j] = (i == 0 || j == 0) ? 1 : d[i - 1][j] + d[i][j - 1];
            }
        }
        return d[r - 1][c - 1];
    }
}

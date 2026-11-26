// Test-only known-correct Java fixture for ember-grid.
public class Solution {
    public static long grid_min(long[][] grid) {
        int h = grid.length;
        int w = grid[0].length;
        long[][] d = new long[h][w];
        for (int i = 0; i < h; i += 1) {
            for (int j = 0; j < w; j += 1) {
                if (i == 0 && j == 0) {
                    d[i][j] = grid[0][0];
                } else {
                    long best = Math.min(i > 0 ? d[i - 1][j] : Long.MAX_VALUE / 2,
                                         j > 0 ? d[i][j - 1] : Long.MAX_VALUE / 2);
                    d[i][j] = best + grid[i][j];
                }
            }
        }
        return d[h - 1][w - 1];
    }
}

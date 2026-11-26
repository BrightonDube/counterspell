// Test-only known-correct Java fixture for grille-toll.
public class Solution {
    public static long toll_grid(long[][] grid) {
        int h = grid.length;
        int w = grid[0].length;
        long big = Long.MAX_VALUE / 4;
        long[][] d = new long[h][w];
        for (long[] row : d) {
            java.util.Arrays.fill(row, big);
        }
        for (int i = 0; i < h; i += 1) {
            for (int j = 0; j < w; j += 1) {
                if (grid[i][j] != 0) {
                    continue;
                }
                if (i == 0 && j == 0) {
                    d[i][j] = 1;
                } else {
                    long best = Math.min(i > 0 ? d[i - 1][j] : big,
                                         j > 0 ? d[i][j - 1] : big);
                    if (best < big) {
                        d[i][j] = best + 1;
                    }
                }
            }
        }
        return d[h - 1][w - 1] >= big ? -1 : d[h - 1][w - 1];
    }
}

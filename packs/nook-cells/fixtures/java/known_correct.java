// Test-only known-correct Java fixture for nook-cells.
public class Solution {
    public static long nook_ways(long[][] grid) {
        int h = grid.length;
        int w = grid[0].length;
        long[][] d = new long[h][w];
        for (int i = 0; i < h; i += 1) {
            for (int j = 0; j < w; j += 1) {
                if (grid[i][j] == 0) {
                    continue;
                }
                if (i == 0 && j == 0) {
                    d[i][j] = 1;
                } else {
                    d[i][j] = (i > 0 ? d[i - 1][j] : 0) + (j > 0 ? d[i][j - 1] : 0);
                }
            }
        }
        return d[h - 1][w - 1];
    }
}

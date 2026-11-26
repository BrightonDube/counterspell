// Test-only known-correct Java fixture for grille-paths.
public class Solution {
    static long f(long[][] grid, int r, int c) {
        if (r >= grid.length || c >= grid[0].length || grid[r][c] == 0) {
            return 0;
        }
        if (r == grid.length - 1 && c == grid[0].length - 1) {
            return 1;
        }
        return f(grid, r, c + 1) + f(grid, r + 1, c);
    }
    public static long path_count(long[][] grid) {
        return f(grid, 0, 0);
    }
}

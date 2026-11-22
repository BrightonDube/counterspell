// Test-only known-correct Java fixture for diagonal-total.
public class Solution {
    public static long diagonal_total(long[][] grid) {
        long edge = Math.min(grid.length, grid.length > 0 ? grid[0].length : 0);
        long total = 0;
        for (int row = 0; row < edge; row++) {
            total += grid[row][row];
        }
        return total;
    }
}

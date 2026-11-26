// Test-only known-correct Java fixture for grid-nook.
public class Solution {
    public static long nook_size(long[][] grid) {
        int rows = grid.length;
        int cols = grid[0].length;
        if (grid[0][0] == 0) {
            return 0;
        }
        boolean[][] seen = new boolean[rows][cols];
        seen[0][0] = true;
        java.util.ArrayDeque<int[]> stack = new java.util.ArrayDeque<>();
        stack.push(new int[]{0, 0});
        int[][] dirs = {{0, 1}, {0, -1}, {1, 0}, {-1, 0}};
        while (!stack.isEmpty()) {
            int[] cell = stack.pop();
            for (int[] d : dirs) {
                int nr = cell[0] + d[0];
                int nc = cell[1] + d[1];
                if (nr >= 0 && nr < rows && nc >= 0 && nc < cols && grid[nr][nc] == 1 && !seen[nr][nc]) {
                    seen[nr][nc] = true;
                    stack.push(new int[]{nr, nc});
                }
            }
        }
        long count = 0;
        for (boolean[] row : seen) {
            for (boolean v : row) {
                if (v) {
                    count += 1;
                }
            }
        }
        return count;
    }
}

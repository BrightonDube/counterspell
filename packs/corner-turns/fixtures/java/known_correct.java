// Test-only known-correct Java fixture for corner-turns.
public class Solution {
    public static long corner_turns(long[][] points) {
        long count = 0;
        for (int i = 1; i < points.length - 1; i += 1) {
            long ux = points[i][0] - points[i - 1][0];
            long uy = points[i][1] - points[i - 1][1];
            long vx = points[i + 1][0] - points[i][0];
            long vy = points[i + 1][1] - points[i][1];
            if (ux * vy - uy * vx > 0) {
                count += 1;
            }
        }
        return count;
    }
}

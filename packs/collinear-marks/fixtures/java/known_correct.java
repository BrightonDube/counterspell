// Test-only known-correct Java fixture for collinear-marks.
public class Solution {
    public static long collinear_marks(long[][] points) {
        if (points.length < 2 || (points[0][0] == points[1][0] && points[0][1] == points[1][1])) {
            return -1;
        }
        long x0 = points[0][0];
        long y0 = points[0][1];
        long dx = points[1][0] - x0;
        long dy = points[1][1] - y0;
        long count = 0;
        for (long[] p : points) {
            if ((p[0] - x0) * dy == (p[1] - y0) * dx) {
                count += 1;
            }
        }
        return count;
    }
}

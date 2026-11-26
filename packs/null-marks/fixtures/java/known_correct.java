// Test-only known-correct Java fixture for null-marks.
public class Solution {
    public static long null_marks(long[][] rows) {
        long count = 0;
        for (long[] r : rows) {
            if (r[1] == -1) {
                count += 1;
            }
        }
        return count;
    }
}

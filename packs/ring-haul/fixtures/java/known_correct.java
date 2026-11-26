// Test-only known-correct Java fixture for ring-haul.
public class Solution {
    static long line(long[] ms, int lo, int hi) {
        long take = 0;
        long skip = 0;
        for (int i = lo; i < hi; i += 1) {
            long t = skip + ms[i];
            skip = Math.max(skip, take);
            take = t;
        }
        return Math.max(take, skip);
    }
    public static long ring_haul(long[] marks) {
        int n = marks.length;
        if (n <= 1) {
            return n == 1 ? marks[0] : 0;
        }
        return Math.max(line(marks, 1, n), line(marks, 0, n - 1));
    }
}

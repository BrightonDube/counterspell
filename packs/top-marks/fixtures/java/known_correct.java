// Test-only known-correct Java fixture for top-marks.
public class Solution {
    public static long top_marks(long[] scores, long k) {
        Long[] s = java.util.Arrays.stream(scores).boxed()
            .sorted(java.util.Comparator.reverseOrder()).toArray(Long[]::new);
        long total = 0;
        for (int i = 0; i < Math.min((int) k, s.length); i++) total += s[i];
        return total;
    }
}

// Test-only known-correct Java fixture for ember-floor.
public class Solution {
    public static long floor_sum(long[] marks, long k) {
        long[] sorted = marks.clone();
        java.util.Arrays.sort(sorted);
        long total = 0;
        for (int i = 0; i < sorted.length && i < k; i += 1) {
            total += sorted[i];
        }
        return total;
    }
}

// Test-only known-correct Java fixture for kth-ember.
public class Solution {
    public static long kth_mark(long[] marks, long k) {
        long[] sorted = marks.clone();
        java.util.Arrays.sort(sorted);
        if (k > 0 && k <= sorted.length) {
            return sorted[(int) k - 1];
        }
        return -1;
    }
}

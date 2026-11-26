// Test-only known-correct Java fixture for ember-slate.
public class Solution {
    public static long slate_len(long[] marks) {
        if (marks.length == 0) {
            return 0;
        }
        long[] d = new long[marks.length];
        java.util.Arrays.fill(d, 1);
        long best = 1;
        for (int i = 0; i < marks.length; i += 1) {
            for (int j = 0; j < i; j += 1) {
                if (marks[j] < marks[i]) {
                    d[i] = Math.max(d[i], d[j] + 1);
                }
            }
            best = Math.max(best, d[i]);
        }
        return best;
    }
}

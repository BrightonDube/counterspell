// Test-only known-correct Java fixture for nest-count.
public class Solution {
    public static long nest_pairs(long[][] spans) {
        long n = 0;
        for (int i = 0; i < spans.length; i += 1) {
            for (int j = i + 1; j < spans.length; j += 1) {
                long a = spans[i][0], b = spans[i][1];
                long c = spans[j][0], d = spans[j][1];
                if ((a < c && d < b) || (c < a && b < d)) {
                    n += 1;
                }
            }
        }
        return n;
    }
}

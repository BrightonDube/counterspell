// Test-only known-correct Java fixture for gap-blocks.
public class Solution {
    public static long block_count(long[][] spans) {
        if (spans.length == 0) {
            return 0;
        }
        java.util.Arrays.sort(spans, (x, y) -> x[0] != y[0] ? Long.compare(x[0], y[0]) : Long.compare(x[1], y[1]));
        long blocks = 1;
        long end = spans[0][1];
        for (int i = 1; i < spans.length; i += 1) {
            long a = spans[i][0], b = spans[i][1];
            if (a > end) {
                blocks += 1;
                end = b;
            } else {
                end = Math.max(end, b);
            }
        }
        return blocks;
    }
}

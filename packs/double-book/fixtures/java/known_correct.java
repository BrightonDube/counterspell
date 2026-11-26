// Test-only known-correct Java fixture for double-book.
public class Solution {
    public static long room_count(long[][] spans) {
        long[][] evs = new long[spans.length * 2][2];
        for (int i = 0; i < spans.length; i += 1) {
            evs[2 * i] = new long[]{spans[i][0], 1};
            evs[2 * i + 1] = new long[]{spans[i][1], -1};
        }
        java.util.Arrays.sort(evs, (x, y) -> x[0] != y[0] ? Long.compare(x[0], y[0]) : Long.compare(y[1], x[1]));
        long cur = 0;
        long best = 0;
        for (long[] e : evs) {
            cur += e[1];
            if (cur > best) {
                best = cur;
            }
        }
        return best;
    }
}

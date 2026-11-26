// Test-only known-correct Java fixture for pack-shelf.
public class Solution {
    public static long pack_worth(long[][] items, long cap) {
        int c = (int) cap;
        long[] d = new long[c + 1];
        for (long[] item : items) {
            int w = (int) item[0];
            long v = item[1];
            for (int i = c; i >= w; i -= 1) {
                d[i] = Math.max(d[i], d[i - w] + v);
            }
        }
        return d[c];
    }
}

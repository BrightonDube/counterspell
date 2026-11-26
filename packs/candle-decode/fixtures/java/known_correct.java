// Test-only known-correct Java fixture for candle-decode.
public class Solution {
    public static long decode_ways(String glyphs) {
        int n = glyphs.length();
        if (n == 0) {
            return 1;
        }
        long[] d = new long[n + 1];
        d[0] = 1;
        for (int i = 1; i <= n; i += 1) {
            if (glyphs.charAt(i - 1) != '0') {
                d[i] += d[i - 1];
            }
            if (i >= 2) {
                int pair = Integer.parseInt(glyphs.substring(i - 2, i));
                if (pair >= 10 && pair <= 26) {
                    d[i] += d[i - 2];
                }
            }
        }
        return d[n];
    }
}

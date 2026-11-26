// Test-only known-correct Java fixture for token-window.
public class Solution {
    public static long token_window(long[] tokens, long w, long mark) {
        int width = (int) Math.min(w, tokens.length);
        if (width <= 0) return 0;
        long cur = 0;
        for (int i = 0; i < width; i++) if (tokens[i] == mark) cur++;
        long best = cur;
        for (int i = width; i < tokens.length; i++) {
            cur += (tokens[i] == mark ? 1 : 0) - (tokens[i - width] == mark ? 1 : 0);
            if (cur > best) best = cur;
        }
        return best;
    }
}

// Test-only known-correct Java fixture for longest-run-length.
public class Solution {
    public static long longest_run(String text) {
        long best = 0;
        long streak = 0;
        char last = 0;
        for (int i = 0; i < text.length(); i++) {
            char ch = text.charAt(i);
            streak = (i > 0 && ch == last) ? streak + 1 : 1;
            last = ch;
            if (streak > best) {
                best = streak;
            }
        }
        return best;
    }
}

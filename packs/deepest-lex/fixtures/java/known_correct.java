// Test-only known-correct Java fixture for deepest-lex.
public class Solution {
    public static long longest_word(String words) {
        long best = 0;
        for (String w : words.split(",", -1)) {
            if (!w.isEmpty() && w.length() > best) {
                best = w.length();
            }
        }
        return best;
    }
}

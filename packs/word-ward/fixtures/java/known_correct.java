// Test-only known-correct Java fixture for word-ward.
public class Solution {
    public static long has_word(String words, String word) {
        if (word.isEmpty()) {
            return 0;
        }
        for (String w : words.split(",", -1)) {
            if (w.equals(word)) {
                return 1;
            }
        }
        return 0;
    }
}

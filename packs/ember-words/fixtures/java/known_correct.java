// Test-only known-correct Java fixture for ember-words.
public class Solution {
    public static long word_break(String text, String lexicon) {
        java.util.List<String> words = new java.util.ArrayList<>();
        for (String w : lexicon.split(",", -1)) {
            if (!w.isEmpty()) {
                words.add(w);
            }
        }
        int n = text.length();
        long[] d = new boolean[n + 1];
        d[0] = 1;
        for (int i = 1; i <= n; i += 1) {
            for (String w : words) {
                if (i >= w.length() && d[i - w.length()] && text.substring(i - w.length(), i).equals(w)) {
                    d[i] = 1;
                    break;
                }
            }
        }
        return d[n];
    }
}

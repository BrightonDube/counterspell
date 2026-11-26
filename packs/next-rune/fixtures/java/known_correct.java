// Test-only known-correct Java fixture for next-rune.
public class Solution {
    public static long next_letters(String words, String prefix) {
        java.util.Set<Character> nxts = new java.util.HashSet<>();
        int k = prefix.length();
        for (String w : words.split(",", -1)) {
            if (!w.isEmpty() && w.startsWith(prefix) && w.length() > k) {
                nxts.add(w.charAt(k));
            }
        }
        return nxts.size();
    }
}

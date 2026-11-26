// Test-only known-correct Java fixture for prefix-tally.
public class Solution {
    public static long prefix_count(String words, String prefix) {
        long count = 0;
        for (String w : words.split(",", -1)) {
            if (!w.isEmpty() && w.startsWith(prefix)) {
                count += 1;
            }
        }
        return count;
    }
}

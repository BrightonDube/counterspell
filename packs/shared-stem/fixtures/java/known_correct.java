// Test-only known-correct Java fixture for shared-stem.
public class Solution {
    public static long common_stem(String words) {
        String stem = null;
        for (String w : words.split(",", -1)) {
            if (w.isEmpty()) {
                continue;
            }
            if (stem == null) {
                stem = w;
                continue;
            }
            while (!w.startsWith(stem)) {
                stem = stem.substring(0, stem.length() - 1);
            }
        }
        return stem == null ? 0 : stem.length();
    }
}

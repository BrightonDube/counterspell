// Test-only known-correct Java fixture for calm-spell.
public class Solution {
    public static long longest_calm(String chant) {
        long best = 0;
        long current = 0;
        for (int i = 0; i < chant.length(); i += 1) {
            if (chant.charAt(i) == '#') {
                current = 0;
            } else {
                current += 1;
                if (current > best) {
                    best = current;
                }
            }
        }
        return best;
    }
}

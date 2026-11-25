// Test-only known-correct Java fixture for deepest-ember.
public class Solution {
    public static long deepest_ember(String inscription) {
        long depth = 0;
        long deepest = 0;
        for (int index = 0; index < inscription.length(); index += 1) {
            if (inscription.charAt(index) == '(') {
                depth += 1;
                if (depth > deepest) {
                    deepest = depth;
                }
            } else if (depth > 0) {
                depth -= 1;
            }
        }
        return deepest;
    }
}

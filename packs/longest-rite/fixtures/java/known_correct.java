// Test-only known-correct Java fixture for longest-rite.
public class Solution {
    public static long longest_nested(String inscription) {
        long best = 0;
        int[] stack = new int[inscription.length() + 1];
        int top = 0;
        stack[top] = -1;
        top += 1;
        for (int index = 0; index < inscription.length(); index += 1) {
            if (inscription.charAt(index) == '(') {
                stack[top] = index;
                top += 1;
            } else {
                top -= 1;
                if (top > 0) {
                    long span = index - stack[top - 1];
                    if (span > best) {
                        best = span;
                    }
                } else {
                    stack[top] = index;
                    top += 1;
                }
            }
        }
        return best;
    }
}

// Test-only known-correct Java fixture for stack-sigils.
public class Solution {
    public static long top_sigil(long[] ops) {
        long[] marks = new long[ops.length];
        int top = 0;
        for (long op : ops) {
            if (op == 0) {
                if (top > 0) {
                    top -= 1;
                }
            } else {
                marks[top] = op;
                top += 1;
            }
        }
        return top == 0 ? -1 : marks[top - 1];
    }
}

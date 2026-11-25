// Test-only known-correct Java fixture for cauldron-queue.
public class Solution {
    public static long servings(long[] ops) {
        long[] line = new long[ops.length];
        int front = 0;
        int back = 0;
        long served = 0;
        for (long op : ops) {
            if (op == 0) {
                if (front < back) {
                    front += 1;
                    served += 1;
                }
            } else {
                line[back] = op;
                back += 1;
            }
        }
        return served;
    }
}

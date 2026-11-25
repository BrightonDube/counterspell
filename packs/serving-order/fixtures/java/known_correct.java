// Test-only known-correct Java fixture for serving-order.
public class Solution {
    public static long[] serve_order(long[] ops) {
        long[] line = new long[ops.length];
        long[] served = new long[ops.length];
        int front = 0;
        int back = 0;
        int count = 0;
        for (long op : ops) {
            if (op == 0) {
                if (front < back) {
                    served[count] = line[front];
                    count += 1;
                    front += 1;
                }
            } else {
                line[back] = op;
                back += 1;
            }
        }
        return java.util.Arrays.copyOf(served, count);
    }
}

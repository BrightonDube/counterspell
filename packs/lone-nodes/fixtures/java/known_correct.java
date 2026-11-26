// Test-only known-correct Java fixture for lone-nodes.
public class Solution {
    public static long lone_count(long[][] edges, long n) {
        int size = (int) n;
        if (size <= 0) {
            return 0;
        }
        boolean[] touched = new boolean[size];
        for (long[] e : edges) {
            int a = (int) e[0];
            int b = (int) e[1];
            if (a >= 0 && a < size) {
                touched[a] = true;
            }
            if (b >= 0 && b < size) {
                touched[b] = true;
            }
        }
        long count = 0;
        for (boolean t : touched) {
            if (!t) {
                count += 1;
            }
        }
        return count;
    }
}

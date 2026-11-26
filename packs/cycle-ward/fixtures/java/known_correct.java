// Test-only known-correct Java fixture for cycle-ward.
public class Solution {
    static int find(int[] parent, int x) {
        while (parent[x] != x) {
            parent[x] = parent[parent[x]];
            x = parent[x];
        }
        return x;
    }
    public static long cycle_check(long[][] edges, long n) {
        int size = (int) n;
        int[] parent = new int[size];
        for (int i = 0; i < size; i += 1) {
            parent[i] = i;
        }
        for (long[] e : edges) {
            int a = (int) e[0];
            int b = (int) e[1];
            if (!(a >= 0 && a < size && b >= 0 && b < size)) {
                continue;
            }
            if (a == b) {
                return 1;
            }
            int ra = find(parent, a);
            int rb = find(parent, b);
            if (ra == rb) {
                return 1;
            }
            parent[ra] = rb;
        }
        return 0;
    }
}

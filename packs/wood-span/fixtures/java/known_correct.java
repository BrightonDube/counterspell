// Test-only known-correct Java fixture for wood-span.
public class Solution {
    static int find(int[] parent, int x) {
        while (parent[x] != x) {
            parent[x] = parent[parent[x]];
            x = parent[x];
        }
        return x;
    }
    public static long span_toll(long[][] edges, long[] weights, long n) {
        int size = (int) n;
        if (size <= 0) {
            return 0;
        }
        int[] parent = new int[size];
        for (int i = 0; i < size; i += 1) {
            parent[i] = i;
        }
        java.util.List<long[]> arcs = new java.util.ArrayList<>();
        int len = Math.min(edges.length, weights.length);
        for (int k = 0; k < len; k += 1) {
            int a = (int) edges[k][0];
            int b = (int) edges[k][1];
            long c = weights[k];
            if (a >= 0 && a < size && b >= 0 && b < size && a != b) {
                arcs.add(new long[]{c, a, b});
            }
        }
        arcs.sort((x, y) -> Long.compare(x[0], y[0]));
        long total = 0;
        long used = 0;
        for (long[] e : arcs) {
            int ra = find(parent, (int) e[1]);
            int rb = find(parent, (int) e[2]);
            if (ra != rb) {
                parent[ra] = rb;
                total += e[0];
                used += 1;
            }
        }
        return used == size - 1 ? total : -1;
    }
}

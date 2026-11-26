// Test-only known-correct Java fixture for light-isles.
public class Solution {
    static int find(int[] parent, int x) {
        while (parent[x] != x) {
            parent[x] = parent[parent[x]];
            x = parent[x];
        }
        return x;
    }
    public static long light_isles(long[][] edges, long[] weights, long n, long t) {
        int size = (int) n;
        int[] parent = new int[size];
        for (int i = 0; i < size; i += 1) {
            parent[i] = i;
        }
        int len = Math.min(edges.length, weights.length);
        for (int k = 0; k < len; k += 1) {
            int a = (int) edges[k][0];
            int b = (int) edges[k][1];
            long c = weights[k];
            if (c <= t && a >= 0 && a < size && b >= 0 && b < size) {
                int ra = find(parent, a);
                int rb = find(parent, b);
                if (ra != rb) {
                    parent[ra] = rb;
                }
            }
        }
        java.util.HashSet<Integer> roots = new java.util.HashSet<>();
        for (int i = 0; i < size; i += 1) {
            roots.add(find(parent, i));
        }
        return roots.size();
    }
}

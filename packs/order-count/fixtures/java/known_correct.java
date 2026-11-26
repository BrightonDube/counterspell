// Test-only known-correct Java fixture for order-count.
public class Solution {
    static long go(int n, int k, java.util.List<java.util.List<Integer>> adj, int[] indeg) {
        if (k == n) {
            return 1;
        }
        long total = 0;
        for (int v = 0; v < n; v += 1) {
            if (indeg[v] == 0) {
                indeg[v] = -1;
                for (int w : adj.get(v)) {
                    indeg[w] -= 1;
                }
                total += go(n, k + 1, adj, indeg);
                for (int w : adj.get(v)) {
                    indeg[w] += 1;
                }
                indeg[v] = 0;
            }
        }
        return total;
    }
    public static long order_count(long[][] edges, long n) {
        int size = (int) n;
        java.util.List<java.util.List<Integer>> adj = new java.util.ArrayList<>();
        for (int i = 0; i < size; i += 1) {
            adj.add(new java.util.ArrayList<>());
        }
        int[] indeg = new int[size];
        boolean selfOrder = false;
        for (long[] e : edges) {
            int a = (int) e[0];
            int b = (int) e[1];
            if (!(a >= 0 && a < size && b >= 0 && b < size)) {
                continue;
            }
            if (a == b) {
                selfOrder = true;
                continue;
            }
            if (!adj.get(a).contains(b)) {
                adj.get(a).add(b);
                indeg[b] += 1;
            }
        }
        if (selfOrder) {
            return 0;
        }
        return go(size, 0, adj, indeg);
    }
}

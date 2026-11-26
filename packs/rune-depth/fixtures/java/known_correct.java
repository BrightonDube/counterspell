// Test-only known-correct Java fixture for rune-depth.
public class Solution {
    static boolean dfsDepth(int u, java.util.List<java.util.List<Integer>> adj, int[] state, int[] memo) {
        state[u] = 1;
        int best = 0;
        for (int w : adj.get(u)) {
            if (state[w] == 1) {
                return false;
            }
            if (state[w] == 0 && !dfsDepth(w, adj, state, memo)) {
                return false;
            }
            best = Math.max(best, memo[w]);
        }
        state[u] = 2;
        memo[u] = best + 1;
        return true;
    }
    public static long rune_depth(long[][] edges, long n) {
        int size = (int) n;
        java.util.List<java.util.List<Integer>> adj = new java.util.ArrayList<>();
        for (int i = 0; i < size; i += 1) {
            adj.add(new java.util.ArrayList<>());
        }
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
            }
        }
        if (selfOrder) {
            return 0;
        }
        int[] state = new int[size];
        int[] memo = new int[size];
        for (int v = 0; v < size; v += 1) {
            if (state[v] == 0 && !dfsDepth(v, adj, state, memo)) {
                return 0;
            }
        }
        int best = 0;
        for (int m : memo) {
            best = Math.max(best, m);
        }
        return best;
    }
}

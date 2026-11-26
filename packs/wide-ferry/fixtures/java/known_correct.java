// Test-only known-correct Java fixture for wide-ferry.
public class Solution {
    public static long ferry_load(long[][] edges, long[] weights, long n) {
        int size = (int) n;
        if (size <= 1) {
            return size == 1 ? 0 : -1;
        }
        java.util.List<java.util.List<long[]>> adj = new java.util.ArrayList<>();
        for (int i = 0; i < size; i += 1) {
            adj.add(new java.util.ArrayList<>());
        }
        int len = Math.min(edges.length, weights.length);
        for (int k = 0; k < len; k += 1) {
            int a = (int) edges[k][0];
            int b = (int) edges[k][1];
            long c = weights[k];
            if (a >= 0 && a < size && b >= 0 && b < size) {
                adj.get(a).add(new long[]{b, c});
                adj.get(b).add(new long[]{a, c});
            }
        }
        long[] best = new long[size];
        java.util.Arrays.fill(best, -1);
        best[0] = Long.MAX_VALUE;
        java.util.PriorityQueue<long[]> pq = new java.util.PriorityQueue<>((x, y) -> Long.compare(y[0], x[0]));
        pq.add(new long[]{best[0], 0});
        while (!pq.isEmpty()) {
            long[] top = pq.poll();
            long cap = top[0];
            int u = (int) top[1];
            if (cap < best[u]) {
                continue;
            }
            for (long[] e : adj.get(u)) {
                int v = (int) e[0];
                long cand = Math.min(cap, e[1]);
                if (cand > best[v]) {
                    best[v] = cand;
                    pq.add(new long[]{cand, v});
                }
            }
        }
        return best[size - 1];
    }
}

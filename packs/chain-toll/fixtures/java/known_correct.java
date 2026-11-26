// Test-only known-correct Java fixture for chain-toll.
public class Solution {
    public static long toll_ways(long[][] edges, long[] weights, long n) {
        int size = (int) n;
        if (size <= 0) {
            return 0;
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
        long[] dist = new long[size];
        java.util.Arrays.fill(dist, Long.MAX_VALUE);
        long[] ways = new long[size];
        dist[0] = 0;
        ways[0] = 1;
        java.util.PriorityQueue<long[]> pq = new java.util.PriorityQueue<>((x, y) -> Long.compare(x[0], y[0]));
        pq.add(new long[]{0, 0});
        while (!pq.isEmpty()) {
            long[] top = pq.poll();
            long du = top[0];
            int u = (int) top[1];
            if (du > dist[u]) {
                continue;
            }
            for (long[] e : adj.get(u)) {
                int v = (int) e[0];
                long nd = du + e[1];
                if (nd < dist[v]) {
                    dist[v] = nd;
                    ways[v] = ways[u];
                    pq.add(new long[]{nd, v});
                } else if (nd == dist[v]) {
                    ways[v] += ways[u];
                }
            }
        }
        return dist[size - 1] == Long.MAX_VALUE ? 0 : ways[size - 1];
    }
}

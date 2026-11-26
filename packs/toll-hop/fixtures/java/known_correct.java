// Test-only known-correct Java fixture for toll-hop.
public class Solution {
    public static long hop_toll(long[][] edges, long[] weights, long n, long k) {
        int size = (int) n;
        int cap = (int) k;
        if (size <= 0 || cap < 0) {
            return -1;
        }
        java.util.List<java.util.List<long[]>> adj = new java.util.ArrayList<>();
        for (int i = 0; i < size; i += 1) {
            adj.add(new java.util.ArrayList<>());
        }
        int len = Math.min(edges.length, weights.length);
        for (int i = 0; i < len; i += 1) {
            int a = (int) edges[i][0];
            int b = (int) edges[i][1];
            long c = weights[i];
            if (a >= 0 && a < size && b >= 0 && b < size) {
                adj.get(a).add(new long[]{b, c});
                adj.get(b).add(new long[]{a, c});
            }
        }
        long[][] dist = new long[size][cap + 1];
        for (long[] row : dist) {
            java.util.Arrays.fill(row, Long.MAX_VALUE);
        }
        dist[0][0] = 0;
        java.util.PriorityQueue<long[]> pq = new java.util.PriorityQueue<>((x, y) -> Long.compare(x[0], y[0]));
        pq.add(new long[]{0, 0, 0});
        while (!pq.isEmpty()) {
            long[] top = pq.poll();
            long du = top[0];
            int u = (int) top[1];
            int h = (int) top[2];
            if (du > dist[u][h]) {
                continue;
            }
            for (long[] e : adj.get(u)) {
                int v = (int) e[0];
                if (h + 1 <= cap && du + e[1] < dist[v][h + 1]) {
                    dist[v][h + 1] = du + e[1];
                    pq.add(new long[]{dist[v][h + 1], v, h + 1});
                }
            }
        }
        long best = Long.MAX_VALUE;
        for (long d : dist[size - 1]) {
            best = Math.min(best, d);
        }
        return best == Long.MAX_VALUE ? -1 : best;
    }
}

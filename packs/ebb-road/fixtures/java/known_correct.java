// Test-only known-correct Java fixture for ebb-road.
public class Solution {
    static final long RING = -1000000000000000L;
    public static long ebb_toll(long[][] edges, long[] weights, long n) {
        int size = (int) n;
        if (size <= 0) {
            return -1;
        }
        java.util.List<long[]> arcs = new java.util.ArrayList<>();
        int len = Math.min(edges.length, weights.length);
        for (int k = 0; k < len; k += 1) {
            int a = (int) edges[k][0];
            int b = (int) edges[k][1];
            long c = weights[k];
            if (a >= 0 && a < size && b >= 0 && b < size) {
                arcs.add(new long[]{a, b, c});
            }
        }
        long[] dist = new long[size];
        java.util.Arrays.fill(dist, Long.MAX_VALUE / 4);
        dist[0] = 0;
        for (int i = 0; i < size - 1; i += 1) {
            for (long[] e : arcs) {
                int a = (int) e[0];
                int b = (int) e[1];
                if (dist[a] < Long.MAX_VALUE / 4 && dist[a] + e[2] < dist[b]) {
                    dist[b] = dist[a] + e[2];
                }
            }
        }
        boolean[] marked = new boolean[size];
        for (int i = 0; i < size; i += 1) {
            for (long[] e : arcs) {
                int a = (int) e[0];
                int b = (int) e[1];
                if ((dist[a] < Long.MAX_VALUE / 4 && dist[a] + e[2] < dist[b]) || marked[a]) {
                    marked[b] = true;
                }
            }
        }
        if (marked[size - 1]) {
            return RING;
        }
        return dist[size - 1] >= Long.MAX_VALUE / 4 ? -1 : dist[size - 1];
    }
}

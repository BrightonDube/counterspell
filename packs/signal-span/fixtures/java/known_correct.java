// Test-only known-correct Java fixture for signal-span.
public class Solution {
    public static long span_max(long[][] edges, long n) {
        int size = (int) n;
        if (size <= 0) {
            return 0;
        }
        java.util.List<java.util.List<Integer>> adj = new java.util.ArrayList<>();
        for (int i = 0; i < size; i += 1) {
            adj.add(new java.util.ArrayList<>());
        }
        for (long[] e : edges) {
            int a = (int) e[0];
            int b = (int) e[1];
            if (a >= 0 && a < size && b >= 0 && b < size) {
                adj.get(a).add(b);
                adj.get(b).add(a);
            }
        }
        int[] dist = new int[size];
        java.util.Arrays.fill(dist, -1);
        dist[0] = 0;
        java.util.ArrayDeque<Integer> queue = new java.util.ArrayDeque<>();
        queue.add(0);
        while (!queue.isEmpty()) {
            int x = queue.poll();
            for (int y : adj.get(x)) {
                if (dist[y] < 0) {
                    dist[y] = dist[x] + 1;
                    queue.add(y);
                }
            }
        }
        int best = 0;
        for (int d : dist) {
            best = Math.max(best, d);
        }
        return best;
    }
}

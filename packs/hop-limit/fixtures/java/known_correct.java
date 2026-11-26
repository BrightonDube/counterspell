// Test-only known-correct Java fixture for hop-limit.
public class Solution {
    public static long hop_limit(long n, long[][] links, long ttl) {
        if (n == 0) return 0;
        java.util.List<java.util.List<Integer>> adj = new java.util.ArrayList<>();
        for (int i = 0; i < n; i++) adj.add(new java.util.ArrayList<>());
        for (long[] e : links) { if (0 <= e[0] && e[0] < n && 0 <= e[1] && e[1] < n) { adj.get((int) e[0]).add((int) e[1]); adj.get((int) e[1]).add((int) e[0]); } }
        int[] dist = new int[(int) Math.max(n, 1)];
        java.util.Arrays.fill(dist, -1);
        dist[0] = 0;
        int[] queue = new int[(int) Math.max(n, 1)];
        int head = 0, tail = 0;
        queue[tail++] = 0;
        while (head < tail) {
            int u = queue[head++];
            if (dist[u] >= ttl) continue;
            for (int v : adj.get(u)) {
                if (dist[v] < 0) { dist[v] = dist[u] + 1; queue[tail++] = v; }
            }
        }
        long count = 0;
        for (int d : dist) if (d >= 0) count++;
        return count - 1;
    }
}

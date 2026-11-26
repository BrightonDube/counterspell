// Test-only known-correct Java fixture for route-count.
public class Solution {
    public static long route_count(long n, long[][] links) {
        if (n == 0) return 0;
        java.util.List<java.util.List<Integer>> adj = new java.util.ArrayList<>();
        for (int i = 0; i < n; i++) adj.add(new java.util.ArrayList<>());
        for (long[] e : links) { if (0 <= e[0] && e[0] < n && 0 <= e[1] && e[1] < n) { adj.get((int) e[0]).add((int) e[1]); adj.get((int) e[1]).add((int) e[0]); } }
        boolean[] seen = new boolean[(int) n];
        seen[0] = true;
        int[] queue = new int[(int) Math.max(n, 1)];
        int head = 0, tail = 0;
        queue[tail++] = 0;
        while (head < tail) {
            for (int v : adj.get(queue[head++])) {
                if (!seen[v]) { seen[v] = true; queue[tail++] = v; }
            }
        }
        long count = 0;
        for (boolean s : seen) if (s) count++;
        return count - 1;
    }
}

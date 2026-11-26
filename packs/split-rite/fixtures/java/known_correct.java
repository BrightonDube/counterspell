// Test-only known-correct Java fixture for split-rite.
public class Solution {
    public static long split_rite(long n, long[][] links, long[] down) {
        java.util.Set<Long> dead = new java.util.HashSet<>();
        for (long d : down) dead.add(d);
        java.util.List<java.util.List<Integer>> adj = new java.util.ArrayList<>();
        for (int i = 0; i < n; i++) adj.add(new java.util.ArrayList<>());
        for (int i = 0; i < links.length; i++) {
            if (dead.contains((long) i)) continue;
            if (0 <= links[i][0] && links[i][0] < n && 0 <= links[i][1] && links[i][1] < n) {
                adj.get((int) links[i][0]).add((int) links[i][1]);
                adj.get((int) links[i][1]).add((int) links[i][0]);
            }
        }
        boolean[] seen = new boolean[(int) Math.max(n, 1)];
        long quorum = 0;
        for (int s = 0; s < n; s++) {
            if (seen[s]) continue;
            int size = 0;
            int[] queue = new int[(int) Math.max(n, 1)];
            int head = 0, tail = 0;
            queue[tail++] = s;
            seen[s] = true;
            while (head < tail) {
                size++;
                for (int v : adj.get(queue[head++])) {
                    if (!seen[v]) { seen[v] = true; queue[tail++] = v; }
                }
            }
            if (size * 2 > n) quorum++;
        }
        return quorum;
    }
}

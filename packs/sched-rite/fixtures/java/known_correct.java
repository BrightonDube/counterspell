// Test-only known-correct Java fixture for sched-rite.
public class Solution {
    private static boolean visit(java.util.Map<Long, java.util.List<Long>> adj,
                                 java.util.Map<Long, Integer> state, long u) {
        state.put(u, 1);
        for (long v : adj.getOrDefault(u, java.util.List.of())) {
            int s = state.getOrDefault(v, 0);
            if (s == 1) {
                return true;
            }
            if (s == 0 && visit(adj, state, v)) {
                return true;
            }
        }
        state.put(u, 2);
        return false;
    }

    public static long sched_rite(long[][] pairs) {
        java.util.Map<Long, java.util.List<Long>> adj = new java.util.HashMap<>();
        for (long[] p : pairs) {
            adj.computeIfAbsent(p[0], k -> new java.util.ArrayList<>()).add(p[1]);
        }
        java.util.Map<Long, Integer> state = new java.util.HashMap<>();
        for (long u : adj.keySet()) {
            if (state.getOrDefault(u, 0) == 0 && visit(adj, state, u)) {
                return 0;
            }
        }
        return 1;
    }
}
